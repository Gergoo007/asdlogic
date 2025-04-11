#include "shader.hh"

#include <sstream>
#include <fstream>
#include <iostream>

Shader::Shader(const char* vertex, const char* fragment) {
	std::ifstream vsfile(vertex), fsfile(fragment);
	std::stringstream vssrc, fssrc;
	vssrc << vsfile.rdbuf();
	fssrc << fsfile.rdbuf();
	const std::string& tmp1 = vssrc.str(); // mi a faszom ez? miért így kell megcsinálni?
	const std::string& tmp2 = fssrc.str();
	const char* ptr1 = tmp1.c_str();
	const char* ptr2 = tmp2.c_str();

	GLint shader_sts;
	vhandle = glCreateShader(GL_VERTEX_SHADER);
	gl(glShaderSource(vhandle, 1, &ptr1, 0));
	gl(glCompileShader(vhandle));
	gl(glGetShaderiv(vhandle, GL_COMPILE_STATUS, &shader_sts));
	if (!shader_sts) std::cerr << "Vertex shader compilation failed" << std::endl;
	
	fhandle = glCreateShader(GL_FRAGMENT_SHADER);
	gl(glShaderSource(fhandle, 1, &ptr2, 0));
	gl(glCompileShader(fhandle));
	gl(glGetShaderiv(fhandle, GL_COMPILE_STATUS, &shader_sts));
	if (!shader_sts) std::cerr << "Fragment shader compilation failed" << std::endl;

	program = glCreateProgram();
	gl(glAttachShader(program, vhandle));
	gl(glAttachShader(program, fhandle));
	gl(glLinkProgram(program));
}

void Shader::setProjection(const GLfloat* mat) {
	gl(glUseProgram(program));
	GLuint p = glGetUniformLocation(program, "projection");
	printf("p: %d\n", p);
	gl(glUniformMatrix4fv(p, 1, GL_FALSE, mat));
}

void Shader::use() {
	gl(glUseProgram(program));
}

Shader::~Shader() {
	glDeleteProgram(program);
	glDeleteShader(vhandle);
	glDeleteShader(fhandle);
}
