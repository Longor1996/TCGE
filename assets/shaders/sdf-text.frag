#version 330 core

uniform vec4 color;
uniform sampler2D sdfmap;
in vec3 position;
in vec2 texcoord;

out vec4 Color;

void main() {
	float mask = texture2D(sdfmap, texcoord).a;

	Color.rgb = color.rgb;
	Color.a = mask < 0.5 ? 0.0 : 1.0;

	Color.a *= smoothstep(0.25, 0.75, mask);
}
