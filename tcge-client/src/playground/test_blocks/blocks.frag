#version 330 core

uniform sampler2D atlas;

in vec3 position;
in vec2 texcoord;

out vec4 Color;

void main() {
    Color = texture2D(atlas, texcoord);
}
