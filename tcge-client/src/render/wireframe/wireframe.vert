#version 330 core

uniform mat4 transform;

layout (location = 0) in vec3 Position;
layout (location = 1) in vec4 Color;

out vec4 color;

void main() {
	gl_Position = transform * vec4(Position, 1.0);
	color = Color;
}
