#include <canvas.hh>
#include <cmath>
#include <config.hh>

Canvas maincanvas { .zoom = 1.0f, .panpos = ImVec2(0, 0) };

ImVec2 Canvas::transform(ImVec2 in) {
	in += panpos;
	return in;
}

ImVec2 Canvas::zoomfun(ImVec2 original) {
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return original*zoom - center*zoom + center;
}

f32 Canvas::zoomx(f32 original) {
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return original*zoom - center.x*zoom + center.x;
}

f32 Canvas::zoomy(f32 original) {
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return original*zoom - center.y*zoom + center.y;
}

ImVec2 Canvas::unzoom(ImVec2 zoomed) {
	// TODO: GetWindowPos nem jó mert szar
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return zoomed/zoom + center - center/zoom;
}

void Canvas::drawGrid(ImDrawList* dl) {
	ImVec2 screen = ImVec2(1280, 720);
	ImVec2 center = (screen) / 2;

	// dl->AddLine(ImVec2(center2.x, 0), ImVec2(center2.x, screen.y), 0xffffffff, 1 * maincanvas.zoom);
	// dl->AddLine(ImVec2(0, center2.y), ImVec2(screen.x, center2.y), 0xffffffff, 1 * maincanvas.zoom);

	const f32 GRID_SPACING_ZOOM = GRID_SPACING * maincanvas.zoom;
	const f32 remainderx = ((center.x / GRID_SPACING_ZOOM) - truncf(center.x / GRID_SPACING_ZOOM)) * GRID_SPACING_ZOOM
							+ fmodf(maincanvas.panpos.x * maincanvas.zoom, GRID_SPACING_ZOOM);
	const f32 remaindery = ((center.y / GRID_SPACING_ZOOM) - truncf(center.y / GRID_SPACING_ZOOM)) * GRID_SPACING_ZOOM
							+ fmodf(maincanvas.panpos.y * maincanvas.zoom, GRID_SPACING_ZOOM);

	for (f32 y = remaindery; y < screen.y; y += GRID_SPACING_ZOOM) {
		for (f32 x = remainderx; x < screen.x; x += GRID_SPACING_ZOOM) {
			dl->AddCircleFilled(ImVec2(x, y), 1 * maincanvas.zoom, 0xff3a3b3c);
		}
	}
}
