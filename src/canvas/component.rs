use serde::{Deserialize, Serialize};

use crate::{canvas::{CanvasInner, CompKey, CompStorage, NodeHandler, NodeOwner, Vec2, logic::LL, nodes::{Node, NodeKey, NodeLookup, NodeStorage}, set_nodes, wires::Wires}, config::{self, GRID_SPACING}};

#[allow(unused)]
#[derive(strum::EnumIter, strum::EnumMessage, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CompKind {
	#[strum(message = "AND Gate")]	AndGate,
	#[strum(message = "OR Gate")]	OrGate,
	#[strum(message = "NAND Gate")]	NandGate,
	#[strum(message = "XOR Gate")]	XorGate,
	#[strum(message = "Input")]		Input { state: bool },
}

impl CompKind {
	pub const fn hitbox(&self) -> Vec2 {
		match self {
			CompKind::AndGate | CompKind::OrGate => Vec2::new(4.0, 4.0),
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
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
#[allow(unused)]
pub struct Component {
	pub kind: CompKind,
	pub pos: Vec2,
	pub nodes: Vec<NodeKey>,
	pub id: u64,
	pub move_request: Option<Vec2>,
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
		}
	}

	pub fn draw(&mut self, canvas: &mut CanvasInner, ui: &imgui::Ui) -> bool {
		let draw_list = ui.get_window_draw_list();

		// A görbe valamiért lejjebb van mint kéne, ez szemre belövi
		let curve_y_offset: f32 = -0.03 / canvas.zoom;

		let mut clicked = false;

		match self.kind {
			CompKind::AndGate => {
				draw_list.add_bezier_curve(
					canvas.canvas_to_window(self.pos + Vec2::new(2.00, 4.0 - curve_y_offset)),
					canvas.canvas_to_window(self.pos + Vec2::new(4.67, 4.0 - curve_y_offset)),
					canvas.canvas_to_window(self.pos + Vec2::new(4.67, 0.0 - curve_y_offset)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.00, 0.0 - curve_y_offset)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 4.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 4.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 0.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 0.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 4.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 0.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();
			},
			CompKind::OrGate => {
				// Hogy a Node-ok rácsra illeszkedjenek
				const OFFSET: Vec2 = Vec2::new(0.231, 0.0);

				// Elülső görbe
				{
					draw_list.add_bezier_curve(
						canvas.canvas_to_window(self.pos + Vec2::new(2.00, 4.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(4.50, 4.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(5.00, 2.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(5.00, 2.0 - curve_y_offset)),
						0xffffffff
					)
						.thickness(config::COMPONENT_THICKNESS)
						.build();

					draw_list.add_bezier_curve(
						canvas.canvas_to_window(self.pos + Vec2::new(2.00, 0.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(4.50, 0.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(5.00, 2.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(5.00, 2.0 - curve_y_offset)),
						0xffffffff
					)
						.thickness(config::COMPONENT_THICKNESS)
						.build();
				}

				// Hátsó görbe
				{
					draw_list.add_bezier_curve(
						canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(0.00, 0.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(1.30, 1.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(1.30, 3.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(0.00, 4.0 - curve_y_offset)),
						0xffffffff
					)
						.thickness(config::COMPONENT_THICKNESS)
						.build();
				}

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(0.0, 4.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 4.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(0.0, 0.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 0.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();
			},
			CompKind::NandGate => {
				draw_list.add_bezier_curve(
					canvas.canvas_to_window(self.pos + Vec2::new(2.00, 4.0 - curve_y_offset)),
					canvas.canvas_to_window(self.pos + Vec2::new(4.67, 4.0 - curve_y_offset)),
					canvas.canvas_to_window(self.pos + Vec2::new(4.67, 0.0 - curve_y_offset)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.00, 0.0 - curve_y_offset)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 4.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 4.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 0.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 0.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 4.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 0.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_circle(
					canvas.canvas_to_window(self.pos + Vec2::new(4.5, 2.0)),
					canvas.zoom * (GRID_SPACING / 2.0),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();
			},
			CompKind::XorGate => {
				// Hogy a Node-ok rácsra illeszkedjenek
				const OFFSET: Vec2 = Vec2::new(0.231, 0.0);

				// Elülső görbe
				{
					draw_list.add_bezier_curve(
						canvas.canvas_to_window(self.pos + Vec2::new(2.00, 4.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(4.50, 4.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(5.00, 2.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(5.00, 2.0 - curve_y_offset)),
						0xffffffff
					)
						.thickness(config::COMPONENT_THICKNESS)
						.build();

					draw_list.add_bezier_curve(
						canvas.canvas_to_window(self.pos + Vec2::new(2.00, 0.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(4.50, 0.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(5.00, 2.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + Vec2::new(5.00, 2.0 - curve_y_offset)),
						0xffffffff
					)
						.thickness(config::COMPONENT_THICKNESS)
						.build();
				}

				// Hátsó görbe #1
				let so = Vec2::new(0.7, 0.0);
				{
					draw_list.add_bezier_curve(
						canvas.canvas_to_window(self.pos + so + OFFSET + Vec2::new(0.00, 0.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + so + OFFSET + Vec2::new(1.30, 1.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + so + OFFSET + Vec2::new(1.30, 3.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + so + OFFSET + Vec2::new(0.00, 4.0 - curve_y_offset)),
						0xffffffff
					)
						.thickness(config::COMPONENT_THICKNESS)
						.build();
				}

				// Hátsó görbe #2
				{
					draw_list.add_bezier_curve(
						canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(0.00, 0.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(1.30, 1.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(1.30, 3.0 - curve_y_offset)),
						canvas.canvas_to_window(self.pos + OFFSET + Vec2::new(0.00, 4.0 - curve_y_offset)),
						0xffffffff
					)
						.thickness(config::COMPONENT_THICKNESS)
						.build();
				}

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + so + OFFSET + Vec2::new(0.0, 4.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 4.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + so + OFFSET + Vec2::new(0.0, 0.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 0.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();
			},
			CompKind::Input { state: _ } => {
				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 0.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 0.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 0.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 2.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 2.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 2.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 2.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 0.0)),
					0xffffffff
				)
					.thickness(config::COMPONENT_THICKNESS)
					.build();
			}
		}

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

	pub fn move_to(&mut self, to: Vec2, nodes: &mut NodeHandler, cidx: CompKey, wires: &Wires, comps: &CompStorage, generation: &mut u32) {
		for nidx in &self.nodes {
			nodes.move_node(*nidx, to - self.pos, NodeOwner::Comp(cidx), wires, comps, generation);
			*generation += 1;
		}

		self.pos = to;
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
}
