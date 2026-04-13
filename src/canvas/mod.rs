use std::{collections::HashMap, fmt::{Debug}};

use glam::IVec2;
use strum::{EnumMessage, IntoEnumIterator};

use crate::{canvas::{component::{CompKind, Component, Node}, inner::CanvasInner, logic::LL, wires::{Wire, Wires}}, config};

mod component;
mod wires;
mod inner;
mod logic;

pub type Vec2 = glam::Vec2;

pub type NodeStorage = generational_arena::Arena<Node>;
pub type NodeKey = generational_arena::Index;
pub type NodeLookup = HashMap<IVec2, Vec<NodeKey>>;

pub type CompStorage = generational_arena::Arena<Component>;
pub type CompKey = generational_arena::Index;

pub type WireStorage = generational_arena::Arena<Wire>;
pub type WireKey = generational_arena::Index;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum NodeOwner {
	Wire(WireKey),
	Comp(CompKey),
}

fn vec2int(float: Vec2) -> IVec2 {
	IVec2 { x: float.x as i32, y: float.y as i32 }
}

pub struct Canvas {
	pub inner: CanvasInner,
	pub comps: CompStorage,
	pub wires: Wires,
	pub nodes: NodeHandler,
}

pub struct NodeHandler {
	pub node_storage: NodeStorage,
	pub node_lookup: NodeLookup,
}

pub fn set_nodes(node_lookup: &NodeLookup, node_storage: &mut NodeStorage, wires: &Wires, comps: &CompStorage, at: Vec2, generation: u32, ll: LL) {
	for node in &node_lookup[&vec2int(at)] {
		let k = *node;
		let node = &mut node_storage[k];

		if node.generation == generation {
			if node.logic_lvl != ll {
				eprintln!("Short circuit!");
			}
			return;
		}

		node.logic_lvl = ll;
		node.generation = generation;

		if let Some(NodeOwner::Wire(w)) = node.owner {
			// Ha vezeték, akkor a másik node-ot is be kell állítani
			if k == wires.wires[w].startnode.unwrap() {
				set_nodes(node_lookup, node_storage, wires, comps, wires.wires[w].end, generation, ll);
			} else if k == wires.wires[w].endnode.unwrap() {
				set_nodes(node_lookup, node_storage, wires, comps, wires.wires[w].start, generation, ll);
			} else {
				unreachable!("he????");
			}
		} else if let Some(NodeOwner::Comp(c)) = node.owner {
			// Ha pedig komponens, akkor update-elni kell azt
			comps[c].update(generation, node_lookup, node_storage, wires, comps);
		}
	}
}

impl NodeHandler {
	pub fn new() -> Self {
		Self {
			node_storage: NodeStorage::new(),
			node_lookup: NodeLookup::new(),
		}
	}

	pub fn add_node(&mut self, n: Node) -> NodeKey {
		let pos = n.pos;
		let idx = self.node_storage.insert(n);
		self.node_lookup.entry(vec2int(pos)).and_modify(|v| { v.push(idx); }).or_insert(vec![idx]);
		idx
	}

	pub fn count_nodes(&self, at: Vec2) -> usize {
		self.node_lookup[&vec2int(at)].len()
	}

	pub fn remove_node(&mut self, at: Vec2, owner: NodeOwner) -> Node {
		let vals = self.node_lookup.get_mut(&vec2int(at)).unwrap();
		let mut node = None;

		// Az extract_if eltávolítja az adott NodeKey-t a HashMap vektorából,
		// a closure-ön belül pedig a NodeStorage Arénából is kiszedem
		
		vals.retain_mut(|e| {
			let remove = *self.node_storage[*e].owner.as_ref().unwrap() == owner;
			if remove {
				node = self.node_storage.remove(*e);
			}
			!remove
		});

		// Ha ez volt az ujtolsó Node ezen a koordinátán, akkor
		// fel is szabadítom ezt a bejegyzést a HashMap-ből
		if vals.len() == 0 {
			self.node_lookup.remove(&vec2int(at));
		}

		node.unwrap()
	}

	pub fn move_node(&mut self, nidx: NodeKey, by: Vec2, owner: NodeOwner) {
		let oldpos = self.node_storage[nidx].pos;
		let newpos = oldpos + by;
		let k = vec2int(oldpos);

		// Koordináták frissítése a HashMap-ben:
		// 1.1. ki kell venni a releváns NodeKey-eket az adott koordinátából
		let mut iter = self.node_lookup.get_mut(&k).unwrap().extract_if(.., |v| {
			self.node_storage[*v].owner == Some(owner)
		});
		let node = iter.next().unwrap();

		assert!(iter.next().is_none());

		drop(iter);

		// 1.2. ha nem maradt másik Node akkor a Vektor felszabadításra kerül
		if self.node_lookup.get_mut(&k).unwrap().len() == 0 {
			self.node_lookup.remove(&k);
		}

		// 2. vissza kell tenni az új koordinátára a kivett értékeket
		self.node_lookup.entry(vec2int(newpos)).and_modify(|nodekeys| {
			nodekeys.push(node);
		}).or_insert(vec![ node ]);

		// Koordináták frissítése az Arénában
		self.node_storage[nidx].pos += by;
	}
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
		};

		s.add_comp(CompKind::AndGate, Vec2::new(0.0, 0.0));

		s.wires.try_add(Wire { start: Vec2::new(0.0, 1.0), end: Vec2::new(-5.0, 1.0), startnode: None, endnode: None }, &mut s.nodes);
		s.add_comp(CompKind::Input { state: false }, Vec2::new(-7.0, 0.0));

		s.wires.try_add(Wire { start: Vec2::new(0.0, 3.0), end: Vec2::new(-5.0, 3.0), startnode: None, endnode: None }, &mut s.nodes);
		s.add_comp(CompKind::Input { state: false }, Vec2::new(-7.0, 2.0));

		s.wires.try_add(Wire { start: Vec2::new(4.0, 2.0), end: Vec2::new(8.0, 2.0), startnode: None, endnode: None }, &mut s.nodes);

		s
	}

	pub fn draw(&mut self, ui: &imgui::Ui) {
		let io = ui.io();

		// Zoom kezelése
		self.inner.zoom = self.inner.zoom + ((io.mouse_wheel * 1024.0).round() / 1024.0) / 20.0 * self.inner.zoom;

		// Pan
		if io.mouse_down[imgui::MouseButton::Middle as usize] {
			self.inner.pan += Vec2::from(io.mouse_delta) / self.inner.zoom;
		}

		ui.text(format!("FPS: {:.2} ({:.2} ms)", 1.0 / io.delta_time, io.delta_time));
		ui.text(format!("zoom: {}", self.inner.zoom));
		let pos = self.inner.window_to_canvas(ui.io().mouse_pos.into());
		ui.text(format!("mouse ({}, {})", pos.x, pos.y));

		if let Some(_) = ui.begin_popup_context_window() {
			self.inner.record_mouse(&io.mouse_pos.into());

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

		let keys: Vec<_> = self.comps.iter().map(|kv| kv.0).collect();
		for i in keys {
			if self.comps.get_mut(i).unwrap().draw(&mut self.inner, ui) {
				self.comps.get_mut(i).unwrap().on_click();
				self.comps[i].update(self.inner.update_generation, &self.nodes.node_lookup, &mut self.nodes.node_storage, &self.wires, &self.comps);
				self.inner.update_generation += 1;
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

				self.comps[*k].move_to(newpos, &mut self.nodes, *k);
			}
		}

		// Node-ok megrajzolása (minden koordináta első Node-ja)
		let mut newwires = Vec::new();
		for (_, node) in &self.nodes.node_lookup {
			let nodeidx = node[0];
			let node = &self.nodes.node_storage[nodeidx];

			let tobeadded = node.draw(nodeidx, &mut self.inner, ui);
			if let Some(w) = tobeadded.0 { newwires.push(w); }
			if let Some(w) = tobeadded.1 { newwires.push(w); }
		}
		for w in newwires { self.wires.try_add(w, &mut self.nodes); }

		// Debug: Node-ok rajzolása
		for (_, n) in &self.nodes.node_storage {
			let draw_list = ui.get_window_draw_list();

			draw_list.add_circle(self.inner.canvas_to_window(n.pos), config::NODE_RADIUS, n.logic_lvl.to_color())
				.filled(true)
				.build();
		}
	}

	fn add_comp(&mut self, asd: CompKind, at: Vec2) {
		let c = Component::new(asd, at, self.inner.compid, &mut self.nodes);
		let idx = self.comps.insert(c);
		for n in &mut self.comps[idx].nodes {
			self.nodes.node_storage[*n].owner.replace(NodeOwner::Comp(idx));
		}
		self.inner.compid += 1;
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
