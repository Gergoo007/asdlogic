#pragma once

#include "types.hh"
#include "shader.hh"

#include <GL/glew.h>

#include <ft2build.h>
#include FT_FREETYPE_H // asd123

#include <glm/glm.hpp>

struct Character {
	GLuint texture;			// OpenGL handle
	glm::ivec2 size;		// Dimensions
	glm::ivec2 bearing;		// Offset from baseline to left/top of glyph
	u32 advance;			// Offset to advance to next glyph

	Character(GLuint texture, glm::ivec2 size, glm::ivec2 bearing, u32 advance);
};

struct Text {
	Text();
	void prepare();
	void draw();
	~Text();
};

struct TextEngine {
	FT_Library ft;
	FT_Face face;
	Character* chars;
	GLuint vao, vbo, textColor;
	Shader fs;

	TextEngine(const char* ttf, u32 height);
	void drawText(const char* text, f32 x, f32 y, f32 scale);
	void setColor(f32 r, f32 g, f32 b);
	~TextEngine();
};
