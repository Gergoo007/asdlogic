use crate::{canvas::{CanvasInner, CompKey, CompStorage, NodeHandler, NodeKey, NodeLookup, NodeOwner, NodeStorage, Vec2, logic::LL, set_nodes, wires::{Wire, Wires}}, config};

#[derive(PartialEq, Debug, Clone)]
pub struct Node {
	pub pos: Vec2,
	pub owner: Option<NodeOwner>,
	pub logic_lvl: LL,
	pub generation: u32,
}

impl Node {
	pub fn draw(&self, nodeid: NodeKey, inner: &mut CanvasInner, ui: &imgui::Ui)
	-> (Option<Wire>, Option<Wire>) {
		let draw_list = ui.get_window_draw_list();
		draw_list.add_circle(inner.canvas_to_window(self.pos), config::NODE_RADIUS, 0xffffffff).filled(true).build();

		let offset = config::NODE_HITBOX / 2.0;

		ui.set_cursor_pos(inner.canvas_to_window(self.pos - offset));

		ui.invisible_button(format!("node{:?}", nodeid), inner.canvas_to_window_size(self.pos - offset, config::NODE_HITBOX));

		let mut w1: Wire = Wire::skeleton(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));
		let mut w2: Wire = Wire::skeleton(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));

		let active = ui.is_item_active();
		let deactivated = ui.is_item_deactivated();

		if active {
			draw_list.add_rect(inner.canvas_to_window(self.pos - offset), inner.canvas_to_window(self.pos - offset) + inner.canvas_to_window_size(self.pos - offset, config::NODE_HITBOX), 0xffffffff)
				.build();
		}

		if active || deactivated {
			let from = self.pos;
			let to = inner.window_to_canvas(ui.io().mouse_pos.into());
			let vec = to - from;

			// 0 ha vízszintesen kezdte el húzni a felhasználó,
			// 1 ha függőlegesen
			let len = (to - from).length();
			if len == inner.zoom {
				inner.wire_horiz = (to - from).x.abs() != 0.0;
			}

			if inner.wire_horiz {
				// Vízszintes először
				w1 = Wire::skeleton(from, Vec2::new(to.x, from.y));
				w2 = Wire::skeleton(Vec2::new(to.x, from.y), to);

				if vec.x == 0.0 {
					inner.wire_horiz = !inner.wire_horiz;
				}
			} else {
				// Függőleges először
				w1 = Wire::skeleton(from, Vec2::new(from.x, to.y));
				w2 = Wire::skeleton(Vec2::new(from.x, to.y), to);

				if vec.y == 0.0 {
					inner.wire_horiz = !inner.wire_horiz;
				}
			}
		}

		if active {
			w1.draw(inner, &draw_list, None);
			w2.draw(inner, &draw_list, None);
		}

		let mut ret = (None, None);
		if deactivated {
			if w1.start != w1.end { ret.0.replace(w1); }
			if w2.start != w2.end { ret.1.replace(w2); }
		}
		return ret;
	}
}

#[allow(unused)]
#[derive(strum::EnumIter, strum::EnumMessage, Debug, PartialEq, Clone)]
pub enum CompKind {
	#[strum(message = "AND Gate")]	AndGate,
	#[strum(message = "OR Gate")]	OrGate,
	#[strum(message = "NAND Gate")]	NandGate,
	#[strum(message = "NOR Gate")]	XorGate,
	#[strum(message = "Input")]		Input { state: bool },
}

impl CompKind {
	pub const fn hitbox(&self) -> Vec2 {
		match self {
			CompKind::AndGate | CompKind::OrGate => Vec2::new(4.0, 4.0),
			CompKind::NandGate => todo!(),
			CompKind::XorGate => todo!(),
			CompKind::Input { state: _ } => Vec2::new(2.0, 2.0),
		}
	}

	// FONTOS: az outputok jönnek először
	// A tuple második eleme a kimenetek darabszáma
	pub fn nodes(&self) -> (Vec<Node>, usize) {
		match self {
			CompKind::AndGate =>
				(
					vec![
						Node { pos: Vec2::new(4.0, 2.0), owner: None, logic_lvl: LL::U, generation: 0, },
						Node { pos: Vec2::new(0.0, 1.0), owner: None, logic_lvl: LL::U, generation: 0, },
						Node { pos: Vec2::new(0.0, 3.0), owner: None, logic_lvl: LL::U, generation: 0, },
					],
					1
				),
			CompKind::OrGate =>
				(
					vec![
						Node { pos: Vec2::new(5.0, 2.0), owner: None, logic_lvl: LL::U, generation: 0, },
						Node { pos: Vec2::new(1.0, 1.0), owner: None, logic_lvl: LL::U, generation: 0, },
						Node { pos: Vec2::new(1.0, 3.0), owner: None, logic_lvl: LL::U, generation: 0, },
					],
					1
				),
			CompKind::NandGate => todo!(),
			CompKind::XorGate => todo!(),
			CompKind::Input { state: _ } =>
				(
					vec![
						Node { pos: Vec2::new(2.0, 1.0), owner: None, logic_lvl: LL::L, generation: 0, },
					],
					1,
				),
		}
	}
}

#[derive(PartialEq, Clone)]
#[allow(unused)]
pub struct Component {
	pub kind: CompKind,
	pub pos: Vec2,
	pub nodes: Vec<NodeKey>,
	pub id: u64,
	pub move_request: Option<Vec2>,
	pub clicked_at: Option<Vec2>,
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
			clicked_at: None,
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

				// Hátsó görbe görbe
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

			},
			CompKind::XorGate => {

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

		if ui.invisible_button(format!("comp{}", self.id), canvas.canvas_to_window_size(self.pos, self.kind.hitbox())) {
			if let Some(pos) = self.clicked_at && pos.distance_squared(ui.io().mouse_pos.into()) <= config::CLICK_ALLOWANCE.sqrt() {
				clicked = true;
			}
		}

		if ui.is_item_active() {
			if self.clicked_at.is_none() {
				self.clicked_at.replace(ui.io().mouse_pos.into());
			}

			if canvas.grab_mouse_offset.is_none() {
				canvas.grab_mouse_offset.replace((self.id, canvas.canvas_to_window(self.pos) - Vec2::from(ui.io().mouse_pos)));
			}

			self.move_request.replace(canvas.window_to_canvas(canvas.grab_mouse_offset.unwrap().1 + Vec2::from(ui.io().mouse_pos)));
		} else {
			self.clicked_at.take();

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
			nodes.move_node(*nidx, to - self.pos, NodeOwner::Comp(cidx), wires, comps, *generation);
			*generation += 1;
		}

		self.pos = to;
	}

	pub fn on_click(&mut self) {
		match &mut self.kind {
			CompKind::Input { state } => {
				*state = !*state;
				// self.update(generation, &nodes.node_lookup, &mut nodes.node_storage, wires, comps);
			},
			_ => {}
		}
	}

	pub fn update(&self, generation: u32, node_lookup: &NodeLookup, node_storage: &mut NodeStorage, wires: &Wires, comps: &CompStorage) {
		// Kimenet kiszámítása a bemeneteknek megfelelően
		match self.kind {
			CompKind::AndGate => {
				let mut value = node_storage[self.nodes[1]].logic_lvl;
				for n in self.nodes.iter().skip(1) {
					value &= node_storage[*n].logic_lvl;
				}
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, value);
			},
			CompKind::OrGate => {
				let mut value = node_storage[self.nodes[1]].logic_lvl;
				for n in self.nodes.iter().skip(1) {
					value |= node_storage[*n].logic_lvl;
				}
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, value);
			},
			CompKind::NandGate => {
				let mut value = node_storage[self.nodes[1]].logic_lvl;
				for n in self.nodes.iter().skip(1) {
					value &= node_storage[*n].logic_lvl;
				}
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, !value);
			},
			CompKind::XorGate => {
				let mut value = node_storage[self.nodes[1]].logic_lvl;
				for n in self.nodes.iter().skip(1) {
					value ^= node_storage[*n].logic_lvl;
				}
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, value);
			},
			CompKind::Input { state } => {
				node_storage[self.nodes[0]].logic_lvl = state.into();
				set_nodes(node_lookup, node_storage, wires, comps, node_storage[self.nodes[0]].pos, generation, state.into());
			},
		}
	}
}
