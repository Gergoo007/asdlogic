use crate::{canvas::{CanvasInner, CompKey, NodeHandler, NodeKey, NodeOwner, Vec2, wires::{Wire, Wires}}, config};

#[derive(PartialEq)]
pub struct Node {
	pub pos: Vec2,
	pub owner: Option<NodeOwner>,
}

impl Node {
	fn draw(&self, c: &Gate, nidx: usize, inner: &mut CanvasInner, draw_list: &imgui::DrawListMut, ui: &imgui::Ui)
	-> (Option<Wire>, Option<Wire>) {
		draw_list.add_circle(inner.canvas_to_window(self.pos), config::NODE_RADIUS, 0xffffffff).filled(true).build();

		let offset = config::NODE_HITBOX / 2.0;

		ui.set_cursor_pos(inner.canvas_to_window(self.pos - offset));

		ui.invisible_button(format!("comp{}input{}", c.id, nidx), inner.canvas_to_window_size(self.pos - offset, config::NODE_HITBOX));

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
			w1.draw(inner, draw_list, None);
			w2.draw(inner, draw_list, None);
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
#[derive(strum::EnumIter, strum::EnumMessage, Debug, PartialEq)]
pub enum GateKind {
	#[strum(message = "AND Gate")]	AndGate,
	#[strum(message = "OR Gate")]	OrGate,
	#[strum(message = "NAND Gate")]	NandGate,
	#[strum(message = "NOR Gate")]	NorGate,
}

impl GateKind {
	pub const fn hitbox(&self) -> Vec2 {
		match self {
			GateKind::AndGate => Vec2::new(3.67, 4.0),
			GateKind::OrGate => todo!(),
			GateKind::NandGate => todo!(),
			GateKind::NorGate => todo!(),
		}
	}

	pub const fn nodes(&self) -> [Node; 3] {
		match self {
			GateKind::AndGate => {
				[
					Node { pos: Vec2::new(0.0, 1.0), owner: None },
					Node { pos: Vec2::new(0.0, 3.0), owner: None },
					Node { pos: Vec2::new(4.0, 2.0), owner: None },
				]
			},
			GateKind::OrGate => todo!(),
			GateKind::NandGate => todo!(),
			GateKind::NorGate => todo!(),
		}
	}
}

#[derive(PartialEq)]
#[allow(unused)]
pub struct Gate {
	pub kind: GateKind,
	pub pos: Vec2,
	pub nodes: [NodeKey; 3],
	pub id: u64,
	pub move_request: Option<Vec2>
}

impl Gate {
	pub fn new(kind: GateKind, pos: Vec2, id: u64, nodes: &mut NodeHandler) -> Self {
		let nodes = kind.nodes().map(|mut v| { v.pos += pos; nodes.add_node(v) });

		Self {
			kind,
			pos,
			nodes,
			id,
			move_request: None,
		}
	}

	pub fn draw(&mut self, canvas: &mut CanvasInner, wires: &mut Wires, nodes: &mut NodeHandler, ui: &imgui::Ui) {
		let draw_list = &ui.get_window_draw_list();

		// A görbe valamiért lejjebb van mint kéne, ez szemre belövi
		let curve_y_offset: f32 = -0.03 / canvas.zoom;

		// draw_list.add_circle(canvas.canvas_to_window(self.pos), 10.0, 0xffffffff).build();

		match self.kind {
			GateKind::AndGate => {
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
			GateKind::OrGate => {

			},
			GateKind::NandGate => {

			},
			GateKind::NorGate => {

			},
		}

		for (nidx, n) in self.nodes.iter().enumerate() {
			let tobeadded = nodes.node_storage[*n].draw(self, nidx, canvas, draw_list, ui);
			if let Some(w) = tobeadded.0 { wires.try_add(w, nodes); }
			if let Some(w) = tobeadded.1 { wires.try_add(w, nodes); }
			// n.draw(self, nidx, canvas, wires, draw_list, ui);
		}

		ui.set_cursor_pos(canvas.canvas_to_window(self.pos));
		ui.invisible_button(format!("comp{}", self.id), canvas.canvas_to_window_size(self.pos, self.kind.hitbox()));
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
	}

	pub fn move_to(&mut self, to: Vec2, nodes: &mut NodeHandler, cidx: CompKey) {
		// Régi node-ok eltávolítása, újak hozzáadása
		// Csak a HashMap-nél kell megváltoztatni a Key-t, az Arénában maradhat az Index

		// // Koordináták frissítése a HashMap-ben
		// for nidx in self.nodes {
		// 	let oldpos = nodes.node_storage[nidx].pos;
			
		// }

		// // Koordináták frissítése az Arénában
		// for nidx in self.nodes {
		// 	nodes.node_storage[nidx].pos += to - self.pos;
		// }

		for nidx in self.nodes {
			nodes.move_node(nidx, to - self.pos, NodeOwner::Comp(cidx));
		}

		self.pos = to;
	}

	fn update(&mut self) {

	}
}
