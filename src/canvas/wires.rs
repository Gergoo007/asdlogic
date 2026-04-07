use palette::FromColor;

use crate::{canvas::{CanvasInner, NodeHandler, NodeKey, NodeOwner, Vec2, WireStorage, component::Node}, config};

#[derive(Debug)]
pub struct Wire {
	pub start: Vec2,
	pub end: Vec2,
	pub startnode: Option<NodeKey>,
	pub endnode: Option<NodeKey>,
}

impl Wire {
	pub fn skeleton(start: Vec2, end: Vec2) -> Self {
		Self {
			start,
			end,
			startnode: None,
			endnode: None,
		}
	}

	pub fn overwrite(&mut self, start: Vec2, end: Vec2, nodes: &mut NodeHandler) {
		let n1 = nodes.remove_node(self.start, |_| true);
		let n2 = nodes.remove_node(self.end, |_| true);

		self.start = start;
		self.end = end;

		nodes.add_node(Node { pos: self.start, owner: n1.owner });
		nodes.add_node(Node { pos: self.end, owner: n2.owner });
	}

	pub fn draw(&self, canvas: &CanvasInner, draw_list: &imgui::DrawListMut, color: Option<u32>) {
		draw_list.add_line(canvas.canvas_to_window(self.start), canvas.canvas_to_window(self.end), if let Some(c) = color { c } else { 0xffffffff })
			.thickness(config::WIRE_THICKNESS)
			.build();
	}

	pub fn touches(&self, p: Vec2) -> bool {
		let v = self.end - self.start;
		let w = p - self.start;

		let cross = v.perp_dot(w).abs();
		if cross > 1e-6 {
			return false;
		}

		let dot = w.dot(v);
		if dot < 0.0 || dot > v.length_squared() {
			return false;
		}

		true
	}
}

pub struct Wires {
	wires: WireStorage,
}

impl Wires {
	pub fn new() -> Self {
		Self {
			wires: WireStorage::new(),
		}
	}

	pub fn draw(&self, canvas: &CanvasInner, draw_list: &imgui::DrawListMut) {
		for (widx, w) in &self.wires {
			let hsl: palette::Hsla = palette::Hsla::new(widx.into_raw_parts().0 as f32 / self.wires.len() as f32 * 360.0, 0.8, 0.5, 1.0);
			let rgb: palette::Srgb<f32> = palette::Srgb::from_color(hsl);

			let r = (rgb.red	* 255.0) as u32;
			let g = (rgb.green	* 255.0) as u32;
			let b = (rgb.blue	* 255.0) as u32;
			let a = (hsl.alpha	* 255.0) as u32;

			let argb: u32 = (a << 24) | (r << 16) | (g << 8) | b;

			w.draw(canvas, draw_list, Some(argb));
		}
	}

	pub fn try_add(&mut self, new: Wire, nodes: &mut NodeHandler) {
		for (_, old) in self.wires.iter_mut() {
			let v1 = old.start - old.end;
			let v2 = new.start - new.end;
			let parallel = v1.perp_dot(v2).abs() < 1e-6;

			// Egy régi vezetékben elfér az új
			if old.touches(new.start) && old.touches(new.end) {
				return;
			}

			// Az új vezetékben elfér egy másik régebbi
			if new.touches(old.start) && new.touches(old.end) {
				old.overwrite(new.start, new.end, nodes);
				return;
			}

			// Az új vezetéket össze lehet vonni egy meglévővel (csak érintkeznek)
			if parallel {
				if old.start == new.start {
					old.overwrite(new.end, old.end, nodes);
					return;
				} else if old.start == new.end {
					old.overwrite(old.end, new.start, nodes);
					return;
				} else if old.end == new.start {
					old.overwrite(old.start, new.end, nodes);
					return;
				} else if old.end == new.end {
					old.overwrite(old.start, new.start, nodes);
					return;
				}

				// Az új vezetéket össze lehet vonni egy meglévővel (az egyik a másikból indul ki + párhuzamosak)
				if old.touches(new.start) {
					if new.touches(old.start) {
						old.overwrite(old.end, new.end, nodes);
						return;
					} else if new.touches(old.end) {
						old.overwrite(old.start,new.end, nodes);
						return;
					}
				} else if old.touches(new.end) {
					if new.touches(old.start) {
						old.overwrite(old.end, new.start, nodes);
						return;
					} else if new.touches(old.end) {
						old.overwrite(old.start, new.start, nodes);
						return;
					}
				}
			}
		}

		let start = new.start;
		let end = new.end;
		let wire = self.wires.insert(new);
		self.wires[wire].startnode.replace(nodes.add_node(Node { pos: start, owner: Some(NodeOwner::Wire(wire)) }));
		self.wires[wire].endnode.replace(nodes.add_node(Node { pos: end, owner: Some(NodeOwner::Wire(wire)) }));
	}
}
