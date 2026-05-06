use serde::{Deserialize, Serialize};
use crate::canvas::{Canvas, Vec2, component::CompKind};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Action {
	AddComp { to: Vec2, kind: CompKind, },
	DeleteComp { at: Vec2, kind: CompKind },
	MoveComp { from: Vec2, to: Vec2 },
	MoveWire { from: Vec2, to: Vec2, by: Vec2 },
	AddWire { from: Vec2, to: Vec2 },
	DeleteWire { from: Vec2, to: Vec2 },
	OverwriteWire { oldstart: Vec2, oldend: Vec2, newstart: Vec2, newend: Vec2, },
}

impl Action {
	fn reverse(&self) -> Self {
		match *self {
			Action::AddComp { to, kind } => Action::DeleteComp { at: to, kind },
			Action::DeleteComp { at, kind } => Action::AddComp { to: at, kind },
			Action::MoveComp { from, to } => Action::MoveComp { from: to, to: from },
			Action::MoveWire { from, to, by } => Action::MoveWire { from: from + by, to: to + by, by: -by },
			Action::AddWire { from, to } => Action::DeleteWire { from, to },
			Action::DeleteWire { from, to } => Action::AddWire { from, to },
			Action::OverwriteWire { oldstart, oldend, newstart, newend } =>
				Action::OverwriteWire { oldstart: newstart, oldend: newend, newstart: oldstart, newend: oldend, },
		}
	}
}

#[derive(Serialize, Deserialize)]
struct Timepoint {
	actions: Vec<Action>,
}

impl Timepoint {
	pub fn new() -> Self {
		Self {
			actions: Vec::new()
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct History {
	    records: Vec<Timepoint>,
	    cursor: usize,
	pub recording: bool,
}

impl History {
	pub fn new() -> Self {
		Self {
			records: Vec::new(),
			cursor: 0,
			recording: false,
		}
	}
}

impl Canvas {
	pub fn start_record(&mut self) {
		assert!(!self.history.recording);
		self.history.recording = true;
		self.history.records.resize_with(self.history.cursor + 1, || Timepoint::new());
	}

	pub fn end_record(&mut self) {
		assert!(self.history.recording);
		self.history.recording = false;
		self.history.cursor += 1;
	}

	pub fn add_and_execute(&mut self, action: Action) {
		assert!(self.history.recording);
		self.history.records[self.history.cursor].actions.push(action);
		self.execute_action(action);
	}

	pub fn record_and_execute(&mut self, action: Action) {
		self.start_record();
		self.add_and_execute(action);
		self.end_record();
	}

	fn execute_action(&mut self, action: Action) {
		// println!("execute {:?}", action);
		match action {
			Action::AddComp { to, kind } => {
				self.add_comp(kind, to);
			},
			Action::AddWire { from, to } => {
				self.wires.add(from, to, &mut self.nodes);
			},
			Action::DeleteComp { at, kind: _ } => {
				let mut rem = None;
				for (cidx, c) in &self.comps {
					if c.pos == at {
						rem.replace(cidx);
						break;
					}
				}
				let t = self.comps.remove(rem.unwrap()).unwrap();
				t.remove_nodes(&mut self.nodes, rem.unwrap(), &self.wires, &self.comps, &mut self.update_generation);
			},
			Action::DeleteWire { from, to } => {
				let mut to_remove = None;
				for (widx, w) in &self.wires.wires {
					if w.start == from && w.end == to {
						w.remove_nodes(&mut self.nodes, widx, &self.wires, &self.comps, &mut self.update_generation);
						to_remove.replace(widx);
					}
				}
				self.wires.wires.remove(to_remove.unwrap());
			},
			Action::OverwriteWire { oldstart, oldend, newstart, newend, } => {
				let (widx, _) = self.wires.wires.iter_mut().find(|(_, w)| w.start == oldstart && w.end == oldend).unwrap();
				let mut jaj = self.wires.wires[widx].clone();
				jaj.modify(newstart, newend, &mut self.nodes, widx, &self.wires, &self.comps, &mut self.update_generation);
				self.wires.wires[widx] = jaj;
			},
			Action::MoveComp { from, to } => {
				let r = self.comps.iter_mut().find(|(_, c)| c.pos == from);
				if r.is_none() {
					println!("failed to find c {:?}", from);
					return;
				}
				let (cidx, _) = r.unwrap();
				let mut c2 = self.comps[cidx].clone();
				c2.move_by(to - from, &mut self.nodes, cidx, &self.wires, &self.comps, &mut self.update_generation);
				self.comps[cidx] = c2;
			},
			Action::MoveWire { from, to, by } => {
				let r = self.wires.wires.iter().find(|(_, w)| (w.start == from && w.end == to) || (w.end == from && w.start == to));
				if r.is_none() {
					println!("failed to find w {:?}", from);
					return;
				}
				let (widx, _) = r.unwrap();
				let mut w = self.wires.wires[widx].clone();
				w.move_by(by, widx, &mut self.nodes, &self.wires, &self.comps, &mut self.update_generation);
				self.wires.wires[widx] = w;
			}
		}
	}

	fn revert_action(&mut self, action: Action) { self.execute_action(action.reverse()); }

	pub fn undo(&mut self) {
		if self.history.cursor <= 0 {
			println!("Nothing to undo!");
		} else {
			self.history.cursor -= 1;
			let asd = &self.history.records[self.history.cursor].actions;
			for i in (0..asd.len()).rev() {
				let act = self.history.records[self.history.cursor].actions[i];
				self.revert_action(act);
			}
		}
	}

	pub fn redo(&mut self) {
		if self.history.cursor >= self.history.records.len() {
			println!("Nothing to redo!");
		} else {
			let asd = &self.history.records[self.history.cursor].actions;
			for i in 0..asd.len() {
				let act = self.history.records[self.history.cursor].actions[i];
				self.execute_action(act);
			}
			self.history.cursor += 1;
		}
	}
}
