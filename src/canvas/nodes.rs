use std::collections::HashMap;

use glam::IVec2;
use serde::{Deserialize, Serialize};

use crate::{canvas::{CompStorage, ElemIndex, Vec2, inner::CanvasInner, logic::LL, vec2int, wires::{Wire, Wires}}, config};

pub type NodeStorage = typed_generational_arena::StandardArena<Node>;
pub type NodeKey = typed_generational_arena::StandardIndex<Node>;
pub type NodeLookup = HashMap<IVec2, Vec<NodeKey>>;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Node {
	pub pos: Vec2,
	pub owner: Option<ElemIndex>,
	pub logic_lvl: LL,
	pub generation: u32,
	pub output: bool,
}

impl Node {
	pub fn draw(&self, inner: &mut CanvasInner, ui: &imgui::Ui) {
		ui.get_window_draw_list().add_circle(inner.canvas_to_window(self.pos), config::NODE_RADIUS * inner.zoom, self.logic_lvl.to_color())
			.filled(true)
			.build();
	}

	pub fn process(&self, nodeid: NodeKey, inner: &mut CanvasInner, ui: &imgui::Ui)
	-> (Option<Wire>, Option<Wire>) {
		// draw_list.add_circle(inner.canvas_to_window(self.pos), config::NODE_RADIUS, 0xffffffff).filled(true).build();

		let offset = config::NODE_HITBOX / 2.0;

		ui.set_cursor_pos(inner.canvas_to_window(self.pos - offset));

		ui.invisible_button(format!("node{:?}", nodeid), inner.canvas_to_window_size(self.pos - offset, config::NODE_HITBOX));

		let mut w1: Wire = Wire::skeleton(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));
		let mut w2: Wire = Wire::skeleton(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));

		let active = ui.is_item_active();
		let deactivated = ui.is_item_deactivated();

		if active {
			ui.get_window_draw_list().add_rect(inner.canvas_to_window(self.pos - offset), inner.canvas_to_window(self.pos - offset) + inner.canvas_to_window_size(self.pos - offset, config::NODE_HITBOX), 0xffffffff)
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
			w1.draw(inner, ui, None, None);
			w2.draw(inner, ui, None, None);
		}

		let mut ret = (None, None);
		if deactivated {
			if w1.start != w1.end { ret.0.replace(w1); }
			if w2.start != w2.end { ret.1.replace(w2); }
		}
		return ret;
	}
}

#[derive(Serialize, Deserialize)]
pub struct NodeHandler {
	pub node_storage: NodeStorage,
	pub node_lookup: NodeLookup,
}

pub fn check_driven(node_lookup: &NodeLookup, node_storage: &mut NodeStorage, wires: &Wires, comps: &CompStorage, at: Vec2, generation: &mut u32, inc_gen: bool) -> bool {
	if inc_gen {
		*generation += 1;
	}

	if !node_lookup.contains_key(&vec2int(at)) { return false; }

	for node in &node_lookup[&vec2int(at)] {
		let k = *node;
		let node = &mut node_storage[k];

		let oldgen = node.generation;

		node.generation = *generation;

		if node.output {
			return true;
		}

		// Itt már jártam
		if oldgen == *generation {
			return false;
		}

		if let Some(ElemIndex::Wire(w)) = node.owner {
			// Ha vezeték, akkor a másik node-ot is be kell állítani
			if k == wires.wires[w].startnode.unwrap() {
				if check_driven(node_lookup, node_storage, wires, comps, wires.wires[w].end, generation, false) {
					return true;
				}
			} else if k == wires.wires[w].endnode.unwrap() {
				if check_driven(node_lookup, node_storage, wires, comps, wires.wires[w].start, generation, false) {
					return true;
				}
			} else {
				unreachable!("he????");
			}
		}
	}
	return false;
}

// Beállítja az összes Node logikai értékét
// Külön eset ha LL::U-ra kell beállítani, mivel akkor végig fog menni az összes csatlakozáson megnézni hogy
// meg van-e hajtva a hálózat. Ha meg van, akkor nem csinál semmit.
pub fn set_nodes(node_lookup: &NodeLookup, node_storage: &mut NodeStorage, wires: &Wires, comps: &CompStorage, at: Vec2, generation: &mut u32, ll: LL, inc_gen: bool) {
	// Itt kell lecsekkolni hogy lebegőnek kell-e lennie ezeknek a Node-oknak
	if ll == LL::U && inc_gen {
		if check_driven(node_lookup, node_storage, wires, comps, at, generation, true) {
			return;
		}
	}

	if inc_gen {
		*generation += 1;
	}

	if !node_lookup.contains_key(&vec2int(at)) { return; }

	for node in &node_lookup[&vec2int(at)] {
		let k = *node;
		let node = &mut node_storage[k];

		if node.generation == *generation {
			if node.logic_lvl != ll {
				eprintln!("Short circuit! {:?} vs {:?}", node.logic_lvl, ll);
			}
			return;
		}

		node.logic_lvl = ll;
		node.generation = *generation;

		if let Some(ElemIndex::Wire(w)) = node.owner {
			// Ha vezeték, akkor a másik node-ot is be kell állítani
			if k == wires.wires[w].startnode.unwrap() {
				set_nodes(node_lookup, node_storage, wires, comps, wires.wires[w].end, generation, ll, false);
			} else if k == wires.wires[w].endnode.unwrap() {
				set_nodes(node_lookup, node_storage, wires, comps, wires.wires[w].start, generation, ll, false);
			} else {
				unreachable!("he????");
			}
		} else if let Some(ElemIndex::Comp(c)) = node.owner {
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
		let output = n.output;
		let idx = self.node_storage.insert(n);
		self.node_lookup.entry(vec2int(pos)).and_modify(|v| {
			// Ha már van itt Node, akkor az új Node is felveszi az itt lévő Node-ok logikai értékét
			let ll = self.node_storage[idx].logic_lvl;
			v.push(idx);
			for v in v.iter() {
				if !self.node_storage[*v].logic_lvl.merge(ll) {
					if output {
						unreachable!("Short circuit when adding new node!");
					}
				}
			}
		}).or_insert(vec![idx]);
		idx
	}

	pub fn count_nodes(&self, at: Vec2) -> usize {
		if self.node_lookup.contains_key(&vec2int(at)) {
			self.node_lookup[&vec2int(at)].len()
		} else {
			0
		}
	}

	pub fn remove_node(&mut self, at: Vec2, owner: ElemIndex) -> Node {
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

	pub fn query_node(&self, at: Vec2) -> LL {
		if self.node_lookup.contains_key(&vec2int(at)) {
			self.node_storage[self.node_lookup[&vec2int(at)][0]].logic_lvl
		} else {
			LL::U
		}
	}

	pub fn move_node(&mut self, nidx: NodeKey, by: Vec2, owner: ElemIndex, wires: &Wires, comps: &CompStorage, generation: &mut u32) {
		let oldpos = self.node_storage[nidx].pos;
		let newpos = oldpos + by;
		let k = vec2int(oldpos);

		// Ha van már Node a cél koordinátán akkor ez tárolja el az értékét
		let mut destination_logic_level = None;

		// Koordináták frissítése a HashMap-ben:
		// 1.1. ki kell venni a releváns NodeKey-eket az adott koordinátából
		let mut iter = self.node_lookup.get_mut(&k).unwrap().extract_if(.., |v| {
			self.node_storage[*v].owner == Some(owner)
		});
		let node = iter.next().unwrap();

		// assert!(iter.next().is_none());

		drop(iter);

		// 1.2. ha nem maradt másik Node akkor a Vektor felszabadításra kerül
		if self.node_lookup.get_mut(&k).unwrap().len() == 0 {
			self.node_lookup.remove(&k);
		}

		// 2. vissza kell tenni az új koordinátára a kivett értékeket
		self.node_lookup.entry(vec2int(newpos)).and_modify(|nodekeys| {
			destination_logic_level.replace(self.node_storage[nodekeys[0]].logic_lvl);
			nodekeys.push(node);
		}).or_insert(vec![ node ]);

		// Koordináták frissítése az Arénában
		self.node_storage[nidx].pos += by;

		// Az esetleges új kapcsolatok feldolgozása
		let ll = self.node_storage[nidx].logic_lvl;
		set_nodes(&self.node_lookup, &mut self.node_storage, wires, comps, newpos, generation, ll, true);

		// Ha nincs meghajtva ez a Node akkor legyen LL::U az értéke
		if !check_driven(&self.node_lookup, &mut self.node_storage, wires, comps, newpos, generation, true) && !self.node_storage[nidx].output {
			self.node_storage[nidx].logic_lvl = LL::U;
		} else {
			if let Some(lvl) = destination_logic_level {
				self.node_storage[nidx].logic_lvl = lvl;
			}
		}
	}
}
