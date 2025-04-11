#pragma once

// Definition in main.c
void ogl_print_errors(char const* const fun, char const* const file, int const line);

#define _DEBUG
#ifdef _DEBUG
#define gl(x) { ogl_print_errors("before "#x, __FILE__, __LINE__); (x); ogl_print_errors(#x, __FILE__, __LINE__); }
#else
#define gl(x) (x)
#endif

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef unsigned long long u64;

typedef signed char i8;
typedef signed short i16;
typedef signed int i32;
typedef signed long i64;

typedef float f32;
typedef double f64;
