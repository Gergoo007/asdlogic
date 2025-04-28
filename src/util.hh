#pragma once

#include "imgui/imgui.h"
#include "types.hh"
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

extern View view;

ImVec2 transform(ImVec2 in);
ImVec2 zoom(ImVec2 original);
f32 zoomx(f32 original);
f32 zoomy(f32 original);
ImVec2 unzoom(ImVec2 original);
