#pragma once

#include "types.hh"
#include "shader.hh"
#include <GL/glew.h>

struct Rectangle {
	f32 x, y, w, h;
	GLuint vbo, vao, ebo, pos, color;
	Shader* s;

	Rectangle(Shader* s, f32 x, f32 y, f32 w, f32 h);
	void prepare();
	void draw();
	void setPos(u32 x, u32 y);
	void setColor(f32 r, f32 g, f32 b);
	~Rectangle();
};
