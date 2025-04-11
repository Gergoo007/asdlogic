CFLAGS := $(shell pkg-config --libs --cflags gl glew glfw3 freetype2)

all:
	g++ $(CFLAGS) $(shell find src/ -type f -name '*.cc') -o logic123
