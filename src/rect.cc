#include "rect.hh"
#include "colors.hh"

#include <GLFW/glfw3.h>
#include <math.h>

Rectangle::Rectangle(Shader* s, f32 x, f32 y, f32 w, f32 h) : x(x), y(y), w(w), h(h), s(s) {
	GLfloat const verts[] = {
		x, y + h,
		x + w, y,
		x, y,
		x + w, y + h,
	};

	GLuint const elems[] {
		0, 1, 2,
		0, 3, 1
	};

	gl(glGenVertexArrays(1, &vao));
	gl(glBindVertexArray(vao));

	gl(glGenBuffers(1, &vbo));
	gl(glBindBuffer(GL_ARRAY_BUFFER, vbo));
	gl(glBufferData(GL_ARRAY_BUFFER, sizeof(verts), verts, GL_STATIC_DRAW));

	gl(glGenBuffers(1, &ebo));
	gl(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo));
	gl(glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(elems), elems, GL_STATIC_DRAW));

	glEnableVertexAttribArray(0);

	gl(glBindBuffer(GL_ARRAY_BUFFER, vbo));
	gl(glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 0, 0));
	gl(glBindBuffer(GL_ARRAY_BUFFER, 0));

	// pos = glGetUniformLocation(s->program, "pos");
	color = glGetUniformLocation(s->program, "color");

	setColor(1, 1, 1);
}

void Rectangle::prepare() {
	gl(glBindVertexArray(vao));
}

void Rectangle::draw() {
	s->use();
	gl(glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0));
}

void Rectangle::setPos(u32 x, u32 y) {
	// TODO
}

void Rectangle::setColor(f32 r, f32 g, f32 b) {
	s->use();
	gl(glUniform3f(color, r, g, b));
}

Rectangle::~Rectangle() {
	gl(glDeleteBuffers(1, &ebo));
	gl(glDeleteBuffers(1, &vbo));
	gl(glDeleteVertexArrays(1, &vao));
}
