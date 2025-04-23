#include "compdefs.hh"
#include "comp.hh"

#include <iostream>

const char* comptypes[] = {
	COMPTYPELIST
};

void draw_and(PARAMS) {
	draw_list->AddBezierCubic(
		ImVec2(pos.x + 20, pos.y + 0.5),
		ImVec2(pos.x + 100, pos.y + 0.5),
		ImVec2(pos.x + 100, pos.y + 0.5 + 100),
		ImVec2(pos.x + 20, pos.y + 0.5 + 100),
		c,
		th + 0.2
	);
	draw_list->AddLine(
		ImVec2(pos.x, pos.y),
		ImVec2(pos.x + 20, pos.y),
		c,
		th
	);
	draw_list->AddLine(
		ImVec2(pos.x, pos.y + 100),
		ImVec2(pos.x + 20, pos.y + 100),
		c,
		th
	);
	draw_list->AddLine(
		ImVec2(pos.x, pos.y),
		ImVec2(pos.x, pos.y + 100),
		c,
		th
	);
}

void draw_nand(PARAMS) {
	draw_and(draw_list, pos, c);
	draw_list->AddCircle(ImVec2(pos.x + 79 + 7, pos.y + 50), ballw / 2, c, 0, th);
}

void draw_nor(PARAMS) {
	printf("Nincs impl.: draw_nor\n");
}

void draw_or(PARAMS) {
	printf("Nincs impl.: draw_or\n");
}

void draw_xnor(PARAMS) {
	printf("Nincs impl.: draw_xnor\n");
}

void draw_xor(PARAMS) {
	printf("Nincs impl.: draw_xor\n");
}

void draw_input(PARAMS) {
	draw_list->AddQuad(pos, ImVec2(pos.x + 50, pos.y), ImVec2(pos.x + 50, pos.y + 50), ImVec2(pos.x, pos.y + 50), c, th);
}

void (*compdraw[])(PARAMS) = {
	/* AND_GATE  */ draw_and,
	/* NAND_GATE */ draw_nand,
	/* OR_GATE   */ draw_or,
	/* NOR_GATE  */ draw_nor,
	/* XOR_GATE  */ draw_xor,
	/* XNOR_GATE */ draw_xnor,
	/* INPUT     */ draw_input,
};

ImVec2 compdims[] = {
	/* AND_GATE  */ ImVec2(gatebasew, 						gatebaseh),
	/* NAND_GATE */ ImVec2(gatebasew + ballw + noderad - 1,	gatebaseh), // Hosszabb a kis karika miatt
	/* OR_GATE   */ ImVec2(gatebasew, 						gatebaseh),
	/* NOR_GATE  */ ImVec2(gatebasew + ballw + noderad - 1,	gatebaseh), // Hosszabb a kis karika miatt
	/* XOR_GATE  */ ImVec2(gatebasew, 						gatebaseh),
	/* XNOR_GATE */ ImVec2(gatebasew + ballw + noderad - 1,	gatebaseh), // Hosszabb a kis karika miatt
	/* INPUT     */ ImVec2(50,								50),
};

NodeDef::NodeDef(ImVec2 _relpos, bool _inp): relpos(_relpos), inp(_inp) {  }

std::list<NodeDef> compnodes[] = {
	/* AND_GATE  */ { NodeDef(ImVec2(0, 0.25), true), NodeDef(ImVec2(0, 0.75), true), NodeDef(ImVec2(1.0, 0.50), false), },
	/* NAND_GATE */ { NodeDef(ImVec2(0, 0.25), true), NodeDef(ImVec2(0, 0.75), true), NodeDef(ImVec2(1.0, 0.50), false), },
	/* OR_GATE   */ { NodeDef(ImVec2(0, 0.25), true), NodeDef(ImVec2(0, 0.75), true), NodeDef(ImVec2(1.0, 0.50), false), },
	/* NOR_GATE  */ { NodeDef(ImVec2(0, 0.25), true), NodeDef(ImVec2(0, 0.75), true), NodeDef(ImVec2(1.0, 0.50), false), },
	/* XOR_GATE  */ { NodeDef(ImVec2(0, 0.25), true), NodeDef(ImVec2(0, 0.75), true), NodeDef(ImVec2(1.0, 0.50), false), },
	/* XNOR_GATE */ { NodeDef(ImVec2(0, 0.25), true), NodeDef(ImVec2(0, 0.75), true), NodeDef(ImVec2(1.0, 0.50), false), },
	/* INPUT     */ { NodeDef(ImVec2(1, 0.50), false) },
};
