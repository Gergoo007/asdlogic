#include "compdefs.hh"
#include "comp.hh"

#include <iostream>

#include <util.hh>
#include <canvas.hh>
#include <config.hh>

const char* comptypes[] = {
	COMPTYPELIST
};

const f32 curveat = 1.75;
const f32 curvelen = 3;
const f32 curveyoffs = 0.001;

void draw_and(PARAMS) {
	draw_list->AddBezierCubic(
		mc.convert(ImVec2(pos.x + curveat, pos.y + curveyoffs)),
		mc.convert(ImVec2(pos.x + curveat + curvelen, pos.y)),
		mc.convert(ImVec2(pos.x + curveat + curvelen, pos.y + gatebaseh)),
		mc.convert(ImVec2(pos.x + curveat, pos.y + curveyoffs + gatebaseh)),
		c,
		linew + 0.2
	);
	draw_list->AddLine(
		mc.convert(ImVec2(pos.x, pos.y)),
		mc.convert(ImVec2(pos.x + curveat, pos.y)),
		c,
		linew
	);
	draw_list->AddLine(
		mc.convert(ImVec2(pos.x, pos.y + gatebaseh)),
		mc.convert(ImVec2(pos.x + curveat, pos.y + gatebaseh)),
		c,
		linew
	);
	draw_list->AddLine(
		mc.convert(ImVec2(pos.x, pos.y)),
		mc.convert(ImVec2(pos.x, pos.y + gatebaseh)),
		c,
		linew
	);
}

void draw_nand(PARAMS) {
	draw_and(draw_list, pos, c, _comp);
	const f32 _rad = (f32)GRID_SPACING;
	draw_list->AddCircle(mc.convert(ImVec2(pos.x + curveat + curvelen - 0.25, pos.y + (f32)gatebaseh / 2)), _rad * mc.zoom / 2, c, 0, linew);
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
	draw_list->AddQuad(mc.convert(pos), mc.convert(ImVec2(pos.x + 50, pos.y)), mc.convert(ImVec2(pos.x + 50, pos.y + 50)), mc.convert(ImVec2(pos.x, pos.y + 50)), c, linew);
	ImGui::SetCursorPos(mc.convert(ImVec2(pos.x, pos.y)));
	CenteredText(_comp->state.INPUT.value ? "1" : "0", ImVec2(50, 50) * mc.zoom);
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
	/* INPUT     */ ImVec2(2,								2),
};

NodeDef::NodeDef(ImVec2 _relpos, bool _inp): relpos(_relpos), inp(_inp) {  }

std::list<NodeDef> compnodes[] = {
	/* AND_GATE  */ { NodeDef(ImVec2(0, 1), true), NodeDef(ImVec2(0, 3), true), NodeDef(ImVec2(4, 2), false), },
	/* NAND_GATE */ { NodeDef(ImVec2(0, 1), true), NodeDef(ImVec2(0, 3), true), NodeDef(ImVec2(5, 2), false), },
	/* OR_GATE   */ { NodeDef(ImVec2(0, 1), true), NodeDef(ImVec2(0, 3), true), NodeDef(ImVec2(4, 2), false), },
	/* NOR_GATE  */ { NodeDef(ImVec2(0, 1), true), NodeDef(ImVec2(0, 3), true), NodeDef(ImVec2(4, 2), false), },
	/* XOR_GATE  */ { NodeDef(ImVec2(0, 1), true), NodeDef(ImVec2(0, 3), true), NodeDef(ImVec2(4, 2), false), },
	/* XNOR_GATE */ { NodeDef(ImVec2(0, 1), true), NodeDef(ImVec2(0, 3), true), NodeDef(ImVec2(4, 2), false), },
	/* INPUT     */ { NodeDef(ImVec2(1, 2), false) },
};
