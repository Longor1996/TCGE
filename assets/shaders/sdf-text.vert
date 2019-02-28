#version 330 core

uniform mat4 transform;
layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 TexCoord;

out vec3 position;
out vec2 texcoord;

void main() {
    gl_Position = transform * vec4(Position, 0.0, 1.0);
    position = vec3(Position, 0.0);
    texcoord = TexCoord;
}
