#include <GLFW/glfw3.h>

#include <iostream>
#include <string>
#include <fstream>
#include <sstream>
#include <vector>
#include <algorithm>

#include <colors.hh>
#include <comp/comp.hh>
#include <canvas.hh>

#include <imgui/imgui.h>
#include <imgui/imgui_impl_glfw.h>
#include <imgui/imgui_impl_opengl3.h>

#include <util.hh>

#include <cmath>

#define min(a, b) ((a) > (b) ? (b) : (a))
#define max(a, b) ((a) < (b) ? (b) : (a))

int main() {
	if (!glfwInit())
		return 1;

	GLFWwindow* window = glfwCreateWindow(1280, 720, "Hello World", NULL, NULL);
	if (!window) {
		glfwTerminate();
		return 1;
	}
	glfwMakeContextCurrent(window);
	glfwSwapInterval(0);

	// Setup Dear ImGui context
	IMGUI_CHECKVERSION();
	ImGui::CreateContext();
	ImGuiIO& io = ImGui::GetIO();
	io.ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard;
	io.ConfigFlags |= ImGuiConfigFlags_DockingEnable;

	// Setup Platform/Renderer backends
	ImGui_ImplGlfw_InitForOpenGL(window, true);
	ImGui_ImplOpenGL3_Init();

	ImFont* font1 = io.Fonts->AddFontFromFileTTF("arial.ttf", 20);

	while (!glfwWindowShouldClose(window)) {
		double start = glfwGetTime();

		glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
		glClearColor(BG_COLOR, 1);

		ImGui_ImplOpenGL3_NewFrame();
		ImGui_ImplGlfw_NewFrame();
		ImGui::NewFrame();

		ImGui::PushFont(font1);

		ImGui::Begin(
			"main",
			0,
			ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoResize |
			ImGuiWindowFlags_NoScrollbar | ImGuiWindowFlags_NoTitleBar |
			ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoBackground |
			ImGuiWindowFlags_NoCollapse
		);
			int w, h;
			bool windowrightclick = true;
			glfwGetWindowSize(window, &w, &h);
			ImGui::SetWindowSize(ImVec2(w, h));
			ImGui::SetWindowPos(ImVec2(0, 0));

			if (ImGui::IsItemActive()) {
				if (sel.active) {
					sel.to = io.MousePos;
				} else {
					sel.active = true;
					sel.from = io.MousePos;
				}
			} else {
				sel.active = false;
			}

			ImDrawList* draw_list = ImGui::GetWindowDrawList();
			mc.drawGrid(draw_list);

			for (u32 i = 0; i < comps.size(); i++) {
				auto& c = comps[i];
				ImGui::SetWindowFontScale(mc.zoom);
				c->draw(draw_list);
				ImGui::SetWindowFontScale(1);
				if (ImGui::BeginPopupContextItem(c->context_menu_id)) {
					windowrightclick = false;
					if (ImGui::Selectable("ok 123")) {
						printf("%s pressed\n", c->id);
					}
					if (ImGui::Selectable("Delete")) {
						delete comps[i];

						// ezt is ki a faszom találta ki?
						std::vector<Component*>::iterator it = comps.begin();
						std::advance(it, i);
						comps.erase(it);
					}
					ImGui::EndPopup();
				}
			}

			// Kijelölő téglalap
			if (sel.active)
				draw_list->AddRectFilled(sel.from, sel.to, 0x80808080);

			// Mik vannak ebben a téglalapban
			if (sel.active) {
				for (Component* c : comps) {
					if (
						std::clamp(mc.zoomx(c->pos.x + mc.panpos.x), min(sel.from.x, sel.to.x), max(sel.from.x, sel.to.x)) == (mc.zoomx(c->pos.x + mc.panpos.x)) &&
						std::clamp(mc.zoomy(c->pos.y + mc.panpos.y), min(sel.from.y, sel.to.y), max(sel.from.y, sel.to.y)) == (mc.zoomy(c->pos.y + mc.panpos.y))
					)
						c->selected = true;
					else
						c->selected = false;
				}
			}

			// Ha a felhasználó megnyomta az 'e' gombot, akkor az összes kijelölt
			// elem törlődik
			if (ImGui::IsKeyDown(ImGuiKey_Delete)) {
				for (u32 i = 0; i < comps.size(); i++) {
					if (!comps[i]->selected) continue;

					delete comps[i];
					// ezt is ki a faszom találta ki?
					std::vector<Component*>::iterator it = comps.begin();
					std::advance(it, i);
					comps.erase(it);
				}
			}

			// Vágólap műveletek
			if (ImGui::IsKeyDown(ImGuiKey_LeftCtrl)) {
				if (ImGui::IsKeyReleased(ImGuiKey_C)) {
					clipboard.copy();
				} else if (ImGui::IsKeyReleased(ImGuiKey_X)) {
					clipboard.cut();
				} else if (ImGui::IsKeyReleased(ImGuiKey_V)) {
					clipboard.paste();
				} else if (ImGui::IsKeyReleased(ImGuiKey_D)) {
					clipboard.copy();
					clipboard.paste();
				}
			}

			if (windowrightclick && ImGui::BeginPopupContextWindow("item1")) {
				for (u32 i = 0; i < DEFINED_COMPS; i++) {
					if (ImGui::Selectable(comptypes[i])) {
						comps.push_back(new Component(mc.snap(ImGui::GetMousePos()), (Comps)i));
					}
				}
				ImGui::EndPopup();
			}

			if (ImGui::IsMouseDragging(ImGuiMouseButton_Middle)) {
				mc.panpos += ImVec2((std::round(io.MouseDelta.x * 1024) / 1024) / mc.zoom, (std::round(io.MouseDelta.y * 1024) / 1024) / mc.zoom);
			}

			if (ImGui::IsKeyDown(ImGuiKey_LeftCtrl)) {
				mc.zoom = mc.zoom + (std::round(io.MouseWheel * 1024) / 1024) / 20.f * mc.zoom;
			}

			// printf("%d %d\n", (u32)mc.snap(ImGui::GetMousePos()).x, (u32)mc.snap(ImGui::GetMousePos()).y);

			// Karika ahol van a kurzor grid-re snappelve
			ImVec2 gridpos = mc.snap(ImGui::GetMousePos());
			ImGui::SetCursorPos(ImVec2(0, 0));
			ImGui::Text("%f %f", gridpos.x, gridpos.y);
			draw_list->AddCircleFilled(mc.convert(gridpos), 4, 0xffffffff);
			draw_list->AddCircleFilled(ImGui::GetMousePos(), 4, 0xff00ffff);

			// ImGui::SetWindowFontScale(1);

			// if (ImGui::BeginMenuBar()) {
			// 	ImGui::MenuItem("asd");
			// 	ImGui::EndMenuBar();
			// }
		ImGui::End();

		ImGui::PopFont();

		ImGui::Render();
		ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());

		glfwSwapBuffers(window);
		glfwPollEvents();

		double diff = glfwGetTime() - start;
		// printf("FPS %lf\n", 1 / diff);
	}

	// Komponensek free-elése
	for (auto ptr : comps) {
		free(ptr);
	}

	ImGui_ImplOpenGL3_Shutdown();
	ImGui_ImplGlfw_Shutdown();
	ImGui::DestroyContext();

	glfwTerminate();
}
