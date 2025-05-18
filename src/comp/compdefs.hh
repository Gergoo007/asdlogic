#pragma once

#include <imgui/imgui.h>
#include <types.hh>

#include <list>

struct Component;
#define PARAMS ImDrawList* draw_list, ImVec2 pos, ImU32 c, Component* _comp

#define noderad ((f32)4)
#define hitbox ((f32)12)
#define wireth ((f32)2)
#define white IM_COL32(200, 200, 200, 255)
#define red IM_COL32(200, 0, 0, 255)
#define ballw ((f32)14) // Inverter karika
#define linew ((f32)1.2)

#define gatebasew 8
#define gatebaseh 5

#define COMPTYPELIST \
	ADD_COMPTYPE(AND_GATE) \
	ADD_COMPTYPE(NAND_GATE) \
	ADD_COMPTYPE(OR_GATE) \
	ADD_COMPTYPE(NOR_GATE) \
	ADD_COMPTYPE(XOR_GATE) \
	ADD_COMPTYPE(XNOR_GATE) \
	ADD_COMPTYPE(INPUT)

#define ADD_COMPTYPE(x) x,

enum Comps {
	COMPTYPELIST
};

#undef ADD_COMPTYPE
#define ADD_COMPTYPE(x) #x,

#define DEFINED_COMPS 7

struct NodeDef {
	ImVec2 relpos;
	bool inp;

	NodeDef(ImVec2 _relpos, bool _inp);
};

extern void (*compdraw[])(PARAMS);
extern ImVec2 compdims[];
extern const char* comptypes[];
extern std::list<NodeDef> compnodes[];
