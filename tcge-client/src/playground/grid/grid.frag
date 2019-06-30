#version 330 core
#extension GL_OES_standard_derivatives : enable

in vec3 position;
out vec4 Color;

float grid_xz(vec2 vertex) {
	vec2 coord = vertex;
  vec2 grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
  float line = min(grid.x, grid.y);
  return 1.0 - min(line, 1.0);
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

const float grid_level_0 = 1.0 / 1.0;
const float grid_level_1 = 1.0 / 16.0;
const float grid_level_2 = 1.0 / 256.0;

const float glc0 = 1.0 / 6;
const float glc1 = 1.0 / 5;
const float glc2 = 1.0 / 4;

void main() {
	float glp2 = grid_xz(position.xz * grid_level_2);
	float glp1 = grid_xz(position.xz * grid_level_1);
	float glp0 = grid_xz(position.xz * grid_level_0);
	
	float s = max(glp2, max(glp1, glp0));
	
	vec3 hsv = vec3(1.0, 0.0, (glp0*glc0 + glp1*glc1 + glp2*glc2));
	
	Color = vec4(hsv2rgb(hsv), s);
}
