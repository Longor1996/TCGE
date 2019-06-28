#version 330 core

in vec3 position;
out vec4 Color;

uniform vec4 color;

void main() {
	Color = color;
}
