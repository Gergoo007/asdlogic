#include "text.hh"
#include "shader.hh"

#include <iostream>
#include <glm/ext.hpp>

Character::Character(
	GLuint texture,
	glm::ivec2 size,
	glm::ivec2 bearing,
	u32 advance) :

	texture(texture),
	size(size),
	bearing(bearing),
	advance(advance)
{}

TextEngine::TextEngine(const char* ttf, u32 height) : fs("font.vert", "font.frag") {
	if (FT_Init_FreeType(&ft))
		std::cerr << "TextEngine: FreeType init error" << std::endl;

	if (FT_New_Face(ft, "arial.ttf", 0, &face))
		std::cerr << "TextEngine: Failed to load TTF 'face'" << std::endl;

	// A width 0 mert majd az FT eldönti mekkora
	FT_Set_Pixel_Sizes(face, 0, 48);

	gl(glPixelStorei(GL_UNPACK_ALIGNMENT, 1));

	chars = (Character*) malloc(sizeof(Character) * 128);

	for (u32 i = 33; i < 127; i++) {
		if (FT_Load_Char(face, i, FT_LOAD_RENDER))
			std::cerr << "TextEngine: Unable to render character '" << i << "'" << std::endl;

		GLuint tex;
		gl(glGenTextures(1, &tex));
		gl(glBindTexture(GL_TEXTURE_2D, tex));
		gl(glTexImage2D(
			GL_TEXTURE_2D,
			0,
			GL_RED,
			face->glyph->bitmap.width,
			face->glyph->bitmap.rows,
			0,
			GL_RED,
			GL_UNSIGNED_BYTE,
			face->glyph->bitmap.buffer
		));
		glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
		glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
		glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
		glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

		chars[i] = Character(
			tex,
			glm::ivec2(face->glyph->bitmap.width, face->glyph->bitmap.rows),
			glm::ivec2(face->glyph->bitmap_left, face->glyph->bitmap_top),
			face->glyph->advance.x
		);
	}

	gl(glEnable(GL_BLEND));
	gl(glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA));

	fs.use();
	glm::mat4 projection = glm::ortho(0.0f, 1280.0f, 720.0f, 0.0f);

	textColor = glGetUniformLocation(fs.program, "color");
	gl(glUniform3f(textColor, 1, 1, 1));

	GLuint proj = glGetUniformLocation(fs.program, "projection");
	gl(glUniformMatrix4fv(proj, 1, GL_FALSE, glm::value_ptr(projection)));

	gl(glGenVertexArrays(1, &vao));
	gl(glGenBuffers(1, &vbo));
	gl(glBindVertexArray(vao));
	gl(glBindBuffer(GL_ARRAY_BUFFER, vbo));
	gl(glBufferData(GL_ARRAY_BUFFER, sizeof(float) * 6 * 4, NULL, GL_DYNAMIC_DRAW));
	gl(glEnableVertexAttribArray(0));
	gl(glVertexAttribPointer(0, 4, GL_FLOAT, GL_FALSE, 4 * sizeof(float), 0));
	glBindBuffer(GL_ARRAY_BUFFER, 0);
	glBindVertexArray(0);

	// Space hozzáadása a betűtípushoz
	chars[' '] = chars['l'];
	chars[' '].texture = 0;
}

void TextEngine::setColor(f32 r, f32 g, f32 b) {
	textColor = glGetUniformLocation(fs.program, "color");
	gl(glUniform3f(textColor, r, g, b));
}

void TextEngine::drawText(const char* text, f32 x, f32 y, f32 scale) {
	fs.use();

	gl(glActiveTexture(GL_TEXTURE0));
	gl(glBindVertexArray(vao));

	for (u32 i = 0; i < strlen(text); i++) {
		Character* c = &chars[text[i]];

		f32 xpos = x + c->bearing.x * scale;
		f32 ypos = y - c->size.y * scale;
		f32 w = c->size.x * scale;
		f32 h = c->size.y * scale;
		f32 verts[6][4] = {
			{ xpos,     ypos + h,   0.0f, 1.0f },
			{ xpos,     ypos,       0.0f, 0.0f },
			{ xpos + w, ypos,       1.0f, 0.0f },

			{ xpos,     ypos + h,   0.0f, 1.0f },
			{ xpos + w, ypos,       1.0f, 0.0f },
			{ xpos + w, ypos + h,   1.0f, 1.0f }
		};

		gl(glBindTexture(GL_TEXTURE_2D, c->texture));
		gl(glBindBuffer(GL_ARRAY_BUFFER, vbo));
		gl(glBufferSubData(GL_ARRAY_BUFFER, 0, sizeof(verts), verts));
		gl(glBindBuffer(GL_ARRAY_BUFFER, 0));
		
		gl(glDrawArrays(GL_TRIANGLES, 0, 6));

		x += (c->advance >> 6) * scale;
	}
}

TextEngine::~TextEngine() {
	FT_Done_Face(face);
	FT_Done_FreeType(ft);
	free(chars);
}
