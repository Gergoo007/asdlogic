use std::{fmt::{Debug, format}, thread::{self, JoinHandle}};

use glam::IVec2;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use strum::{EnumMessage, IntoEnumIterator};

use crate::{canvas::{component::{CompKind, Component}, inner::CanvasInner, nodes::{NodeHandler, check_driven, set_nodes}, wires::{Wire, WireKey, Wires}}, config};

mod component;
mod wires;
mod inner;
mod logic;
mod nodes;

pub type Vec2 = glam::Vec2;

pub type CompStorage = typed_generational_arena::StandardArena<Component>;
pub type CompKey = typed_generational_arena::StandardIndex<Component>;

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum NodeOwner {
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
		};
		
		s.add_comp(CompKind::OrGate, Vec2::new(0.0, -5.0));
		
		s.add_comp(CompKind::AndGate, Vec2::new(0.0, 0.0));

		s.wires.try_add(Wire { start: Vec2::new(0.0, 1.0), end: Vec2::new(-5.0, 1.0), startnode: None, endnode: None }, &mut s.nodes);
		s.add_comp(CompKind::Input { state: false }, Vec2::new(-7.0, 0.0));

		s.wires.try_add(Wire { start: Vec2::new(0.0, 3.0), end: Vec2::new(-5.0, 3.0), startnode: None, endnode: None }, &mut s.nodes);
		s.add_comp(CompKind::Input { state: false }, Vec2::new(-7.0, 2.0));

		s.wires.try_add(Wire { start: Vec2::new(4.0, 2.0), end: Vec2::new(8.0, 2.0), startnode: None, endnode: None }, &mut s.nodes);

		s
	}

	pub fn draw(&mut self, ui: &imgui::Ui, device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration) {
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

		ui.text(format!("FPS: {:.2} ({:.2} ms)", 1.0 / ui.io().delta_time, ui.io().delta_time));
		ui.text(format!("zoom: {}", self.inner.zoom));
		let pos = self.inner.window_to_canvas(ui.io().mouse_pos.into());
		ui.text(format!("mouse ({}, {})", pos.x, pos.y));
		ui.text(format!("generation #{}", self.update_generation));

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

		// Node-ok megrajzolása (minden koordináta első Node-ja)
		// Először kell a Node-okat rajzolni az invisible_button Z-koordinátája miatt
		let mut newwires = Vec::new();
		for (_, node) in &self.nodes.node_lookup {
			let nodeidx = node[0];
			let node = &self.nodes.node_storage[nodeidx];

			let tobeadded = node.draw(nodeidx, &mut self.inner, ui);
			if let Some(w) = tobeadded.0 { newwires.push(w); }
			if let Some(w) = tobeadded.1 { newwires.push(w); }
		}
		for w in newwires { self.wires.try_add(w, &mut self.nodes); }

		let keys: Vec<_> = self.comps.iter().map(|kv| kv.0).collect();
		for i in keys {
			if self.comps.get_mut(i).unwrap().draw(&mut self.inner, ui) {
				self.comps.get_mut(i).unwrap().on_click();
				self.comps[i].update(&mut self.update_generation, &self.nodes.node_lookup, &mut self.nodes.node_storage, &self.wires, &self.comps);
				self.update_generation += 1;
			}
		}

		self.wires.draw(&mut self.inner, ui, &self.nodes);

		let keys: Vec<_> = self.comps.iter().map(|(idx, _)| idx).collect();

		'turip: for k in &keys {
			let request = self.comps[*k].move_request.take();

			if let Some(newpos) = request {
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

				let mut element = self.comps[*k].clone();
				element.move_to(newpos, &mut self.nodes, *k, &self.wires, &self.comps, &mut self.update_generation);
				self.comps[*k] = element;
			}
		}

		// Debug
		let draw_list = ui.get_window_draw_list();
		for (_, n) in &self.nodes.node_storage {
			draw_list.add_circle(self.inner.canvas_to_window(n.pos), config::NODE_RADIUS, n.logic_lvl.to_color())
				.filled(true)
				.build();
		}
	}

	fn add_comp(&mut self, asd: CompKind, at: Vec2) {
		let c = Component::new(asd, at, self.inner.compid, &mut self.nodes);
		let idx = self.comps.insert(c);
		for n in &self.comps[idx].nodes {
			self.nodes.node_storage[*n].owner.replace(NodeOwner::Comp(idx));

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
