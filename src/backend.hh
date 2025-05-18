#pragma once

#include <imgui/imgui_impl_glfw.h>
#include <imgui/imgui_impl_opengl3.h>
#include <imgui/imgui_impl_vulkan.h>

#include <types.hh>

#if defined(_WIN64) || defined(_WIN32) || defined(__CYGWIN__)
#include <imgui/imgui_impl_dx12.h>
#endif

struct Backend {
	enum Selection {
		BE_GLFW3_OPENGL3,
		BE_GLFW3_VULKAN,
		BE_DX12,
	};

	struct {
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
				VkAllocationCallbacks*	g_Allocator;
				VkInstance				g_Instance;
				VkPhysicalDevice		g_PhysicalDevice;
				VkDevice				g_Device;
				u32						g_QueueFamily;
				VkQueue					g_Queue;
				VkPipelineCache			g_PipelineCache;
				VkDescriptorPool		g_DescriptorPool;

				ImGui_ImplVulkanH_Window* g_MainWindowData;
				u32						g_MinImageCount;
				bool					g_SwapChainRebuild;
			} vulkan;
			struct {

			} dx12;
		};
	} backendStuff;

	Selection backend;

	void (*init)(GLFWwindow* win);
	void (*newframe)();
	void (*render)(ImDrawData *draw_data);
	void (*shutdown)();

	void setBackend(const char* name);
};
