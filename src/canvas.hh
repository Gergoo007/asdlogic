#pragma once

#include <types.hh>

struct Canvas {
	f32 zoom;
	ImVec2 panpos;

	// ImVec2 transform(ImVec2 in);
	// ImVec2 zoomfun(ImVec2 original);
	f32 zoomx(f32 original);
	f32 zoomy(f32 original);
	// ImVec2 unzoom(ImVec2 original);
	
	void drawGrid(ImDrawList* dl);
	ImVec2 snap(ImVec2 coord);
	ImVec2 convert(ImVec2 o);
};

extern Canvas mc;
