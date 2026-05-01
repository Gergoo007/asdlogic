use glam::vec2;
use imgui::{DrawListMut};
use serde::{Deserialize, Serialize};

use crate::{canvas::{CanvasInner, CompKey, CompStorage, ElemIndex, NodeHandler, Vec2, logic::LL, nodes::{Node, NodeKey, NodeLookup, NodeStorage}, set_nodes, wires::Wires}, config::{self, GRID_SPACING}};

#[allow(unused)]
#[derive(strum::EnumIter, strum::EnumMessage, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CompKind {
	#[strum(message = "AND Gate")]	AndGate,
	#[strum(message = "OR Gate")]	OrGate,
	#[strum(message = "NAND Gate")]	NandGate,
	#[strum(message = "XOR Gate")]	XorGate,
	#[strum(message = "Input")]		Input { state: bool },
}

pub enum ShapeElement {
	Line(Vec2, Vec2),
	Bezier(Vec2, Vec2, Vec2, Vec2),
	Circle(Vec2),
	Nop,
}

const fn ca(a: Vec2, b: Vec2) -> Vec2 { Vec2::new(a.x + b.x, a.y + b.y) }
const fn ca2(a: Vec2, b: Vec2, c: Vec2) -> Vec2 { Vec2::new(a.x + b.x + c.x, a.y + b.y + c.y) }

impl CompKind {
	pub const fn shape(&self) -> [ShapeElement; 6] {
		// Hogy a Node-ok rácsra illeszkedjenek
		const OR_OFFSET: Vec2 = Vec2::new(0.231, 0.0);

		const SHIELD_OFFSET: Vec2 = Vec2::new(0.7, 0.0);

		match self {
			CompKind::AndGate => [
				ShapeElement::Bezier(vec2(2.00, 4.0), vec2(4.67, 4.0), vec2(4.67, 0.0), vec2(2.00, 0.0)),
				ShapeElement::Line(vec2(0.0, 4.0), vec2(2.0, 4.0)),
				ShapeElement::Line(vec2(0.0, 0.0), vec2(2.0, 0.0)),
				ShapeElement::Line(vec2(0.0, 4.0), vec2(0.0, 0.0)),
				ShapeElement::Nop, ShapeElement::Nop,
			],
			CompKind::OrGate => [
				ShapeElement::Bezier(vec2(2.00, 4.0), vec2(4.50, 4.0), vec2(5.00, 2.0), vec2(5.00, 2.0)),
				ShapeElement::Bezier(vec2(2.00, 0.0), vec2(4.50, 0.0), vec2(5.00, 2.0), vec2(5.00, 2.0)),
				ShapeElement::Bezier(ca(vec2(0.00, 0.0), OR_OFFSET), ca(vec2(1.30, 1.0), OR_OFFSET), ca(vec2(1.30, 3.0), OR_OFFSET), ca(vec2(0.00, 4.0), OR_OFFSET)),
				ShapeElement::Line(ca(vec2(0.0, 4.0), OR_OFFSET), vec2(2.0, 4.0)),
				ShapeElement::Line(ca(vec2(0.0, 0.0), OR_OFFSET), vec2(2.0, 0.0)),
				ShapeElement::Nop,
			],
			CompKind::NandGate => [
				ShapeElement::Bezier(vec2(2.00, 4.0), vec2(4.67, 4.0), vec2(4.67, 0.0), vec2(2.00, 0.0)),
				ShapeElement::Line(vec2(0.0, 4.0), vec2(2.0, 4.0)),
				ShapeElement::Line(vec2(0.0, 0.0), vec2(2.0, 0.0)),
				ShapeElement::Line(vec2(0.0, 4.0), vec2(0.0, 0.0)),
				ShapeElement::Circle(vec2(4.5, 2.0)),
				ShapeElement::Nop,
			],
			CompKind::XorGate => [
				ShapeElement::Bezier(vec2(2.00, 4.0), vec2(4.50, 4.0), vec2(5.00, 2.0), vec2(5.00, 2.0)),
				ShapeElement::Bezier(vec2(2.00, 0.0), vec2(4.50, 0.0), vec2(5.00, 2.0), vec2(5.00, 2.0)),
				
				ShapeElement::Bezier(ca2(vec2(0.00, 0.0), OR_OFFSET, SHIELD_OFFSET), ca2(vec2(1.30, 1.0), OR_OFFSET, SHIELD_OFFSET), ca2(vec2(1.30, 3.0), OR_OFFSET, SHIELD_OFFSET), ca2(vec2(0.00, 4.0), OR_OFFSET, SHIELD_OFFSET)),
				ShapeElement::Bezier(ca(vec2(0.00, 0.0), OR_OFFSET), ca(vec2(1.30, 1.0), OR_OFFSET), ca(vec2(1.30, 3.0), OR_OFFSET), ca(vec2(0.00, 4.0), OR_OFFSET)),

				ShapeElement::Line(ca2(vec2(0.0, 4.0), OR_OFFSET, SHIELD_OFFSET), vec2(2.0, 4.0)),
				ShapeElement::Line(ca2(vec2(0.0, 0.0), OR_OFFSET, SHIELD_OFFSET), vec2(2.0, 0.0)),
			],
			CompKind::Input { state: _ } => [
				ShapeElement::Line(vec2(0.0, 0.0), vec2(2.0, 0.0)),
				ShapeElement::Line(vec2(2.0, 0.0), vec2(2.0, 2.0)),
				ShapeElement::Line(vec2(2.0, 2.0), vec2(0.0, 2.0)),
				ShapeElement::Line(vec2(0.0, 2.0), vec2(0.0, 0.0)),
				ShapeElement::Nop, ShapeElement::Nop,
			],
		}
	}

	pub const fn hitbox(&self) -> Vec2 {
		match self {
			CompKind::AndGate => Vec2::new(4.0, 4.0),
			CompKind::OrGate => Vec2::new(5.0, 4.0),
			CompKind::NandGate => Vec2::new(5.0, 4.0),
			CompKind::XorGate => Vec2::new(5.0, 4.0),
			CompKind::Input { state: _ } => Vec2::new(2.0, 2.0),
		}
	}

	// FONTOS: az outputok jönnek először
	// A tuple második eleme a kimenetek darabszáma
	pub fn nodes(&self) -> (Vec<Node>, usize) {
		match self {
			CompKind::AndGate => (
				vec![
					Node { pos: Vec2::new(4.0, 2.0), owner: None, logic_lvl: LL::default(), generation: 0, output: true, },
					Node { pos: Vec2::new(0.0, 1.0), owner: None, logic_lvl: LL::default(), generation: 0, output: false, },
					Node { pos: Vec2::new(0.0, 3.0), owner: None, logic_lvl: LL::default(), generation: 0, output: false, },
				],
				1
			),
			CompKind::OrGate => (
				vec![
					Node { pos: Vec2::new(5.0, 2.0), owner: None, logic_lvl: LL::default(), generation: 0, output: true, },
					Node { pos: Vec2::new(1.0, 1.0), owner: None, logic_lvl: LL::default(), generation: 0, output: false, },
					Node { pos: Vec2::new(1.0, 3.0), owner: None, logic_lvl: LL::default(), generation: 0, output: false, },
				],
				1
			),
			CompKind::NandGate => (
				vec![
					Node { pos: Vec2::new(5.0, 2.0), owner: None, logic_lvl: LL::default(), generation: 0, output: true, },
					Node { pos: Vec2::new(0.0, 1.0), owner: None, logic_lvl: LL::default(), generation: 0, output: false, },
					Node { pos: Vec2::new(0.0, 3.0), owner: None, logic_lvl: LL::default(), generation: 0, output: false, },
				],
				1
			),
			CompKind::XorGate => (
				vec![
					Node { pos: Vec2::new(5.0, 2.0), owner: None, logic_lvl: LL::default(), generation: 0, output: true, },
					Node { pos: Vec2::new(1.0, 1.0), owner: None, logic_lvl: LL::default(), generation: 0, output: false, },
					Node { pos: Vec2::new(1.0, 3.0), owner: None, logic_lvl: LL::default(), generation: 0, output: false, },
				],
				1
			),
			CompKind::Input { state: _ } => (
				vec![
					Node { pos: Vec2::new(2.0, 1.0), owner: None, logic_lvl: LL::L, generation: 0, output: true, },
				],
				1,
			),
		}
	}

	pub fn draw(&self, pos: Vec2, color: u32, canvas: &CanvasInner, draw_list: &DrawListMut) {
		let th = config::COMPONENT_THICKNESS * canvas.zoom;
		for s in &self.shape() {
			match *s {
				ShapeElement::Line(vec2, vec3) => draw_list.add_line(
					canvas.canvas_to_window(pos + vec2),
					canvas.canvas_to_window(pos + vec3),
					color
				).thickness(th).build(),
				ShapeElement::Bezier(vec2, vec3, vec4, vec5) => draw_list.add_bezier_curve(
					canvas.canvas_to_window(pos + vec2),
					canvas.canvas_to_window(pos + vec3),
					canvas.canvas_to_window(pos + vec4),
					canvas.canvas_to_window(pos + vec5),
					color
				).thickness(th).build(),
				ShapeElement::Circle(vec2) => draw_list.add_circle(
					canvas.canvas_to_window(pos + vec2),
					canvas.zoom * (GRID_SPACING / 2.0),
					color
				)
					.thickness(th)
					.build(),
				ShapeElement::Nop => {},
			}
		}
	}
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
#[allow(unused)]
pub struct Component {
	pub kind: CompKind,
	pub pos: Vec2,
	pub nodes: Vec<NodeKey>,
	pub id: u64,
	pub move_request: Option<Vec2>,
	pub selected: bool,
}

impl Component {
	pub fn new(kind: CompKind, pos: Vec2, id: u64, nodes: &mut NodeHandler) -> Self {
		let nodes: Vec<NodeKey> = kind.nodes().0.iter().map(|v| { let mut copy = (*v).clone(); copy.pos += pos; nodes.add_node(copy) }).collect();

		Self {
			kind,
			pos,
			nodes,
			id,
			move_request: None,
			selected: false,
		}
	}

	pub fn process(&mut self, canvas: &mut CanvasInner, ui: &imgui::Ui) -> bool {
		let mut clicked = false;

		ui.set_cursor_pos(canvas.canvas_to_window(self.pos));

		ui.invisible_button(format!("comp{}", self.id), canvas.canvas_to_window_size(self.pos, self.kind.hitbox()));
		if ui.is_item_hovered() && ui.is_key_pressed_no_repeat(imgui::Key::E) {
			clicked = true;
		}

		if ui.is_item_active() {
			if canvas.grab_mouse_offset.is_none() {
				canvas.grab_mouse_offset.replace((self.id, canvas.canvas_to_window(self.pos) - Vec2::from(ui.io().mouse_pos)));
			}
			self.move_request.replace(canvas.window_to_canvas(canvas.grab_mouse_offset.unwrap().1 + Vec2::from(ui.io().mouse_pos)));
		} else {
			if let Some(turi) = canvas.grab_mouse_offset {
				if turi.0 == self.id {
					canvas.grab_mouse_offset.take();
				}
			}
		}

		clicked
	}

	pub fn move_by(&mut self, by: Vec2, nodes: &mut NodeHandler, cidx: CompKey, wires: &Wires, comps: &CompStorage, generation: &mut u32) {
		for nidx in &self.nodes {
			nodes.move_node(*nidx, by, ElemIndex::Comp(cidx), wires, comps, generation);
			*generation += 1;
		}

		self.pos += by;
	}

	pub fn on_click(&mut self) {
		match &mut self.kind {
			CompKind::Input { state } => {
				*state = !*state;
			},
			_ => {}
		}
	}

	pub fn update(&self, generation: &mut u32, node_lookup: &NodeLookup, node_storage: &mut NodeStorage, wires: &Wires, comps: &CompStorage) {
		// Kimenet kiszámítása a bemeneteknek megfelelően
		match self.kind {
			CompKind::AndGate => {
				let mut value = node_storage[self.nodes[1]].logic_lvl;
				for n in self.nodes.iter().skip(2) {
					value &= node_storage[*n].logic_lvl;
				}
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, value, false);
			},
			CompKind::OrGate => {
				let mut value = node_storage[self.nodes[1]].logic_lvl;
				for n in self.nodes.iter().skip(2) {
					value |= node_storage[*n].logic_lvl;
				}
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, value, false);
			},
			CompKind::NandGate => {
				let mut value = node_storage[self.nodes[1]].logic_lvl;
				for n in self.nodes.iter().skip(2) {
					value &= node_storage[*n].logic_lvl;
				}
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, !value, false);
			},
			CompKind::XorGate => {
				let mut value = node_storage[self.nodes[1]].logic_lvl;
				for n in self.nodes.iter().skip(2) {
					value ^= node_storage[*n].logic_lvl;
				}
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, value, false);
			},
			CompKind::Input { state } => {
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, state.into(), false);
			},
		}
	}

	pub fn remove_nodes(&self, nodes: &mut NodeHandler, cidx: CompKey, wires: &Wires, comps: &CompStorage, generation: &mut u32) {
		for n in &self.nodes {
			nodes.remove_node(nodes.node_storage[*n].pos, ElemIndex::Comp(cidx), wires, comps, generation, true);
		}
	}
}
