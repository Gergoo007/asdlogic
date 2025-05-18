#include <iostream>
#include <algorithm>
#include <cmath>

#include <types.hh>
#include <util.hh>
#include <canvas.hh>
#include <config.hh>
#include "comp.hh"
#include "compdefs.hh"

static u32 counter;
Node* selected_node;

ImU32 ns_colors[4] = {
	/* [NS_0] = */ IM_COL32(255, 76, 60, 255),
	/* [NS_1] = */ IM_COL32(0, 255, 255, 255),
	/* [NS_U] = */ IM_COL32(200, 200, 200, 255),
	/* [NS_E] = */ IM_COL32(255, 0, 0, 255),
};

static u8 updategen = 0;

static inline void process_node(ImDrawList* draw_list, ImVec2 pos, Node& node) {
	const ImColor c = node.parent->selected ? red : ns_colors[node.state];

	draw_list->AddCircleFilled(mc.convert(pos + node.pos), noderad * mc.zoom, c);
	// printf("trui %f %f\n", pos.x + node.pos.x, mc.convert(ImVec2(pos.x + node.pos.x, 0)).x);
	const ImVec2 hb = ImVec2(hitbox, hitbox) * mc.zoom;
	ImGui::SetCursorPos(mc.convert(pos + node.pos) - hb / 2);
	ImGui::InvisibleButton(node.id, hb);

	if (ImGui::IsItemActive()) {
		draw_list->AddLine(mc.convert(pos + node.pos), ImGui::GetMousePos(), ns_colors[node.state], wireth);
		selected_node = &node;
	}

	if (ImGui::IsItemHovered(ImGuiHoveredFlags_AllowWhenBlockedByActiveItem) && ImGui::GetIO().MouseReleased[0]) {
		// node: ez a node a kurzor alatt
		// selected: ahonnan húzom a kábelt
		if (&node == selected_node) { printf("Saját magába csatlakozás\n"); return; }
		if (node.input == selected_node->input) { printf("Érvénytelen csatlakozás!\n"); return; }

		// Ez se normális
		if (std::find(selected_node->connect_to.begin(), selected_node->connect_to.end(), &node) != selected_node->connect_to.end()) {
			printf("Már van conn, törlés\n");

			for (u32 i = 0; i < selected_node->connect_to.size(); i++) {
				if (selected_node->connect_to[i] == &node)
					selected_node->connect_to.erase(selected_node->connect_to.begin() + i);
			}

			for (u32 i = 0; i < node.connect_to.size(); i++) {
				if (node.connect_to[i] == selected_node)
					node.connect_to.erase(node.connect_to.begin() + i);
			}

			selected_node->parent->update(++updategen);
			node.parent->update(++updategen);

			return;
		}

		selected_node->connect_to.push_back(&node);
		node.connect_to.push_back(selected_node);

		printf("Conn made (%p <-> %p)\n", selected_node, &node);

		if (selected_node->parent != node.parent) {
			selected_node->parent->update(++updategen);
			node.parent->update(++updategen);
		}
	}

	// Összes csatlakozés törlése egy node-on
	if (ImGui::IsItemHovered(ImGuiHoveredFlags_AllowWhenBlockedByActiveItem) && ImGui::IsKeyDown(ImGuiKey_Delete)) {
		for (Node* n : node.connect_to) {
			n->connect_to.erase(std::remove(n->connect_to.begin(), n->connect_to.end(), &node), n->connect_to.end());
			node.connect_to.erase(std::remove(node.connect_to.begin(), node.connect_to.end(), n), node.connect_to.end());
			
			n->parent->update(++updategen);
			node.parent->update(++updategen);
		}
	}

	if (!node.input) {
		if (node.connect_to.size()) {
			for (Node* node2 : node.connect_to) {
				draw_list->AddLine(
					mc.convert(pos + node.pos),
					mc.convert(node2->parent->pos + node2->pos),
					ns_colors[node.state],
					wireth
				);
			}
		}
	}
}

Component::Component(ImVec2 _pos, Comps _type): pos(_pos), type(_type) {
	gen_ids(false);

	for (NodeDef& def : compnodes[type]) {
		if (def.inp) {
			ins.push_back(Node(def.relpos, this, true));
		} else {
			outs.push_back(Node(def.relpos, this, false));
		}
	}

	// Állapot inicializálása
	switch (type) {
		case Comps::INPUT: {
			state.INPUT.value = 0;
			break;
		}

		default: break;
	}

	// Output node-ok inicializálása
	update(++updategen);
}

void Component::draw(ImDrawList* draw_list) {
	for (Node& node : ins) {
		process_node(draw_list, pos, node);
	}

	for (Node& node : outs) {
		process_node(draw_list, pos, node);
	}

	const ImColor c = selected ? red : white;

	ImGui::SetCursorPos(mc.convert(pos));
	static ImVec2 sum;
	ImGui::InvisibleButton(id, compdims[type] * GRID_SPACING_ZOOM);
	if (ImGui::IsItemActive()) {
		sum += ImGui::GetIO().MouseDelta / mc.zoom;

		// Ha van kijelölés akkor az
		// összes jelölt komponens (benne van ez is)
		// menjen egyszerre, ha nincs csak ez
		bool sel = false;
		ImVec2 trunced = ImVec2(std::trunc(sum.x / GRID_SPACING), std::trunc(sum.y / GRID_SPACING));
		for (Component* c : comps) {
			if (c->selected) {
				sel = true;
				c->pos += trunced;
			}
		}
		if (sel) sum -= trunced*GRID_SPACING;
		if (!sel) {
			ImVec2 trunced = ImVec2(std::trunc(sum.x / GRID_SPACING), std::trunc(sum.y / GRID_SPACING));
			pos += trunced;
			sum -= trunced*GRID_SPACING;
		}
	}

	if (ImGui::IsItemHovered() && ImGui::IsKeyPressed(ImGuiKey_E)) {
		if (type == Comps::INPUT) {
			state.INPUT.value = !state.INPUT.value;
			update(++updategen);
		}
	}

	(compdraw[type])(draw_list, pos, c, this);
}

Component::~Component() {  }

void Component::update(u8 upd) {
	if (updated == upd) {
		// Már update-elve van, tehát nincs teendő
		printf("Frissítve van már\n");
		return;
	}
	updated++;

	for (Node& i : ins) {
		// Input node frissítése, konfliktusok keresése
		NodeState s = NodeState::NS_U;
		for (Node* o : i.connect_to) {
			if (o->state == NodeState::NS_U) continue;
			if (s == NodeState::NS_U) {
				s = o->state;
				continue;
			} else {
				if (o->state != s) {
					s = NodeState::NS_E;
					goto nextin;
				} else {
					continue;
				}
			}
		}
	nextin:
		i.state = s;
	}

	NodeState out;
	bool invert = false;
	switch (type) {
		case NAND_GATE:
			invert = true;
		case AND_GATE: {
			out = NodeState::NS_1;

			for (Node& i : ins) {
				switch (i.state) {
					case NodeState::NS_E:
					case NodeState::NS_U: {
						out = NodeState::NS_U;
						goto nextupdate;
					}
					case NodeState::NS_0: {
						out = NodeState::NS_0;
						continue;
					}
					case NodeState::NS_1:
						continue;
				}
			}
			break;
		}

		case INPUT: {
			if (state.INPUT.value & 1)
				out = NodeState::NS_1;
			else
				out = NodeState::NS_0;
			break;
		}

		default: {
			printf("Nincs impl: %s\n", comptypes[type]);
			break;
		}
	}

nextupdate:
	if (invert) {
		if (out == NodeState::NS_0)
			out = NodeState::NS_1;
		else if (out == NodeState::NS_1)
			out = NodeState::NS_0;
	}
	outs[0].state = out;

	// Az outs[0]-n lévő elemek frissítése
	for (Component* c : comps) {
		for (Node& n : c->ins) {
			for (u32 i = 0; i < n.connect_to.size(); i++) {
				if (n.connect_to[i]->parent == this) {
					c->update(upd);
				}
			}
		}
	}

	updated = true;
}

void Component::gen_ids(bool regen_nodes) {
	sprintf(id, "comp%d", counter);
	sprintf(context_menu_id, "cm%d", counter);
	counter++;

	if (regen_nodes) {
		for (Node& n : ins) {
			sprintf(n.id, "n%d", counter);
			counter++;
		}

		for (Node& n : outs) {
			sprintf(n.id, "n%d", counter);
			counter++;
		}
	}
}

Node::Node(ImVec2 _pos, Component* _parent, bool in): pos(_pos), connect_to(0), parent(_parent), input(in) {
	sprintf(id, "n%d", counter);
	counter++;
}

// Ez azért kell, hogy az utalásokat erre a node-ra
// eltávolítsuk más node-okból
Node::~Node() {
	for (Node* n : connect_to) {
		n->connect_to.erase(std::remove(n->connect_to.begin(), n->connect_to.end(), this), n->connect_to.end());
	}
}
