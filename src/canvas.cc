#include <canvas.hh>
#include <cmath>
#include <config.hh>

Canvas mc { .zoom = 1.0f, .panpos = ImVec2(0, 0) };

// ImVec2 Canvas::transform(ImVec2 in) {
// 	return in + panpos;
// }

// ImVec2 Canvas::zoomfun(ImVec2 original) {
// 	ImVec2 center = (ImVec2(1280, 720) / 2);
// 	return original*zoom - center*zoom + center;
// }

f32 Canvas::zoomx(f32 original) {
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return original*zoom - center.x*zoom + center.x;
}

f32 Canvas::zoomy(f32 original) {
	ImVec2 center = (ImVec2(1280, 720) / 2);
	return original*zoom - center.y*zoom + center.y;
}

// ImVec2 Canvas::unzoom(ImVec2 zoomed) {
// 	// TODO: GetWindowPos nem jó mert szar
// 	ImVec2 center = (ImVec2(1280, 720) / 2);
// 	return zoomed/zoom + center - center/zoom;
// }

void Canvas::drawGrid(ImDrawList* dl) {
	ImVec2 screen = ImVec2(1280, 720);
	ImVec2 center = (screen) / 2;

	// dl->AddLine(ImVec2(center.x, 0), ImVec2(center.x, screen.y), 0xffffffff, 2 * zoom);
	// dl->AddLine(ImVec2(0, center.y), ImVec2(screen.x, center.y), 0xffffffff, 2 * zoom);

	f32 remainderx = fmod(((center.x / GRID_SPACING_ZOOM) - trunc(center.x / GRID_SPACING_ZOOM)) * GRID_SPACING_ZOOM
						+ fmod(panpos.x * zoom, GRID_SPACING_ZOOM), GRID_SPACING_ZOOM);
	f32 remaindery = fmod(((center.y / GRID_SPACING_ZOOM) - trunc(center.y / GRID_SPACING_ZOOM)) * GRID_SPACING_ZOOM
						+ fmod(panpos.y * zoom, GRID_SPACING_ZOOM), GRID_SPACING_ZOOM);
	if (remainderx < 0) remainderx += GRID_SPACING_ZOOM;
	if (remaindery < 0) remaindery += GRID_SPACING_ZOOM;

	for (f32 y = remaindery; y < screen.y; y += GRID_SPACING_ZOOM) {
		for (f32 x = remainderx; x < screen.x; x += GRID_SPACING_ZOOM) {
			dl->AddCircleFilled(ImVec2(x, y), GRID_RAD * zoom, 0xff3a3b3c);
		}
	}

	// dl->AddCircleFilled(ImVec2(remainderx, remaindery), 2 * zoom, 0xffffffff);
}

// Képernyő koordinátából canvas koordináta
ImVec2 Canvas::snap(ImVec2 coord) {
	const ImVec2 screen = ImVec2(1280, 720);
	const ImVec2 center = (screen) / 2;

	// Ennyi pixellel van elcsúszva a grid, tehát a pontok amikhez a koordinátát snappelni kell
	const f32 panoffsetx = fmod(panpos.x, GRID_SPACING) * zoom;
	const f32 panoffsety = fmod(panpos.y, GRID_SPACING) * zoom;

	// Ennyi grid koordinátával van elcsúsztatva a canvas
	const f32 pancoordx = std::trunc(panpos.x / GRID_SPACING);
	const f32 pancoordy = std::trunc(panpos.y / GRID_SPACING);
	// printf("---------------\n%f %f\n%f %f\n%f %f\n---------------\n", panpos.x, panpos.y, pancoordx, pancoordy, panoffsetx, panoffsety);

	f32 remainderx = fmod(((center.x / GRID_SPACING_ZOOM) - truncf(center.x / GRID_SPACING_ZOOM)) * GRID_SPACING_ZOOM
						+ fmodf(panpos.x * zoom, GRID_SPACING_ZOOM), GRID_SPACING_ZOOM);
	f32 remaindery = fmod(((center.y / GRID_SPACING_ZOOM) - truncf(center.y / GRID_SPACING_ZOOM)) * GRID_SPACING_ZOOM
						+ fmodf(panpos.y * zoom, GRID_SPACING_ZOOM), GRID_SPACING_ZOOM);
	if (remainderx < 0) remainderx += GRID_SPACING_ZOOM;
	if (remaindery < 0) remaindery += GRID_SPACING_ZOOM;

	coord = coord - center;

	// printf("pc %f %f\n", pancoordx, pancoordy);

	return ImVec2(
		std::round((coord.x - panoffsetx) / GRID_SPACING_ZOOM) - pancoordx,
		std::round((coord.y - panoffsety) / GRID_SPACING_ZOOM) - pancoordy 
	);
}

// Canvas koordinátából képernyő koordináta
ImVec2 Canvas::convert(ImVec2 gridpos) {
	const ImVec2 screen = ImVec2(1280, 720);
	const ImVec2 center = (screen) / 2;

	// Ennyivel van elcsúszva a grid, tehát a pontok amikhez a koordinátát snappelni kell
	const f32 panoffsetx = fmodf(panpos.x, GRID_SPACING);
	const f32 panoffsety = fmodf(panpos.y, GRID_SPACING);

	// Ennyi grid koordinátával van elcsúsztatva a canvas
	const f32 pancoordx = panpos.x / GRID_SPACING;
	const f32 pancoordy = panpos.y / GRID_SPACING;

	// Ha a képernyő mérete nem osztható GRID_SPACING-el
	const f32 gridoffsetx = fmodf(screen.x, GRID_SPACING);
	const f32 gridoffsety = fmodf(screen.y, GRID_SPACING);

	f32 remainderx = fmod(((center.x / GRID_SPACING_ZOOM) - truncf(center.x / GRID_SPACING_ZOOM)) * GRID_SPACING_ZOOM
						+ fmodf(panpos.x * zoom, GRID_SPACING_ZOOM), GRID_SPACING_ZOOM);
	f32 remaindery = fmod(((center.y / GRID_SPACING_ZOOM) - truncf(center.y / GRID_SPACING_ZOOM)) * GRID_SPACING_ZOOM
						+ fmodf(panpos.y * zoom, GRID_SPACING_ZOOM), GRID_SPACING_ZOOM);
	if (remainderx < 0) remainderx += GRID_SPACING_ZOOM;
	if (remaindery < 0) remaindery += GRID_SPACING_ZOOM;

	return (gridpos + ImVec2(pancoordx, pancoordy)) * GRID_SPACING_ZOOM + center;
}
