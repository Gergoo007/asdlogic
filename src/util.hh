#pragma once

#include <imgui/imgui.h>
#include <types.hh>
#include <vector>

struct Selection {
	bool active; // true amíg a felhasználó húzza/tartja a kijelölést
	ImVec2 from; // Innen kezdte húzni
	ImVec2 to; // Ide húzta
};

struct View {
	ImVec2 panpos;
	f32 zoom;
};

extern Selection sel;

struct Component;
extern std::vector<Component*> comps;

// Via https://github.com/phicore/ImGuiStylingTricks
void CenteredText(const char* label, const ImVec2& size_arg);

struct Clipboard {
	private:
	std::vector<Component*> clipboard;

	public:
	ImVec2 copymousepos;
	void copy();
	void paste();
	void cut();
};

extern Clipboard clipboard;
