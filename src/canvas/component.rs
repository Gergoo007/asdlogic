use crate::canvas::{CanvasInner, Vec2};

struct Node {

}

#[allow(unused)]
#[derive(strum::EnumIter, strum::EnumMessage, Debug)]
pub enum GateKind {
	#[strum(message = "AND Gate")]	AndGate,
	#[strum(message = "OR Gate")]	OrGate,
	#[strum(message = "NAND Gate")]	NandGate,
	#[strum(message = "NOR Gate")]	NorGate,
}

impl GateKind {
	pub const fn hitbox(&self) -> Vec2 {
		match self {
			GateKind::AndGate => Vec2::new(4.67, 4.0),
			GateKind::OrGate => todo!(),
			GateKind::NandGate => todo!(),
			GateKind::NorGate => todo!(),
		}
	}
}

#[allow(unused)]
pub struct Gate {
	pub kind: GateKind,
	pub pos: Vec2,
		inputs: Vec<Node>,
		output: Node,
	pub id: u64,
	pub move_request: Option<Vec2>
}

impl Gate {
	pub fn new(kind: GateKind, pos: Vec2, id: u64) -> Self {
		let inputs = vec![
			Node {  },
			Node {  },
		];

		let output = Node {

		};

		Gate {
			kind,
			pos,
			inputs,
			output,
			id,
			move_request: None,
		}
	}

	pub fn draw(&mut self, canvas: &mut CanvasInner, ui: &imgui::Ui) {
		let draw_list = &ui.get_window_draw_list();

		// A görbe valamiért lejjebb van mint kéne, ez szemre belövi
		let curve_y_offset: f32 = -0.03 / canvas.zoom;

		match self.kind {
			GateKind::AndGate => {
				draw_list.add_bezier_curve(
					canvas.canvas_to_window(self.pos + Vec2::new(2.00, 4.0 - curve_y_offset)),
					canvas.canvas_to_window(self.pos + Vec2::new(4.67, 4.0 - curve_y_offset)),
					canvas.canvas_to_window(self.pos + Vec2::new(4.67, 0.0 - curve_y_offset)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.00, 0.0 - curve_y_offset)),
					0xffffffff
				).build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 4.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 4.0)),
					0xffffffff
				).build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 0.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(2.0, 0.0)),
					0xffffffff
				).build();

				draw_list.add_line(
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 4.0)),
					canvas.canvas_to_window(self.pos + Vec2::new(0.0, 0.0)),
					0xffffffff
				).build();
			},
			GateKind::OrGate => {

			},
			GateKind::NandGate => {

			},
			GateKind::NorGate => {

			},
		}

		// draw_list.add_rect(canvas.canvas_to_window(self.pos), canvas.canvas_to_window(self.pos + self.kind.hitbox()), 0x80808080).build();

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
			// self.move_request.take();
		}
	}

	fn update(&mut self) {

	}
}

// pub enum Component {
// 	Gate(Gate)
// }

// impl Component {
// 	pub fn draw(&mut self, canvas: &mut CanvasInner, ui: &imgui::Ui) {
// 		match self {
// 			Component::Gate(gate) => {
// 				gate.draw(canvas, ui);
// 			},
// 		}
// 	}
// }
