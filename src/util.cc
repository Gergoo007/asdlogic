#include <util.hh>
#include <imgui/imgui_internal.h>

#include <comp/comp.hh>

std::vector<Component*> comps;
Clipboard clipboard;
Selection sel;

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

void Clipboard::copy() {
	copymousepos = ImGui::GetMousePos();
	for (Component* c : comps) {
		if (c->selected) {
			clipboard.push_back(c);
		}
	}
}

void Clipboard::cut() {
	for (u32 i = 0; i < comps.size(); i++) {
		if (!comps[i]->selected) continue;
		clipboard.push_back(comps[i]);

		delete comps[i];
		// ezt is ki a faszom találta ki?
		std::vector<Component*>::iterator it = comps.begin();
		std::advance(it, i);
		comps.erase(it);
	}
}

void Clipboard::paste() {
	
}
