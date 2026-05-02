use std::{f32::consts::PI, fmt::Debug, thread::{self, JoinHandle}};

use glam::IVec2;
use imgui::{MouseButton, Key};
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use strum::{EnumMessage, IntoEnumIterator};
use wgpu::{Queue, RenderPass};

use crate::{canvas::{component::{CompKind, Component}, inner::CanvasInner, nodes::{NodeHandler, check_driven, set_nodes}, renderer::CanvasRenderer, wires::{Wire, WireKey, Wires}}, config};

mod component;
mod wires;
mod inner;
mod logic;
mod nodes;
mod renderer;

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

#[derive(Serialize, Deserialize)]
enum ClipboardElement {
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
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	pos: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Immediates {
	start: [f32; 2],
	step: [f32; 2],
	num_cols: u32,
	zoom: f32,
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
		};
		
		s.add_comp(CompKind::NandGate, Vec2::new(0.0, 5.0));

		s.add_comp(CompKind::OrGate, Vec2::new(0.0, -5.0));
		
		s.add_comp(CompKind::AndGate, Vec2::new(0.0, 0.0));

		s.wires.try_add(Wire { start: Vec2::new(0.0, 1.0), end: Vec2::new(-5.0, 1.0), startnode: None, endnode: None, selected: false }, &mut s.nodes, &s.comps, &mut s.update_generation);
		s.add_comp(CompKind::Input { state: false }, Vec2::new(-7.0, 0.0));

		s.wires.try_add(Wire { start: Vec2::new(0.0, 3.0), end: Vec2::new(-5.0, 3.0), startnode: None, endnode: None, selected: false }, &mut s.nodes, &s.comps, &mut s.update_generation);
		s.add_comp(CompKind::Input { state: false }, Vec2::new(-7.0, 2.0));

		s.wires.try_add(Wire { start: Vec2::new(4.0, 2.0), end: Vec2::new(8.0, 2.0), startnode: None, endnode: None, selected: false }, &mut s.nodes, &s.comps, &mut s.update_generation);

		s
	}

	pub fn draw(&mut self, ui: &imgui::Ui, device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration, rpass: &mut RenderPass, queue: &mut Queue) {
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
				self.benchmarking = false;
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
							self.add_comp(asd, self.inner.get_mouse());
							self.inner.forget_mouse();
						}
					}
				}
			}
		}

		// Node-ok gombjainak létrehozása (minden koordináta első Node-ja)
		// Először kell a Node-okat rajzolni az invisible_button Z-koordinátája miatt
		let mut newwires = Vec::new();
		for (_, node) in &self.nodes.node_lookup {
			let nodeidx = node[0];
			let node = &self.nodes.node_storage[nodeidx];

			let tobeadded = node.process(nodeidx, &mut self.inner, ui);
			if let Some(w) = tobeadded.0 { newwires.push(w); }
			if let Some(w) = tobeadded.1 { newwires.push(w); }
		}
		for w in newwires { self.wires.try_add(w, &mut self.nodes, &self.comps, &mut self.update_generation); }

		let r = self.renderer.as_mut().unwrap();

		// Rajzolás (Wire)
		r.regenerate_buffers(&self.wires, &self.nodes.node_storage, &self.nodes.node_lookup, &self.comps, &self.inner, queue);

		// Rajzolás (Comp)
		r.render(rpass, &self.inner, [ surface_desc.width as f32, surface_desc.height as f32 ]);

		let keys: Vec<_> = self.comps.iter().map(|(idx, _)| idx).collect();
		'turip: for k in &keys {
			if self.comps.get_mut(*k).unwrap().process(&mut self.inner, ui) {
				self.comps.get_mut(*k).unwrap().on_click();
				self.comps[*k].update(&mut self.update_generation, &self.nodes.node_lookup, &mut self.nodes.node_storage, &self.wires, &self.comps);
				self.update_generation += 1;
			}

			if ui.is_item_hovered() {
				if ui.is_mouse_clicked(MouseButton::Left) {
					// Ha a jelenleg hoverelt item is ki van már jelölve, akkor ne deszelektáljunk
					if !self.comps[*k].selected {
						if !ui.is_key_down(Key::LeftShift) { self.select(None); }
						self.select(Some(ElemIndex::Comp(*k)));
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
					for k in &self.selection {
						if let ElemIndex::Comp(k) = *k {
							let mut element = self.comps[k].clone();
							element.move_by(move_by, &mut self.nodes, k, &self.wires, &self.comps, &mut self.update_generation);
							self.comps[k] = element;
						} else if let ElemIndex::Wire(k) = *k {
							let mut element = self.wires.wires[k].clone();
							element.move_by(move_by, k, &mut self.nodes, &self.wires, &self.comps, &mut self.update_generation);
							self.wires.wires[k] = element;
						}
					}
				}
			}
		}

		let mut id = 0;
		let wkeys: Vec<_> = self.wires.wires.iter().map(|(idx, _)| idx).collect();
		for wi in &wkeys {
			self.wires.wires[*wi].process(&self.inner, ui, Some(id));

			if ui.is_item_hovered() {
				if ui.is_mouse_clicked(MouseButton::Left) {
					// Ha a jelenleg hoverelt item is ki van már jelölve, akkor ne deszelektáljunk
					if !self.wires.wires[*wi].selected {
						if !ui.is_key_down(Key::LeftShift) { self.select(None); }
						self.select(Some(ElemIndex::Wire(*wi)));
					}
				}
			}

			id += 1;
		}

		// Egyenkénti kijelölés logika
		if ui.is_mouse_released(MouseButton::Left) {
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
		{
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
		if ui.is_key_down(Key::Delete) {
			for e in &self.selection {
				if let ElemIndex::Comp(c) = e {
					self.comps[*c].remove_nodes(&mut self.nodes, *c, &self.wires, &self.comps, &mut self.update_generation);
					self.comps.remove(*c);
				} else if let ElemIndex::Wire(w) = e {
					self.wires.wires[*w].remove_nodes(&mut self.nodes, *w, &self.wires, &self.comps, &mut self.update_generation);
					self.wires.wires.remove(*w);
				}
			}
			self.selection.clear();
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

	fn copy(&mut self) {
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

	fn paste(&mut self, ui: &imgui::Ui) {
		let m = self.inner.window_to_canvas(ui.io().mouse_pos.into());
		for i in 0..self.clipboard.len() {
			if let ClipboardElement::Component { kind, pos } = &self.clipboard[i] {
				self.add_comp(kind.clone(), m + pos);
			} else if let ClipboardElement::Wire { start, end } = &self.clipboard[i] {
				self.wires.try_add(Wire { start: m + start, end: m + end, startnode: None, endnode: None, selected: false }, &mut self.nodes, &self.comps, &mut self.update_generation);
			}
		}
	}
}

impl Debug for NodeHandler {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// f.debug_struct("NodeHandler").field("node_storage", &self.node_storage).field("node_lookup", &self.node_lookup).finish()
		writeln!(f, "Storage:").unwrap();
		for s in &self.node_storage {
			writeln!(f, "{:?}", s.1).unwrap();
		}

		Ok(())
	}
}
