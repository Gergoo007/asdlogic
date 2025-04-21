# Disable idiotic invisible default variables like CC
MAKEFLAGS += --no-builtin-variables

rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

CCSRCS := $(subst src/,,$(call rwildcard,src,*.cc))
CCOBJS := $(patsubst %.cc,out/%.cc.o,$(CCSRCS))

CPPSRCS += $(subst src/,,$(call rwildcard,src,*.cpp))
CPPOBJS += $(patsubst %.cpp,out/%.cpp.o,$(CPPSRCS))

_CFLAGS := $(CFLAGS) -std=gnu++23 -Wshadow -O0 -g -Wno-multichar \
	$(shell pkg-config --libs --cflags gl glew glfw3 freetype2)

CC := g++
LD ?= ld.gold
AS ?= gcc

link: $(CPPOBJS) $(CCOBJS)
	$(CC) $(_CFLAGS) $(CPPOBJS) $(CCOBJS) -o logic123

out/%.cc.o: src/%.cc
	echo "  > $(CC) $^"
	$(CC) $(_CFLAGS) -c $^ -o $@

out/%.cpp.o: src/%.cpp
	echo "  > $(CC) $^"
	$(CC) $(_CFLAGS) -c $^ -o $@

clean:
	rm -rf out/*
	find src -type d -exec mkdir -p "out/{}" \;
	cp -r out/src/* out/
	rm -rf out/src

.PHONY: modules
$(V).SILENT:
