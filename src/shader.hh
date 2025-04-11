#pragma once

#include "types.hh"
#include <GL/glew.h>

struct Shader {
	GLuint program;
	GLuint vhandle;
	GLuint fhandle;

	Shader(const char* vertex, const char* fragment);
	void use();
	void setProjection(const GLfloat* mat);
	~Shader();
};
