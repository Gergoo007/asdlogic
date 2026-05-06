use serde::{Deserialize, Serialize};

use crate::{canvas::{CanvasInner, CompStorage, ElemIndex, NodeHandler, Vec2, nodes::{Node, NodeKey}}, config};

pub type WireStorage = typed_generational_arena::StandardArena<Wire>;
pub type WireKey = typed_generational_arena::StandardIndex<Wire>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Wire {
	pub start: Vec2,
	pub end: Vec2,
	pub startnode: Option<NodeKey>,
	pub endnode: Option<NodeKey>,
	pub selected: bool,
}

impl Wire {
	pub fn skeleton(start: Vec2, end: Vec2) -> Self {
		Self {
			start,
			end,
			startnode: None,
			endnode: None,
			selected: false,
		}
	}

	pub fn modify(&mut self, start: Vec2, end: Vec2, nodes: &mut NodeHandler, oldidx: WireKey, wires: &Wires, comps: &CompStorage, generation: &mut u32) {
		let node1 = nodes.remove_node(self.start, ElemIndex::Wire(oldidx), wires, comps, generation, false);
		let node2 = nodes.remove_node(self.end, ElemIndex::Wire(oldidx), wires, comps, generation, false);

		self.start = start;
		self.end = end;

		self.startnode.replace(nodes.add_node(Node { pos: self.start, owner: Some(ElemIndex::Wire(oldidx)), ..node1 }));
		self.endnode.replace(nodes.add_node(Node { pos: self.end, owner: Some(ElemIndex::Wire(oldidx)), ..node2 }));
	}

	pub fn process(&self, canvas: &CanvasInner, ui: &imgui::Ui, id: Option<u32>) {
		if let Some(id) = id {
			ui.set_cursor_pos(canvas.canvas_to_window((self.start + self.end - 1.2 / canvas.zoom) / 2.0));
			if canvas.debug {
				ui.text(format!("w{}", id));
			}

			let s = Vec2::new(self.start.x.min(self.end.x), self.start.y.min(self.end.y));
			let e = Vec2::new(self.start.x.max(self.end.x), self.start.y.max(self.end.y));

			let mut size = canvas.canvas_to_window_size(s, e - s);
			let mut pos = canvas.canvas_to_window(s);
			let btnsize = config::WIRE_HITBOX_WIDTH * canvas.zoom;
			if size.x == 0.0 {
				size.x = btnsize;
				pos.x -= btnsize / 2.0;
			} else if size.y == 0.0 {
				size.y = btnsize;
				pos.y -= btnsize / 2.0;
			} else {
				unreachable!("Diagonal wires not supported yet!");
			}

			ui.set_cursor_pos(pos);
			ui.invisible_button(format!("wire{}", id), size);
		}
	}

	pub fn draw(&self, canvas: &CanvasInner, ui: &imgui::Ui, color: Option<u32>, id: Option<u32>) {
		ui.get_window_draw_list().add_line(canvas.canvas_to_window(self.start), canvas.canvas_to_window(self.end), if let Some(c) = color { c } else { 0xffffffff })
			.thickness(config::WIRE_THICKNESS * canvas.zoom)
			.build();

		if let Some(id) = id {
			ui.set_cursor_pos(canvas.canvas_to_window((self.start + self.end - 1.2 / canvas.zoom) / 2.0));
			if canvas.debug {
				ui.text(format!("w{}", id));
			}

			let s = Vec2::new(self.start.x.min(self.end.x), self.start.y.min(self.end.y));
			let e = Vec2::new(self.start.x.max(self.end.x), self.start.y.max(self.end.y));

			let mut size = canvas.canvas_to_window_size(s, e - s);
			let mut pos = canvas.canvas_to_window(s);
			let btnsize = config::WIRE_HITBOX_WIDTH * canvas.zoom;
			if size.x == 0.0 {
				size.x = btnsize;
				pos.x -= btnsize / 2.0;
			} else if size.y == 0.0 {
				size.y = btnsize;
				pos.y -= btnsize / 2.0;
			} else {
				unreachable!("Diagonal wires not supported yet!");
			}

			ui.set_cursor_pos(pos);
			ui.invisible_button(format!("wire{}", id), size);
		}
	}

	pub fn touches(&self, p: Vec2) -> bool {
		if self.start == p { return true; }
		if self.end == p { return true; }

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
	pub fn split(&mut self, at: Vec2, nodes: &mut NodeHandler, widx: WireKey, wires: &Wires, comps: &CompStorage, generation: &mut u32) -> Wire {
		assert!(self.touches(at));
		assert!(self.start != at);
		assert!(self.end != at);

		let oldend = self.end;
		nodes.move_node(self.endnode.unwrap(), at - self.end, ElemIndex::Wire(widx), wires, comps, generation);
		self.end = at;

		Wire {
			start: at,
			end: oldend,
			startnode: None,
			endnode: None,
			selected: false,
		}
	}

	pub fn move_by(&mut self, by: Vec2, widx: WireKey, nodes: &mut NodeHandler, wires: &Wires, comps: &CompStorage, generation: &mut u32) {
		self.start += by;
		nodes.move_node(self.startnode.unwrap(), by, ElemIndex::Wire(widx), wires, comps, generation);
		*generation += 1;

		self.end += by;
		nodes.move_node(self.endnode.unwrap(), by, ElemIndex::Wire(widx), wires, comps, generation);
		*generation += 1;
	}

	pub fn remove_nodes(&self, nodes: &mut NodeHandler, widx: WireKey, wires: &Wires, comps: &CompStorage, generation: &mut u32) {
		nodes.remove_node(nodes.node_storage[self.startnode.unwrap()].pos, ElemIndex::Wire(widx), wires, comps, generation, true);
		nodes.remove_node(nodes.node_storage[self.endnode.unwrap()].pos, ElemIndex::Wire(widx), wires, comps, generation, true);
	}
}

#[derive(Serialize, Deserialize)]
pub struct Wires {
	pub wires: WireStorage,
}

impl Wires {
	pub fn new() -> Self {
		Self {
			wires: WireStorage::new(),
		}
	}

	pub fn add(&mut self, start: Vec2, end: Vec2, nodes: &mut NodeHandler) -> WireKey {
		let wire = self.wires.insert(Wire::skeleton(start, end));

		let mut ll1 = nodes.query_node(start);
		if !ll1.merge(nodes.query_node(end)) {
			// Rövidzárlat; output a rossz Node?

		}

		self.wires[wire].startnode.replace(nodes.add_node(Node { pos: start, owner: Some(ElemIndex::Wire(wire)), logic_lvl: ll1, generation: 0, output: false, }));
		self.wires[wire].endnode.replace(nodes.add_node(Node { pos: end, owner: Some(ElemIndex::Wire(wire)), logic_lvl: ll1, generation: 0, output: false, }));

		wire
	}

	fn overwrite(&mut self, start: Vec2, end: Vec2, nodes: &mut NodeHandler, oldidx: WireKey, comps: &CompStorage, generation: &mut u32) {
		let mut w = self.wires[oldidx].clone();
		w.modify(start, end, nodes, oldidx, self, comps, generation);
		self.wires[oldidx] = w;
	}

	pub fn try_add(&mut self, new: Wire, nodes: &mut NodeHandler, comps: &CompStorage, generation: &mut u32) {
		let mut runanother = true;
		let mut insert = true;

		// Lehet hogy 1 v. 2 vezetéket meg kell felezni
		let mut secondwire: Option<Wire> = None;
		let mut thirdwire: Option<Wire> = None;

		'check: while runanother {
			runanother = false;

			let keys: Vec<_> = self.wires.iter().map(|(e, _)| e).collect();
			for oldidx in keys {
				let old = &self.wires[oldidx];
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

					self.overwrite(new.start, new.end, nodes, oldidx, comps, generation);
					runanother = true; continue 'check;
				}

				// Az új vezetéket össze lehet vonni egy meglévővel (csak érintkeznek)
				if parallel {
					if old.start == new.start && nodes.count_nodes(old.start) <= 1 {
						self.overwrite(new.end, old.end, nodes, oldidx, comps, generation);
						runanother = true; continue 'check;
					} else if old.start == new.end && nodes.count_nodes(old.start) <= 1 {
						self.overwrite(old.end, new.start, nodes, oldidx, comps, generation);
						runanother = true; continue 'check;
					} else if old.end == new.start && nodes.count_nodes(old.end) <= 1 {
						self.overwrite(old.start, new.end, nodes, oldidx, comps, generation);
						runanother = true; continue 'check;
					} else if old.end == new.end && nodes.count_nodes(old.end) <= 1 {
						self.overwrite(old.start, new.start, nodes, oldidx, comps, generation);
						runanother = true; continue 'check;
					}

					// Az új vezetéket össze lehet vonni egy meglévővel (az egyik a másikból indul ki + párhuzamosak)
					if old.touches(new.start) && nodes.count_nodes(new.start) <= 1 {
						if new.touches(old.start) && nodes.count_nodes(old.start) <= 1 {
							self.overwrite(old.end, new.end, nodes, oldidx, comps, generation);
							runanother = true; continue 'check;
						} else if new.touches(old.end) && nodes.count_nodes(old.end) <= 1 {
							self.overwrite(old.start,new.end, nodes, oldidx, comps, generation);
							runanother = true; continue 'check;
						}
					} else if old.touches(new.end) && nodes.count_nodes(new.end) <= 1 {
						if new.touches(old.start) && nodes.count_nodes(old.start) <= 1 {
							self.overwrite(old.end, new.start, nodes, oldidx, comps, generation);
							runanother = true; continue 'check;
						} else if new.touches(old.end) && nodes.count_nodes(old.end) <= 1 {
							self.overwrite(old.start, new.start, nodes, oldidx, comps, generation);
							runanother = true; continue 'check;
						}
					}
				} else {
					if old.touches(new.start) && old.start != new.start && old.end != new.start {
						assert!(secondwire.is_none());
						let mut oldw = self.wires[oldidx].clone();
						let neww = oldw.split(new.start, nodes, oldidx, self, comps, generation);
						self.wires[oldidx] = oldw;
						secondwire.replace(neww);
					} else if old.touches(new.end) && old.start != new.end && old.end != new.end {
						assert!(thirdwire.is_none());
						let mut oldw = self.wires[oldidx].clone();
						let neww = oldw.split(new.end, nodes, oldidx, self, comps, generation);
						self.wires[oldidx] = oldw;
						thirdwire.replace(neww);
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
