#version 330 core

uniform mat4 transform;

layout (location = 0) in vec3 Position;
layout (location = 1) in vec2 TexCoord;
layout (location = 2) in vec3 Normal;
layout (location = 3) in float AO_Term;

out vec3 position;
out vec2 texcoord;
out vec3 normal;
out float ao_term;

void main() {
    gl_Position = transform * vec4(Position, 1.0);
    position = Position;
    texcoord = TexCoord;
    normal   = Normal;
    ao_term  = AO_Term;
}
