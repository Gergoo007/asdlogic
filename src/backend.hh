#pragma once

#include <imgui/imgui_impl_glfw.h>
#include <imgui/imgui_impl_opengl3.h>
#include <imgui/imgui_impl_vulkan.h>

#include <types.hh>

// #if defined(_WIN64) || defined(_WIN32) || defined(__CYGWIN__)
// #include <imgui/imgui_impl_dx12.h>
// #endif

struct Backend {
	enum struct WindowingSelection {
		PLATFORM_WIN32,
		PLATFORM_GLFW3,
	};

	enum struct GlSelection {
		OPENGL3,
		VULKAN,
		DX12,
	};

	struct {
		int width, height;
		union {
			struct {
				GLFWwindow* window;
			} glfw;
			struct {

			} win32;
		};
		union {
			struct {

			} opengl;
			struct {
				VkApplicationInfo appInfo;
				ImGui_ImplVulkan_InitInfo initInfo;
				ImGui_ImplVulkanH_Window* MainWindowData;
			} vulkan;
			struct {

			} dx12;
		};
	} backendStuff;

	WindowingSelection windowing;
	GlSelection gl;

	void init();
	bool shouldclose();
	void newframe();
	void render(ImDrawData* draw_data);
	void handlevents();
	void shutdown();

	void setBackend(const char* name);
};

extern Backend* backend;
