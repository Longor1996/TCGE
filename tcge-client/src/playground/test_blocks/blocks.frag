#version 330 core

uniform sampler2D atlas;

in vec3 position;
in vec2 texcoord;
in float ao_term;

out vec4 Color;

void main() {
    Color = texture2D(atlas, texcoord) * vec4(1.0 - ao_term, 1.0 - ao_term, 1.0 - ao_term, 1.0);
}
