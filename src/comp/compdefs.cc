#include "compdefs.hh"
#include "comp.hh"

#include <iostream>

#include "../imgui/imgui_internal.h"

#include "../util.hh"

// Via https://github.com/phicore/ImGuiStylingTricks
void CenteredText(const char* label, const ImVec2& size_arg) {
	ImGuiWindow* window = ImGui::GetCurrentWindow();
	
	ImGuiContext& g = *GImGui;
	const ImGuiStyle& style = g.Style;
	const ImGuiID id = window->GetID(label);
	const ImVec2 label_size = ImGui::CalcTextSize(label, NULL, true);

	ImVec2 pos = window->DC.CursorPos;
	ImVec2 size = ImGui::CalcItemSize(size_arg, label_size.x + style.FramePadding.x * 2.0f, label_size.y + style.FramePadding.y * 2.0f);

	const ImVec2 pos2 = ImVec2((pos.x + size.x), (pos.y + size.y));
	const ImRect bb(pos, pos2);

	ImGui::ItemSize(size, style.FramePadding.y);

	const ImVec2 pos_min = ImVec2((bb.Min.x + style.FramePadding.x), (bb.Min.y + style.FramePadding.y));
	const ImVec2 pos_max = ImVec2((bb.Max.x - style.FramePadding.x), (bb.Max.y - style.FramePadding.y));

	ImGui::RenderTextClipped(pos_min, pos_max, label, NULL, &label_size, style.ButtonTextAlign, &bb);
}

const char* comptypes[] = {
	COMPTYPELIST
};

void draw_and(PARAMS) {
	draw_list->AddBezierCubic(
		zoom(ImVec2(pos.x + 20, pos.y + 0.5)),
		zoom(ImVec2(pos.x + 100, pos.y + 0.5)),
		zoom(ImVec2(pos.x + 100, pos.y + 0.5 + 100)),
		zoom(ImVec2(pos.x + 20, pos.y + 0.5 + 100)),
		c,
		linew + 0.2
	);
	draw_list->AddLine(
		zoom(ImVec2(pos.x, pos.y)),
		zoom(ImVec2(pos.x + 20, pos.y)),
		c,
		linew
	);
	draw_list->AddLine(
		zoom(ImVec2(pos.x, pos.y + 100)),
		zoom(ImVec2(pos.x + 20, pos.y + 100)),
		c,
		linew
	);
	draw_list->AddLine(
		zoom(ImVec2(pos.x, pos.y)),
		zoom(ImVec2(pos.x, pos.y + 100)),
		c,
		linew
	);
}

void draw_nand(PARAMS) {
	draw_and(draw_list, pos, c, _comp);
	draw_list->AddCircle(ImVec2(pos.x + 79 + 7, pos.y + 50) * view.zoom, ballw / 2 * view.zoom, c, 0, linew);
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
	draw_list->AddQuad(zoom(pos), zoom(ImVec2(pos.x + 50, pos.y)), zoom(ImVec2(pos.x + 50, pos.y + 50)), zoom(ImVec2(pos.x, pos.y + 50)), c, linew);
	ImGui::SetCursorPos(zoom(ImVec2(pos.x, pos.y)));
	CenteredText(_comp->state.INPUT.value ? "1" : "0", ImVec2(50, 50) * view.zoom);
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
