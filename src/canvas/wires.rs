use imgui::Ui;

use crate::{canvas::{CanvasInner, NodeHandler, NodeKey, NodeOwner, Vec2, WireKey, WireStorage, component::{Node}, logic::LL}, config};

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

	pub fn overwrite(&mut self, start: Vec2, end: Vec2, nodes: &mut NodeHandler, oldidx: WireKey) {
		let node1 = nodes.remove_node(self.start, NodeOwner::Wire(oldidx));
		let node2 = nodes.remove_node(self.end, NodeOwner::Wire(oldidx));

		self.start = start;
		self.end = end;

		nodes.add_node(Node { pos: self.start, owner: Some(NodeOwner::Wire(oldidx)), ..node1 });
		nodes.add_node(Node { pos: self.end, owner: Some(NodeOwner::Wire(oldidx)), ..node2 });
	}

	pub fn draw(&self, canvas: &CanvasInner, draw_list: &imgui::DrawListMut, color: Option<u32>) {
		draw_list.add_line(canvas.canvas_to_window(self.start), canvas.canvas_to_window(self.end), if let Some(c) = color { c } else { 0xffffffff })
			.thickness(config::WIRE_THICKNESS)
			.build();
	}

	pub fn touches(&self, p: Vec2) -> bool {
		if self.start == p { return false; }
		if self.end == p { return false; }

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

	// Megfelezi magát, és egy új vezeték koordinátáit adja vissza
	pub fn split(&mut self, at: Vec2) -> Wire {
		assert!(self.touches(at));
		assert!(self.start != at);
		assert!(self.end != at);

		let oldend = self.end;

		self.end = at;

		Wire {
			start: at,
			end: oldend,
			startnode: None,
			endnode: None
		}
	}
}

pub struct Wires {
	pub wires: WireStorage,
}

impl Wires {
	pub fn new() -> Self {
		Self {
			wires: WireStorage::new(),
		}
	}

	fn add(&mut self, start: Vec2, end: Vec2, nodes: &mut NodeHandler) -> WireKey {
		let wire = self.wires.insert(Wire::skeleton(start, end));
		self.wires[wire].startnode.replace(nodes.add_node(Node { pos: start, owner: Some(NodeOwner::Wire(wire)), logic_lvl: LL::U, generation: 0, }));
		self.wires[wire].endnode.replace(nodes.add_node(Node { pos: end, owner: Some(NodeOwner::Wire(wire)), logic_lvl: LL::U, generation: 0, }));
		wire
	}

	pub fn draw(&mut self, canvas: &mut CanvasInner, ui: &Ui, nodes: &NodeHandler) {
		let draw_list = ui.get_window_draw_list();
		for (_, w) in &self.wires {
			// let hsl: palette::Hsla = palette::Hsla::new(widx.into_raw_parts().0 as f32 / self.wires.len() as f32 * 360.0, 0.8, 0.5, 1.0);
			// let rgb: palette::Srgb<f32> = palette::Srgb::from_color(hsl);

			// let r = (rgb.red	* 255.0) as u32;
			// let g = (rgb.green	* 255.0) as u32;
			// let b = (rgb.blue	* 255.0) as u32;
			// let a = (hsl.alpha	* 255.0) as u32;

			// let argb: u32 = (a << 24) | (r << 16) | (g << 8) | b;

			debug_assert!(nodes.node_storage[w.startnode.unwrap()].logic_lvl == nodes.node_storage[w.endnode.unwrap()].logic_lvl);

			let logic_lvl = &nodes.node_storage[w.startnode.unwrap()].logic_lvl;
			let argb = logic_lvl.to_color();

			w.draw(canvas, &draw_list, Some(argb));
		}
	}

	pub fn try_add(&mut self, new: Wire, nodes: &mut NodeHandler) {
		let mut runanother = true;
		let mut insert = true;

		// Lehet hogy 1 v. 2 vezetéket meg kell felezni
		let mut secondwire: Option<Wire> = None;
		let mut thirdwire: Option<Wire> = None;

		'check: while runanother {
			runanother = false;

			for (oldidx, old) in self.wires.iter_mut() {
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
					if nodes.count_nodes(old.start) >= 2 || nodes.count_nodes(old.end) >= 2 {
						// Ezt az esetet hagyjuk is a gecibe, én biztos nem fogom ezt kezelni
						return;
					}

					old.overwrite(new.start, new.end, nodes, oldidx);
					runanother = true; continue 'check;
				}

				// Az új vezetéket össze lehet vonni egy meglévővel (csak érintkeznek)
				if parallel {
					if nodes.count_nodes(old.start) == 1 {
						if old.start == new.start {
							old.overwrite(new.end, old.end, nodes, oldidx);
							runanother = true; continue 'check;
						} else if old.start == new.end {
							old.overwrite(old.end, new.start, nodes, oldidx);
							runanother = true; continue 'check;
						} else if old.end == new.start {
							old.overwrite(old.start, new.end, nodes, oldidx);
							runanother = true; continue 'check;
						} else if old.end == new.end {
							old.overwrite(old.start, new.start, nodes, oldidx);
							runanother = true; continue 'check;
						}
					}

					// Az új vezetéket össze lehet vonni egy meglévővel (az egyik a másikból indul ki + párhuzamosak)
					if old.touches(new.start) {
						if new.touches(old.start) {
							old.overwrite(old.end, new.end, nodes, oldidx);
							runanother = true; continue 'check;
						} else if new.touches(old.end) {
							old.overwrite(old.start,new.end, nodes, oldidx);
							runanother = true; continue 'check;
						}
					} else if old.touches(new.end) {
						if new.touches(old.start) {
							old.overwrite(old.end, new.start, nodes, oldidx);
							runanother = true; continue 'check;
						} else if new.touches(old.end) {
							old.overwrite(old.start, new.start, nodes, oldidx);
							runanother = true; continue 'check;
						}
					}
				} else {
					if old.touches(new.start) && old.start != new.start && old.end != new.start {
						assert!(secondwire.is_none());
						secondwire.replace(old.split(new.start));
					} else if old.touches(new.end) && old.start != new.end && old.end != new.end {
						assert!(thirdwire.is_none());
						thirdwire.replace(old.split(new.end));
					}
				}
			}
		}

		if let Some(w) = secondwire { self.add(w.start, w.end, nodes); }
		if let Some(w) = thirdwire { self.add(w.start, w.end, nodes); }

		if insert {
			self.add(new.start, new.end, nodes);
		}
	}
}
