#include "util.hh"

std::vector<Component*> comps;
Selection sel;
View view { .zoom = 1.0f };

ImVec2 transform(ImVec2 in) {
	in += view.panpos;
	return in;
}

ImVec2 zoom(ImVec2 original) {
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return original*view.zoom - center*view.zoom + center;
	// return original * view.zoom;
}

f32 zoomx(f32 original) {
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return original*view.zoom - center.x*view.zoom + center.x;
	// return original * view.zoom;
}

f32 zoomy(f32 original) {
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return original*view.zoom - center.y*view.zoom + center.y;
	// return original * view.zoom;
}

#include <iostream>

ImVec2 unzoom(ImVec2 zoomed) {
	// TODO: GetWindowPos nem jó mert szar
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return zoomed/view.zoom + center - center/view.zoom;
	// return zoomed / view.zoom;
}
