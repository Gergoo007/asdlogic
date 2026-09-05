use std::{f32::consts::PI, fmt::Debug, thread::{self, JoinHandle}};

use glam::IVec2;
use imgui::{Key::{self}, MouseButton};
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use strum::{EnumMessage, IntoEnumIterator};
use wgpu::{Queue, RenderPass};

use crate::{canvas::{component::{CompKind, Component}, history::{Action, History}, inner::CanvasInner, nodes::{NodeHandler, check_driven, set_nodes}, renderer::CanvasRenderer, wires::{Wire, WireKey, Wires}}, config};

mod component;
mod wires;
mod inner;
mod logic;
mod nodes;
mod renderer;
mod history;
mod keybinds;

pub type Vec2 = glam::Vec2;

pub type CompStorage = typed_generational_arena::StandardArena<Component>;
pub type CompKey = typed_generational_arena::StandardIndex<Component>;

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ElemIndex {
	Wire(WireKey),
	Comp(CompKey),
}

fn vec2int(float: Vec2) -> IVec2 {
	IVec2 { x: float.x as i32, y: float.y as i32 }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum FileAction {
	Save,
	Load,
	#[default]
	None
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ClipboardElement {
	Wire { start: Vec2, end: Vec2},
	Component { kind: CompKind, pos: Vec2 },
}

#[derive(Serialize, Deserialize)]
pub struct Canvas {
	pub inner: CanvasInner,
	pub comps: CompStorage,
	pub wires: Wires,
	pub nodes: NodeHandler,
	pub update_generation: u32,

	#[serde(skip)]
	dialog_thread: Option<JoinHandle<String>>,
	#[serde(skip)]
	action: FileAction,
	#[serde(skip)]
	selection: Vec<ElemIndex>,
	#[serde(skip)]
	selection_start: Vec2,
	#[serde(skip)]
	selection_ongoing: bool,

	clipboard: Vec<ClipboardElement>,
	
	#[serde(skip)]
	pasting_ongoing: bool,

	#[serde(skip)]
	pub renderer: Option<CanvasRenderer>,

	#[serde(skip)]
	benchmarking: bool,
	#[serde(skip)]
	benchmark_cursor: Vec2, // polar coordinates; x is the radius, y is the angle
	#[serde(skip)]
	benchmark_frame_counter: u32, // mennyi képkocka deltája volt <40 fps; ha meg lesz az 5, akkor áll le a bm.

	// A grafikonhoz
	#[serde(skip)]
	deltas: Vec<f32>,
	#[serde(skip)]
	deltas_datapoints: usize,

	history: History,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	pos: [f32; 2],
}

impl Canvas {
	pub fn new(size: &Vec2, device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration) -> Self {
		let mut s = Self {
			inner: CanvasInner::new(size, device, surface_desc),
			comps: CompStorage::new(),
			wires: Wires::new(),
			nodes: NodeHandler::new(),
			update_generation: 0,
			dialog_thread: None,
			action: FileAction::None,
			selection: Vec::new(),
			selection_start: Vec2::new(0.0, 0.0),
			selection_ongoing: false,
			clipboard: Vec::new(),
			pasting_ongoing: false,
			renderer: Some(CanvasRenderer::new(device, surface_desc)),

			benchmarking: false,
			benchmark_cursor: Vec2::new(0.0, 0.0),
			benchmark_frame_counter: 0,
			deltas: Vec::with_capacity(100),
			deltas_datapoints: 100,

			history: History::new()
		};

		s.start_record();

		s.add_and_execute(&Action::AddComp { to: Vec2::new(0.0, 5.0), kind: CompKind::NandGate });
		s.add_and_execute(&Action::AddComp { to: Vec2::new(0.0, -5.0), kind: CompKind::OrGate });
		s.add_and_execute(&Action::AddComp { to: Vec2::new(0.0, 0.0), kind: CompKind::AndGate });

		s.checked_add_wire(Vec2::new(0.0, 1.0), Vec2::new(-5.0, 1.0));
		s.add_and_execute(&Action::AddComp { to: Vec2::new(-7.0, 0.0), kind: CompKind::Input { state: false } });

		s.checked_add_wire(Vec2::new(0.0, 3.0), Vec2::new(-5.0, 3.0));
		s.add_and_execute(&Action::AddComp { to: Vec2::new(-7.0, 2.0), kind: CompKind::Input { state: false } });

		s.checked_add_wire(Vec2::new(4.0, 2.0), Vec2::new(8.0, 2.0));

		s.end_record();

		s
	}

	pub fn checked_split(&mut self, start: Vec2, end: Vec2, at: Vec2) {
		self.add_and_execute(&Action::OverwriteWire {
			oldstart: start,
			oldend: end,
			newstart: start,
			newend: at,
		});
		self.add_and_execute(&Action::AddWire {
			from: at,
			to: end,
		});
	}

	pub fn checked_add_wire(&mut self, from: Vec2, to: Vec2) {
		let new = Wire { start: from, end: to, startnode: None, endnode: None, selected: false };

		let mut runanother = true;
		let mut insert = true;

		'check: while runanother {
			runanother = false;

			let keys: Vec<_> = self.wires.wires.iter().map(|(e, _)| e).collect();
			for oldidx in keys {
				let old = &self.wires.wires[oldidx];
				let v1 = old.start - old.end;
				let v2 = new.start - new.end;
				let parallel = v1.perp_dot(v2).abs() < 1e-6;

				// Egy régi vezetékben elfér az új (tehát az új nem kell)
				if old.touches(new.start) && old.touches(new.end) {
					insert = false;
					break 'check;
				}

				// Az új vezetékben elfér egy másik régebbi
				if new.touches(old.start) && new.touches(old.end) {
					// A régi vezetéknek vannak csatlakozásai
					if self.nodes.count_nodes(old.start) >= 2 || self.nodes.count_nodes(old.end) >= 2 {
						// Ezt az esetet hagyjuk is a gecibe, én biztos nem fogom ezt kezelni
						return;
					}

					self.add_and_execute(&Action::OverwriteWire { oldstart: old.start, oldend: old.end, newstart: new.start, newend: new.end });
					runanother = true; continue 'check;
				}

				// Az új vezetéket össze lehet vonni egy meglévővel (csak érintkeznek)
				if parallel {
					if old.start == new.start && self.nodes.count_nodes(old.start) <= 1 {
						self.add_and_execute(&Action::OverwriteWire { oldstart: old.start, oldend: old.end, newstart: new.end, newend: old.end });
						runanother = true; continue 'check;
					} else if old.start == new.end && self.nodes.count_nodes(old.start) <= 1 {
						self.add_and_execute(&Action::OverwriteWire { oldstart: old.start, oldend: old.end, newstart: old.end, newend: new.start });
						runanother = true; continue 'check;
					} else if old.end == new.start && self.nodes.count_nodes(old.end) <= 1 {
						self.add_and_execute(&Action::OverwriteWire { oldstart: old.start, oldend: old.end, newstart: old.start, newend: new.end });
						runanother = true; continue 'check;
					} else if old.end == new.end && self.nodes.count_nodes(old.end) <= 1 {
						self.add_and_execute(&Action::OverwriteWire { oldstart: old.start, oldend: old.end, newstart: old.start, newend: new.start });
						runanother = true; continue 'check;
					}

					// Az új vezetéket össze lehet vonni egy meglévővel (az egyik a másikból indul ki + párhuzamosak)
					if old.touches(new.start) && self.nodes.count_nodes(new.start) <= 1 {
						if new.touches(old.start) && self.nodes.count_nodes(old.start) <= 1 {
							self.add_and_execute(&Action::OverwriteWire { oldstart: old.start, oldend: old.end, newstart: old.end, newend: new.end });
							runanother = true; continue 'check;
						} else if new.touches(old.end) && self.nodes.count_nodes(old.end) <= 1 {
							self.add_and_execute(&Action::OverwriteWire { oldstart: old.start, oldend: old.end, newstart: old.start, newend: new.end, });
							runanother = true; continue 'check;
						}
					} else if old.touches(new.end) && self.nodes.count_nodes(new.end) <= 1 {
						if new.touches(old.start) && self.nodes.count_nodes(old.start) <= 1 {
							self.add_and_execute(&Action::OverwriteWire { oldstart: old.start, oldend: old.end, newstart: old.end, newend: new.start });
							runanother = true; continue 'check;
						} else if new.touches(old.end) && self.nodes.count_nodes(old.end) <= 1 {
							self.add_and_execute(&Action::OverwriteWire { oldstart: old.start, oldend: old.end, newstart: old.start, newend: new.start });
							runanother = true; continue 'check;
						}
					}
				} else {
					if old.touches(new.start) && old.start != new.start && old.end != new.start {
						self.checked_split(old.start, old.end, new.start);
					} else if old.touches(new.end) && old.start != new.end && old.end != new.end {
						self.checked_split(old.start, old.end, new.end);
					}
				}
			}
		}

		if insert {
			self.add_and_execute(&Action::AddWire { from: new.start, to: new.end });
		}
	}

	pub fn draw(&mut self, ui: &imgui::Ui, device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration, rpass: &mut RenderPass, queue: &mut Queue) {
		if ui.is_key_down(Key::LeftCtrl) {
			if ui.is_key_pressed(Key::Z) {
				self.undo();
			} else if ui.is_key_pressed(Key::Y) {
				self.redo();
			}
		}

		if ui.is_key_pressed(Key::F3) {
			self.inner.debug ^= true;
		}

		if ui.is_key_pressed(Key::F4) {
			self.benchmarking ^= true;
			if self.benchmarking {
				self.select(None);
				self.comps.clear();
				self.wires.wires.clear();
				self.nodes.node_lookup.clear();
				self.nodes.node_storage.clear();

				self.benchmark_cursor = Vec2::new(8.0, 0.0);

				self.inner.zoom = 0.1;
			}
		}

		if self.benchmarking {
			const KIND: CompKind = CompKind::XorGate;

			self.add_comp(
				KIND,
				Vec2::new(
					self.benchmark_cursor.x * self.benchmark_cursor.y.to_radians().cos(),
					self.benchmark_cursor.x * self.benchmark_cursor.y.to_radians().sin(),
				),
			);

			let circumference = 2.0 * self.benchmark_cursor.x * PI;
			// Átló
			let max_comp_size = KIND.hitbox().length() + 1.0;

			let step = 360.0 / (circumference / (max_comp_size + 1.0));

			if self.benchmark_cursor.y < (360.0 - step * 1.5) {
				self.benchmark_cursor.y += step;
			} else {
				self.benchmark_cursor.x += max_comp_size;
				self.benchmark_cursor.y = 0.0;
			}

			if (1.0 / ui.io().delta_time) < 40.0 {
				if self.benchmark_frame_counter == 5 {
					self.benchmarking = false;
				} else {
					self.benchmark_frame_counter += 1;
				}
			} else {
				self.benchmark_frame_counter = 0;
			}

			if self.comps.len() == 2200 {
				self.inner.zoom = 0.05
			}
		}

		let mut flag = false;

		if let Some(_menu1) = ui.begin_menu_bar() {
			if let Some(_menu3) = ui.begin_menu("File") {
				if ui.menu_item("Save") {
					flag = true;
					self.action = FileAction::Save;
				}

				if ui.menu_item("Load") {
					flag = true;
					self.action = FileAction::Load;
				}
			}
		}

		if ui.is_key_down(imgui::Key::LeftCtrl) && ui.is_key_pressed_no_repeat(imgui::Key::S) {
			flag = true;
			self.action = FileAction::Save;
		}

		if ui.is_key_down(imgui::Key::LeftCtrl) && ui.is_key_pressed_no_repeat(imgui::Key::O) {
			flag = true;
			self.action = FileAction::Load;
		}

		if flag {
			let action = self.action;
			self.dialog_thread.replace(
				thread::spawn(move || {
					let dialog =
						if action == FileAction::Load {
							rfd::FileDialog::new().pick_file()
						} else {
							rfd::FileDialog::new().save_file()
						};

					let str = dialog.unwrap().to_str().unwrap().to_string();
					str
				})
			);
		}

		let mut path = None;

		if self.action != FileAction::None {
			if self.dialog_thread.as_ref().unwrap().is_finished() {
				let turi = self.dialog_thread.take().unwrap();
				path.replace(turi.join().unwrap());
			}
		}

		match self.action {
			FileAction::Save => {
				if let Some(path) = path.take() {
					println!("Saving to {}", path);
					self.save(path.as_str());
					self.action = FileAction::None;
				}
			}

			FileAction::Load => {
				if let Some(path) = path.take() {
					println!("Loading from {}", path);
					let second = Self::load(path.as_str(), device, surface_desc);
					*self = second;
					self.action = FileAction::None;
				}
			}

			FileAction::None => {}
		}

		// Zoom kezelése
		self.inner.zoom = self.inner.zoom + ((ui.io().mouse_wheel * 1024.0).round() / 1024.0) / 20.0 * self.inner.zoom;

		// Pan
		if ui.io().mouse_down[imgui::MouseButton::Middle as usize] {
			self.inner.pan += Vec2::from(ui.io().mouse_delta) / self.inner.zoom;
		}

		self.deltas.push(ui.io().delta_time * 1000.0);
		if self.deltas.len() > self.deltas_datapoints {
			self.deltas.remove(0);
		}

		ui.text(format!("FPS: {:.2} ({:.2} ms)", 1.0 / ui.io().delta_time, ui.io().delta_time * 1000.0));
		ui.text(format!("zoom: {}", self.inner.zoom));
		ui.text(format!("pan: {:?}", self.inner.pan));
		let pos = self.inner.window_to_canvas(ui.io().mouse_pos.into());
		ui.text(format!("mouse ({}, {})", pos.x, pos.y));
		ui.text(format!("generation #{}", self.update_generation));
		ui.text(format!("# of components: {}", self.comps.len()));
		ui.text(format!("# of wires: {}", self.wires.wires.len()));
		ui.text(format!("# of nodes: {}", self.nodes.node_storage.len()));
		ui.text(format!("# of lines rendered: {}", self.renderer.as_ref().unwrap().linebuf_len()));
		ui.text("frame delta:");
		ui.plot_histogram("", self.deltas.as_slice())
			.graph_size(Vec2::new(200.0, 50.0))
			.scale_min(0.1)
			.scale_max(50.0)
			.build();

		// Jobboldali vágólap-debugger
		let size: Vec2 = ui.window_size().into();
		ui.window("History")
			.size(Vec2::new(300.0, 300.0), imgui::Condition::Always)
			.position(Vec2::new(size.x - 300.0, 0.0), imgui::Condition::Always)
			.no_decoration()
			.build(|| {
				for (idx, tp) in self.history.records.iter().enumerate() {
					ui.tree_node_config(format!("{}TP #{}", if self.history.cursor > 0 && idx == self.history.cursor - 1 { "->" } else { "" }, idx))
					.default_open(true)
					.build(|| {
						for action in &tp.actions {
							let str: &'static str = action.into();
							ui.bullet_text(format!("{}", str));
						}
					});
				}
			});

		let coord = self.inner.window_to_canvas(ui.io().mouse_pos.into());
		if self.nodes.count_nodes(coord) > 0 {
			ui.text(format!("{:?} is driven: {}", coord, check_driven(&self.nodes.node_lookup, &mut self.nodes.node_storage, &self.wires, &self.comps, coord, &mut self.update_generation, true)));
			self.update_generation += 1;
		}

		if let Some(_) = ui.begin_popup_context_window() {
			self.inner.record_mouse(&ui.io().mouse_pos.into());

			if let Some(_) = ui.begin_menu("Spawn") {
				if let Some(_) = ui.begin_menu("Logic Gate") {
					for asd in CompKind::iter() {
						if ui.menu_item(format!("{}", asd.get_message().unwrap())) {
							// self.add_comp(asd, self.inner.get_mouse());
							self.record_and_execute(&Action::AddComp { to: self.inner.get_mouse(), kind: asd });
							self.inner.forget_mouse();
						}
					}
				}
			}
		}

		// Node-ok gombjainak létrehozása (minden koordináta első Node-ja)
		// Először kell a Node-okat rajzolni az invisible_button Z-koordinátája miatt
		let mut newwires = Vec::new();
		// let mut deleteidx = None;
		
		for (_, node) in &self.nodes.node_lookup {
			let nodeidx = node[0];
			let node = &self.nodes.node_storage[nodeidx];

			let (tobeadded0, tobeadded1, deletion) = node.process(nodeidx, &mut self.inner, ui);

			if let Some(w) = tobeadded0 { newwires.push(w); }
			if let Some(w) = tobeadded1 { newwires.push(w); }

			if deletion {
				// assert_matches!(deleteidx, None);
				// deleteidx.replace(nodeidx);
			}
		}

		// if let Some(deleteidx) = deleteidx {
			
		// }

		if !newwires.is_empty() {
			self.start_record();
			for w in newwires { self.checked_add_wire(w.start, w.end); }
			self.end_record();
		}

		let r = self.renderer.as_mut().unwrap();

		// Rajzolás (Wire)
		r.regenerate_buffers(device, Vec2::new(surface_desc.width as f32, surface_desc.height as f32), &self.wires, &self.nodes.node_storage, &self.nodes.node_lookup, &self.comps, &self.inner, queue);

		// Rajzolás (Comp)
		r.render(rpass, &self.inner, [ surface_desc.width as f32, surface_desc.height as f32 ]);

		let keys: Vec<_> = self.comps.iter().map(|(idx, _)| idx).collect();
		'turip: for k in &keys {
			if self.comps.get_mut(*k).unwrap().process(&mut self.inner, ui, *k) {
				self.comps.get_mut(*k).unwrap().on_click();
				self.comps[*k].update(&mut self.update_generation, &self.nodes.node_lookup, &mut self.nodes.node_storage, &self.wires, &self.comps);
				self.update_generation += 1;
			}

			if ui.is_item_hovered() {
				if ui.is_mouse_clicked(MouseButton::Left) {
					// Ha a jelenleg hoverelt item is ki van már jelölve, akkor ne deszelektáljunk
					if !self.comps[*k].selected && !self.pasting_ongoing {
						if !ui.is_key_down(Key::LeftShift) { self.select(None); }
						self.select(Some(ElemIndex::Comp(*k)));
					}

					if !self.history.recording {
						self.start_record();
					}
				}

				if ui.is_mouse_released(MouseButton::Left) {
					if self.history.recording {
						self.end_record();
					}
				}
			}

			if let Some(newpos) = self.comps[*k].move_request.take() {
				let a1 = newpos;
				let a2 = a1 + self.comps[*k].kind.hitbox();

				for k2 in &keys {
					if k == k2 { continue; }

					let b1 = self.comps[*k2].pos;
					let b2 = b1 + self.comps[*k2].kind.hitbox();

					if a1.x < b2.x && a2.x > b1.x &&
					a1.y < b2.y && a2.y > b1.y {
						continue 'turip;
					}
				}

				if newpos - self.comps[*k].pos != Vec2::new(0.0, 0.0) {
					let move_by = newpos - self.comps[*k].pos;
					for k in 0..self.selection.len() {
						if let ElemIndex::Comp(k) = self.selection[k] {
							self.add_and_execute(&Action::MoveComp { from: self.comps[k].pos, to: self.comps[k].pos + move_by });
						} else if let ElemIndex::Wire(k) = self.selection[k] {
							self.add_and_execute(&Action::MoveWire { from: self.wires.wires[k].start, to: self.wires.wires[k].end, by: move_by });
						}
					}
				}
			}
		}

		let mut id = 0;
		let wkeys: Vec<_> = self.wires.wires.iter().map(|(idx, _)| idx).collect();
		for wi in &wkeys {
			// if !self.wires.wires.contains(*wi) {
			// 	continue;
			// }

			let wire = &self.wires.wires[*wi];
			wire.process(&self.inner, ui, Some(id));

			if ui.is_item_active() {
				if self.inner.grab_mouse_offset.is_none() {
					self.inner.grab_mouse_offset.replace((ElemIndex::Wire(*wi), self.inner.canvas_to_window(wire.start) - Vec2::from(ui.io().mouse_pos)));
				}
			} else {
				if let Some(turi) = self.inner.grab_mouse_offset {
					if let ElemIndex::Wire(id) = turi.0 && id == *wi {
						self.inner.grab_mouse_offset.take();
					}
				}
			}

			if ui.is_item_hovered() {
				if ui.is_mouse_clicked(MouseButton::Left) {
					// Ha a jelenleg hoverelt item is ki van már jelölve, akkor ne deszelektáljunk
					if !wire.selected {
						if ui.is_key_down(Key::LeftShift) {
							self.select(None);
							self.select(Some(ElemIndex::Wire(*wi)));
						} else {
							self.split(*wi, self.inner.window_to_canvas(ui.io().mouse_pos.into()));
						}
					}
				} else if ui.is_mouse_clicked(MouseButton::Middle) {
					self.selection.retain(|e| {
						if let ElemIndex::Wire(w) = e {
							if self.wires.wires[*w].start == wire.start && self.wires.wires[*w].end == wire.end {
								return false;
							}
						}

						return true;
					});

					self.record_and_execute(&Action::DeleteWire { from: wire.start, to: wire.end });
				}
			}

			id += 1;
		}

		// Egyenkénti kijelölés logika
		if ui.is_mouse_released(MouseButton::Left) && !self.pasting_ongoing {
			if !ui.is_any_item_hovered() {
				if !ui.is_key_down(Key::LeftShift) {
					self.select(None);
				}
			}

			if self.selection_ongoing {
				for k in &keys {
					let e1 = self.inner.window_to_canvas(ui.io().mouse_pos.into());
					let e = Vec2::new(self.selection_start.x.max(e1.x), self.selection_start.y.max(e1.y));
					let s = Vec2::new(self.selection_start.x.min(e1.x), self.selection_start.y.min(e1.y));
					if (self.comps[*k].pos.x >= s.x && self.comps[*k].pos.x <= e.x) && (self.comps[*k].pos.y >= s.y && self.comps[*k].pos.y <= e.y) {
						self.select(Some(ElemIndex::Comp(*k)));
					}
				}

				for wi in &wkeys {
					let e1 = self.inner.window_to_canvas(ui.io().mouse_pos.into());
					let e = Vec2::new(self.selection_start.x.max(e1.x), self.selection_start.y.max(e1.y));
					let s = Vec2::new(self.selection_start.x.min(e1.x), self.selection_start.y.min(e1.y));
					if (
					self.wires.wires[*wi].start.x >= s.x && self.wires.wires[*wi].start.x <= e.x) && (self.wires.wires[*wi].start.y >= s.y && self.wires.wires[*wi].start.y <= e.y &&
					self.wires.wires[*wi].end.x >= s.x && self.wires.wires[*wi].end.x <= e.x) && (self.wires.wires[*wi].end.y >= s.y && self.wires.wires[*wi].end.y <= e.y
					) {
						self.select(Some(ElemIndex::Wire(*wi)));
					}
				}

				self.selection_ongoing = false;
			}
		}

		// Jelölő téglalap logika
		if !self.pasting_ongoing {
			if !ui.is_any_item_hovered() {
				if ui.is_mouse_clicked(MouseButton::Left) {
					self.selection_start = self.inner.window_to_canvas(ui.io().mouse_pos.into());
					self.selection_ongoing = true;
				}
			}

			if self.selection_ongoing {
				ui.get_window_draw_list().add_rect(self.inner.canvas_to_window(self.selection_start), ui.io().mouse_pos, 0x06ffffff)
					.filled(true)
					.build();
			}
		}

		// Törlés logika
		if ui.is_key_pressed(Key::Delete) {
			self.start_record();
			for i in 0..self.selection.len() {
				if let ElemIndex::Comp(c) = self.selection[i] {
					self.add_and_execute(&Action::DeleteComp { at: self.comps[c].pos, kind: self.comps[c].kind });
				} else if let ElemIndex::Wire(w) = self.selection[i] {
					self.add_and_execute(&Action::DeleteWire { from: self.wires.wires[w].start, to: self.wires.wires[w].end });
				}
			}
			self.selection.clear();
			self.end_record();
		}

		// Vágólap logika
		if ui.is_key_down(Key::LeftCtrl) && ui.is_key_pressed_no_repeat(Key::C) {
			self.copy();
			self.pasting_ongoing = true;
		}

		if ui.is_key_pressed_no_repeat(Key::Escape) {
			self.pasting_ongoing = false;
		}

		if ui.is_key_down(Key::LeftCtrl) && ui.is_key_pressed_no_repeat(Key::V) {
			self.pasting_ongoing = true;
		}

		if self.pasting_ongoing {
			self.render_clipboard(ui);
		}

		if self.pasting_ongoing && ui.is_mouse_clicked(MouseButton::Left) {
			self.paste(ui);
		}
	}

	fn add_comp(&mut self, asd: CompKind, at: Vec2) {
		let c = Component::new(asd, at, self.inner.compid, &mut self.nodes);
		let idx = self.comps.insert(c);
		for n in &self.comps[idx].nodes {
			self.nodes.node_storage[*n].owner = Some(ElemIndex::Comp(idx));

			let ll = self.nodes.node_storage[*n].logic_lvl;
			let npos = self.nodes.node_storage[*n].pos;
			set_nodes(&self.nodes.node_lookup, &mut self.nodes.node_storage, &self.wires, &self.comps, npos, &mut self.update_generation, ll, true);
		}
		self.inner.compid += 1;
	}

	pub fn save(&self, file: &str) {
		let mut buf = Vec::new();
		self.serialize(&mut Serializer::new(&mut buf)).expect("Failed to serialize save file!");
		std::fs::write(file, buf).expect("Failed to write save file!");
	}

	pub fn load(file: &str, device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration) -> Self {
		let file = std::fs::File::open(file).expect("Failed to open save file!");
		let mut reader = std::io::BufReader::new(file);
		let mut deser = rmp_serde::Deserializer::new(&mut reader);

		let mut almost = Self::deserialize(&mut deser).expect("Failed to deserialize save file!");
		almost.inner.create_pipeline(device, surface_desc);
		almost.renderer.replace(CanvasRenderer::new(device, surface_desc));
		almost
	}

	fn select(&mut self, idx: Option<ElemIndex>) {
		if let Some(idx) = idx {
			self.selection.push(idx);
			match idx {
				ElemIndex::Wire(i) => self.wires.wires[i].selected = true,
				ElemIndex::Comp(i) => self.comps[i].selected = true,
			}
		} else {
			for (_, c) in &mut self.comps { c.selected = false; }
			for (_, w) in &mut self.wires.wires { w.selected = false; }
			self.selection.clear();
		}
	}

	pub fn copy(&mut self) {
		self.clipboard.clear();

		let mut offset = Vec2::new(0.0, 0.0);
		for s in &self.selection {
			if let ElemIndex::Comp(c) = s {
				if self.comps[*c].pos.x < offset.x { offset.x = self.comps[*c].pos.x; }
				if self.comps[*c].pos.y < offset.y { offset.y = self.comps[*c].pos.y; }
			} else if let ElemIndex::Wire(w) = s {
				if self.wires.wires[*w].start.x < offset.x { offset.x = self.wires.wires[*w].start.x; }
				if self.wires.wires[*w].start.y < offset.y { offset.y = self.wires.wires[*w].start.y; }

				if self.wires.wires[*w].end.x < offset.x { offset.x = self.wires.wires[*w].end.x; }
				if self.wires.wires[*w].end.y < offset.y { offset.y = self.wires.wires[*w].end.y; }
			}
		}

		for s in &self.selection {
			if let ElemIndex::Comp(c) = s {
				self.clipboard.push(
					ClipboardElement::Component {
						kind: self.comps[*c].kind.clone(),
						pos: self.comps[*c].pos - offset,
					}
				);
			} else if let ElemIndex::Wire(w) = s {
				self.clipboard.push(
					ClipboardElement::Wire {
						start: self.wires.wires[*w].start - offset,
						end: self.wires.wires[*w].end - offset
					}
				);
			}
		}
	}

	fn render_clipboard(&self, ui: &imgui::Ui) {
		let m = self.inner.window_to_canvas(ui.io().mouse_pos.into());
		for e in &self.clipboard {
			if let ClipboardElement::Component { kind, pos } = e {
				kind.draw(m + *pos, 0xffffffff, &self.inner, &ui.get_window_draw_list());
			} else if let ClipboardElement::Wire { start, end } = e {
				ui.get_window_draw_list().add_line(self.inner.canvas_to_window(m + *start), self.inner.canvas_to_window(m + *end), 0xffffffff)
					.thickness(config::WIRE_THICKNESS * self.inner.zoom)
					.build();
			}
		}
	}

	pub fn paste(&mut self, ui: &imgui::Ui) {
		let m = self.inner.window_to_canvas(ui.io().mouse_pos.into());
		self.record_and_execute(&Action::Paste { at: m, clipboard: self.clipboard.clone() });
	}
}

impl Debug for NodeHandler {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Storage:").unwrap();
		for s in &self.node_storage {
			writeln!(f, "{:?}", s.1).unwrap();
		}

		Ok(())
	}
}
