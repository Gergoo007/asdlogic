#include <GL/glew.h>
#include <GLFW/glfw3.h>
#include <math.h>

#include <ft2build.h>
#include FT_FREETYPE_H

#include <iostream>
#include <string>
#include <fstream>
#include <sstream>

#include <glm/ext.hpp>

#include "shader.hh"
#include "rect.hh"
#include "colors.hh"
#include "text.hh"

Shader* s;
TextEngine* te;

void onResize(GLFWwindow* window, int w, int h) {
	glViewport(0, 0, w, h);

	glm::mat4 projection = glm::ortho(0.0f, (float)w, (float)h, 0.0f);
	s->setProjection(glm::value_ptr(projection));
	te->fs.setProjection(glm::value_ptr(projection));
}

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
	glfwSetWindowSizeCallback(window, onResize);

	GLenum err = glewInit();
	if (GLEW_OK != err) {
		std::cerr << "GLEW error: " << glewGetErrorString(err) << std::endl;
		glfwTerminate();
		return 1;
	}

	s = new Shader("shader.vert", "shader.frag");
	s->use();

	Rectangle* r = new Rectangle(s, 0, 0, 100, 100);
	r->setColor(GRAY);

	te = new TextEngine("arial.ttf", 48);

	onResize(window, 1280, 720);

	while (!glfwWindowShouldClose(window)) {
		double start = glfwGetTime();

		gl(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));
		gl(glClearColor(BG_COLOR, 1));

		// r->setPos(0, 0);
		r->prepare();
		r->draw();

		te->drawText("helo", 0, 68, 1);

		glfwSwapBuffers(window);
		glfwPollEvents();

		double diff = glfwGetTime() - start;
		// printf("FPS %lf\n", 1 / diff);
	}

	delete te;
	delete s;
	delete r;

	glfwTerminate();
}

void ogl_print_errors(char const* const fun, char const* const file, int const line) {
	GLenum e = glGetError();
	if (e != GL_NO_ERROR) {
		char* estr = (char*)gluErrorString(e);
		if (estr)
			fprintf(stderr, "OpenGL error @ %s:%d (%s): '%s'\n", file, line, fun, estr);
		else
			fprintf(stderr, "OpenGL error @ %s:%d (%s): '%d'\n", file, line, fun, e);
	}
}
