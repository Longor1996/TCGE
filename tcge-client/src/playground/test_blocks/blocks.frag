#version 330 core

uniform sampler2D atlas;
uniform vec3 sun = vec3(0.707, 0.707, 0.707);

in vec3 position;
in vec2 texcoord;
in vec3 normal;
in float ao_term;

out vec4 Color;

void main() {
    float lighting = max(dot(normal, sun), 0.0);
    
    Color = texture2D(atlas, texcoord) * vec4(1.0 - ao_term, 1.0 - ao_term, 1.0 - ao_term, 1.0);
    Color.rgb *= 0.25 + lighting;
}
