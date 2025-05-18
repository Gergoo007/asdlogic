#include <backend.hh>

static void opengl3_init(GLFWwindow* window) {
	ImGui_ImplGlfw_InitForOpenGL(window, true);
	ImGui_ImplOpenGL3_Init();
}

static void opengl3_newframe() {
	ImGui_ImplOpenGL3_NewFrame();
	ImGui_ImplGlfw_NewFrame();
}

static void opengl3_render(ImDrawData* draw_data) {
	ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());
}

static void opengl3_shutdown() {
	ImGui_ImplOpenGL3_Shutdown();
	ImGui_ImplGlfw_Shutdown();
}

static void vulkan_init(GLFWwindow* window) {
	ImGui_ImplGlfw_InitForVulkan(window, true);
	ImGui_ImplVulkan_Init(0);
}

void Backend::setBackend(const char* name) {
	if (strcmp(name, "opengl") || strcmp(name, "opengl3")) {
		backend = Selection::BE_GLFW3_OPENGL3;
	} else if (strcmp(name, "vulkan")) {
		backend = Selection::BE_GLFW3_VULKAN;
	} else if (strcmp(name, "directx") || strcmp(name, "directx12") || strcmp(name, "dx12")) {
		backend = Selection::BE_DX12;
	}

	switch (backend) {
		case BE_GLFW3_OPENGL3: {
			init = opengl3_init;
			newframe = opengl3_newframe;
			render = opengl3_render;
			shutdown = opengl3_shutdown;
			break;
		}

		case BE_GLFW3_VULKAN: {
			backendStuff.vulkan.g_Allocator = nullptr;
			backendStuff.vulkan.g_Instance = VK_NULL_HANDLE;
			backendStuff.vulkan.g_PhysicalDevice = VK_NULL_HANDLE;
			backendStuff.vulkan.g_Device = VK_NULL_HANDLE;
			backendStuff.vulkan.g_QueueFamily = (u32)-1;
			backendStuff.vulkan.g_Queue = VK_NULL_HANDLE;
			backendStuff.vulkan.g_PipelineCache = VK_NULL_HANDLE;
			backendStuff.vulkan.g_DescriptorPool = VK_NULL_HANDLE;
			backendStuff.vulkan.g_MinImageCount = 2;
			backendStuff.vulkan.g_SwapChainRebuild = false;
			break;
		}

		case BE_DX12: {

			break;
		}
	}
}
