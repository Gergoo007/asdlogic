#include <GLFW/glfw3.h>

#include <iostream>
#include <string>
#include <fstream>
#include <sstream>
#include <vector>
#include <algorithm>

#include "colors.hh"
// #include "comp/and.hh"
#include "comp/comp.hh"

#include "imgui/imgui.h"
#include "imgui/imgui_impl_glfw.h"
#include "imgui/imgui_impl_opengl3.h"

#include "util.hh"

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

	// GLenum err = glewInit();
	// if (GLEW_OK != err) {
	// 	std::cerr << "GLEW error: " << glewGetErrorString(err) << std::endl;
	// 	glfwTerminate();
	// 	return 1;
	// }

	ImFont* font1 = io.Fonts->AddFontFromFileTTF("arial.ttf", 24);

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
			ImGuiWindowFlags_NoCollapse | ImGuiWindowFlags_MenuBar
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
			for (u32 i = 0; i < comps.size(); i++) {
				auto& c = comps[i];
				ImGui::SetWindowFontScale(view.zoom);
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
						std::clamp(zoomx(c->pos.x + view.panpos.x), min(sel.from.x, sel.to.x), max(sel.from.x, sel.to.x)) == (zoomx(c->pos.x + view.panpos.x)) &&
						std::clamp(zoomy(c->pos.y + view.panpos.y), min(sel.from.y, sel.to.y), max(sel.from.y, sel.to.y)) == (zoomy(c->pos.y + view.panpos.y))
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

			if (windowrightclick && ImGui::BeginPopupContextWindow("item1")) {
				for (u32 i = 0; i < DEFINED_COMPS; i++) {
					if (ImGui::Selectable(comptypes[i])) {
						comps.push_back(new Component(unzoom(ImGui::GetMousePos()), (Comps)i));
					}
				}
				ImGui::EndPopup();
			}

			if (ImGui::IsMouseDragging(ImGuiMouseButton_Middle)) {
				view.panpos += io.MouseDelta / view.zoom;
			}

			if (ImGui::IsKeyDown(ImGuiKey_LeftCtrl)) {
				view.zoom += io.MouseWheel / 20.f * view.zoom;
			}

			// ImGui::SetWindowFontScale(1);

			if (ImGui::BeginMenuBar()) {
				ImGui::MenuItem("asd");
				ImGui::EndMenuBar();
			}
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
