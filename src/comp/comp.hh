#pragma once

#include "../imgui/imgui.h"
#include "compdefs.hh"
#include "../types.hh"

#include <vector>

struct Connection {
	u32 startcompid;
	u32 startcompoutput;
	u32 endcompid;
	u32 endcompoutput;

	std::vector<ImVec2> corners;
};

enum NodeState {
	// o/
	NS_0 = 0,
	NS_1 = 1,
	NS_U = 2,
	NS_E = 3,
};

extern ImU32 ns_colors[];

struct Component;
struct Node {
	ImVec2 pos;
	char id[8];

	// true: bemenet; false: kimenet
	bool input;

	// állapot
	// 0: 0
	// 1: 1
	// 2: határozatlan
	// 3: hiba
	NodeState state = NodeState::NS_U;

	std::vector<Node*> connect_to;
	Component* parent;

	Node(ImVec2 _pos, Component* _parent, bool in);
	~Node();
};

struct Component {
	ImVec2 pos;
	char id[64];
	char context_menu_id[64];

	f32 scale;

	std::vector<Node> ins, outs;

	bool selected = false;

	Comps type;

	void draw(ImDrawList* draw_list);
	void update();

	Component(ImVec2 pos, Comps _type);
	~Component();
};
