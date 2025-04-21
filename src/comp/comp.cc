#include <iostream>
#include <algorithm>

#include "../types.hh"
#include "comp.hh"
#include "compdefs.hh"
#include "input.hh"

static u32 counter;
Node* selected_node;

ImU32 ns_colors[4] = {
	/* [NS_0] = */ IM_COL32(255, 76, 60, 255),
	/* [NS_1] = */ IM_COL32(0, 255, 255, 255),
	/* [NS_U] = */ IM_COL32(200, 200, 200, 255),
	/* [NS_E] = */ IM_COL32(200, 0, 0, 255),
};

static inline void process_node(ImDrawList* draw_list, ImVec2& pos, Node& node) {
	const ImColor c = node.parent->selected ? red : ns_colors[node.state];

	draw_list->AddCircleFilled(ImVec2(pos.x + node.pos.x + th/2, pos.y + node.pos.y), noderad, c);
	ImGui::SetCursorPos(ImVec2(pos.x + node.pos.x - hitbox/2, pos.y + node.pos.y - hitbox/2));
	ImGui::InvisibleButton(node.id, ImVec2(hitbox, hitbox));
	ImGui::GetItemRectMax();

	if (ImGui::IsItemActive()) {
		draw_list->AddLine(ImVec2(pos.x + node.pos.x + th/2, pos.y + node.pos.y), ImGui::GetMousePos(), ns_colors[node.state], wireth);
		selected_node = &node;
	}

	if (ImGui::IsItemHovered(ImGuiHoveredFlags_AllowWhenBlockedByActiveItem) && ImGui::GetIO().MouseReleased[0]) {
		// node: ez a node a kurzor alatt
		// selected: ahonnan húzom a kábelt
		if (&node == selected_node) { printf("Saját magába csatlakozás\n"); return; }
		if (node.input == selected_node->input) { printf("Érvénytelen csatlakozás!\n"); return; }

		// Ez se normális
		if (std::find(selected_node->connect_to.begin(), selected_node->connect_to.end(), &node) != selected_node->connect_to.end()) {
			printf("Már van conn\n");
			return;
		}

		selected_node->connect_to.push_back(&node);
		node.connect_to.push_back(selected_node);

		printf("Conn made (%p <-> %p)\n", selected_node, &node);
	}

	// Összes csatlakozés törlése egy node-on
	if (ImGui::IsItemHovered(ImGuiHoveredFlags_AllowWhenBlockedByActiveItem) && ImGui::IsKeyDown(ImGuiKey_Delete)) {
		for (Node* n : node.connect_to) {
			n->connect_to.erase(std::remove(n->connect_to.begin(), n->connect_to.end(), &node), n->connect_to.end());
			node.connect_to.erase(std::remove(node.connect_to.begin(), node.connect_to.end(), n), node.connect_to.end());
		}
	}

	if (!node.input) {
		if (node.connect_to.size()) {
			for (Node* node2 : node.connect_to) {
				draw_list->AddLine(
					ImVec2(pos.x + node.pos.x + th/2, pos.y + node.pos.y),
					ImVec2(node2->parent->pos.x + node2->pos.x + th/2, node2->parent->pos.y + node2->pos.y),
					ns_colors[node.state],
					wireth
				);
			}
		}
	}
}

Component::Component(ImVec2 _pos, Comps _type): pos(_pos), type(_type) {
	sprintf(id, "comp%d", counter);
	sprintf(context_menu_id, "cm%d", counter);
	counter++;

	ins.push_back(Node(ImVec2(0, compdims[type].y * 0.25), this, true));
	ins.push_back(Node(ImVec2(0, compdims[type].y * 0.75), this, true));

	outs.push_back(Node(ImVec2(compdims[type].x, compdims[type].y * 0.50), this, false));
}

void Component::draw(ImDrawList* draw_list) {
	for (Node& node : ins) {
		process_node(draw_list, pos, node);
	}

	for (Node& node : outs) {
		process_node(draw_list, pos, node);
	}

	const ImColor c = selected ? red : white;

	ImGui::SetCursorPos(pos);
	ImGui::InvisibleButton(id, compdims[type]);
	if (ImGui::IsItemActive()) {
		// Ha van kijelölés akkor az
		// összes jelölt komponens (benne van ez is)
		// menjen egyszerre, ha nincs csak ez
		bool sel = false;
		for (Component* c : comps) {
			if (c->selected) {
				sel = true;
				c->pos.x += ImGui::GetIO().MouseDelta.x;
				c->pos.y += ImGui::GetIO().MouseDelta.y;
			}
		}
		if (!sel) {
			pos.x += ImGui::GetIO().MouseDelta.x;
			pos.y += ImGui::GetIO().MouseDelta.y;
		}
	}

	if (ImGui::IsItemHovered() && ImGui::IsKeyPressed(ImGuiKey_E)) {
		if (type == Comps::INPUT) {
			printf("yo %d %d\n", ((Input*)this)->value, outs[0].state);
			((Input*)this)->value = !((Input*)this)->value;
			update();
		}
	}

	(compdraw[type])(draw_list, pos, c);
}

Component::~Component() {  }

void Component::update() {
	printf("frissít %s\n", comptypes[type]);

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
	if (type == AND_GATE) {
		out = NodeState::NS_1;

		for (Node& i : ins) {
			printf("i.state %d @ %s\n", i.state, comptypes[i.parent->type]);
			switch (i.state) {
				case NodeState::NS_E:
				case NodeState::NS_U: {
					out = NodeState::NS_U;
					goto nextupdate;
				}
				case NodeState::NS_0: {
					out = NodeState::NS_0;
					goto nextupdate;
				}
				case NodeState::NS_1:
					continue;
			}
		}
	} else if (type == INPUT) {
		if (((Input*)this)->value & 1)
			out = NodeState::NS_1;
		else
			out = NodeState::NS_0;
	}

nextupdate:
	outs[0].state = out;

	// Az outs[0]-n lévő elemek frissítése
	for (Component* c : comps) {
		for (Node& n : c->ins) {
			for (u32 i = 0; i < n.connect_to.size(); i++) {
				if (n.connect_to[i]->parent == this) {
					c->update();
				}
			}
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
