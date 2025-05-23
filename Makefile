# Disable idiotic invisible default variables like CC
MAKEFLAGS += --no-builtin-variables

rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

CCSRCS := $(subst src/,,$(call rwildcard,src,*.cc))
CCOBJS := $(patsubst %.cc,out/%.cc.o,$(CCSRCS))
CCDEPENDS := $(patsubst %.cc,out/%.cc.d,$(CCSRCS))

CPPSRCS += $(subst src/,,$(call rwildcard,src,*.cpp))
CPPOBJS += $(patsubst %.cpp,out/%.cpp.o,$(CPPSRCS))
CPPDEPENDS += $(patsubst %.cpp,out/%.cpp.d,$(CPPSRCS))

LINUXFLAGS := $(shell pkg-config --libs --cflags gl glfw3 vulkan)
WIN64FLAGS := -lopengl32 $(shell x86_64-w64-mingw32-pkg-config --libs --cflags glfw3 vulkan) -lgdi32
_CFLAGS := $(CFLAGS) -std=gnu++23 -Wshadow -O0 -g -Wno-multichar -Isrc -MMD -MP

unix: clean
	CC=g++ CFLAGS='$(LINUXFLAGS)' $(MAKE) link

windows: clean
	CC=x86_64-w64-mingw32-g++ CFLAGS='$(WIN64FLAGS)' $(MAKE) link

windows_static: clean
	CC=x86_64-w64-mingw32-g++ CFLAGS='$(WIN64FLAGS)' $(MAKE) static_link

all:
	$(MAKE) unix
	$(MAKE) windows
	$(MAKE) windows_static

static_link: $(CPPOBJS) $(CCOBJS)
	$(CC) $(CPPOBJS) $(CCOBJS) $(_CFLAGS) -static -o asdlogic_static

link: $(CPPOBJS) $(CCOBJS)
	$(CC) $(CPPOBJS) $(CCOBJS) $(_CFLAGS) -o asdlogic

-include $(CCDEPENDS) $(CPPDEPENDS)

out/%.cc.o: src/%.cc Makefile
	echo "  > $(CC) $<"
	$(CC) -c $< $(_CFLAGS) -o $@

out/%.cpp.o: src/%.cpp Makefile
	echo "  > $(CC) $<"
	$(CC) -c $< $(_CFLAGS) -o $@

clean:
	rm -rf out/*
	find src -type d -exec mkdir -p "out/{}" \;
	cp -r out/src/* out/
	rm -rf out/src

.PHONY: modules
$(V).SILENT:
