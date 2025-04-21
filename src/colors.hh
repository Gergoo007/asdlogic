#pragma once

#define GRAYMAKER(x) x, x, x

#define BG_COLOR GRAYMAKER(0.1)
#define LIGHTGRAY GRAYMAKER(0.2)
#define GRAY GRAYMAKER(0.1)

#define rgb(r, g, b) ((f32)r / 255.0f), ((f32)g / 255.0f), ((f32)b / 255.0f)

#define BUTTON rgb(32, 32, 32)
#define BUTTON_HOVER rgb(60, 60, 60)

#define MENUBAR_BG rgb(50, 50, 50)
