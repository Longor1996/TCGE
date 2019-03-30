#version 330 core
#extension GL_OES_standard_derivatives : enable

in vec3 position;
out vec4 Color;

float grid_y(float vertex) {
  float coord = vertex;
  float line = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
  return 1.0 - min(line, 1.0);
}

float grid_xz(vec2 vertex) {
	vec2 coord = vertex;
  vec2 grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
  float line = min(grid.x, grid.y);
  return 1.0 - min(line, 1.0);
}

float grid_xyz(vec3 vertex) {
	vec3 coord = vertex;
  vec3 grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
  float line = min(min(grid.x, grid.y), grid.z);
  return 1.0 - min(line, 1.0);
}

float grid_rad(vec3 vertex) {
	float coord = length(vertex);
  float line = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
  return 1.0 - min(line, 1.0);
}

float grid_lrad(vec3 vertex) {
  const float pi = 3.141592653589793;
  const float scale = 10.0;
  vec2 coord = vec2(length(vertex.xz), atan(vertex.x, vertex.z) * scale / pi);
  vec2 wrapped = vec2(coord.x, fract(coord.y / (2.0 * scale)) * (2.0 * scale));
  vec2 coordWidth = fwidth(coord);
  vec2 wrappedWidth = fwidth(wrapped);
  vec2 width = coord.y < -scale * 0.5 || coord.y > scale * 0.5 ? wrappedWidth : coordWidth;
  vec2 grid = abs(fract(coord - 0.5) - 0.5) / width;
  float line = min(grid.x, grid.y);
  return 1.0 - min(line, 1.0);
}

vec3 rgb2hsv(vec3 c) {
  vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
  vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
  vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

  float d = q.x - min(q.w, q.y);
  float e = 1.0e-10;
  return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

const float grid_level_0 = 1.0 / 1.0;
const float grid_level_1 = 1.0 / 10.0;
const float grid_level_2 = 1.0 / 100.0;
void main() {
    float A = grid_xz(position.xz * grid_level_2);
	float B = grid_xz(position.xz * grid_level_1);
	float C = grid_xz(position.xz * grid_level_0);
	float s = max(A,max(B,C));

	vec3 hsv = vec3(1f - (C*0.5f + B*0.35f + A*0.15f), 1.0f, 1.0f);

	Color = vec4(hsv2rgb(hsv), s);
}
