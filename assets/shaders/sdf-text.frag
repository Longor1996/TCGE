#version 330 core
precision mediump float;

uniform vec4 color;
uniform sampler2D sdfmap;
in vec3 position;
in vec2 texcoord;

out vec4 Color;

void main() {
	float mask = texture2D(sdfmap, texcoord).a;
	float alpha = smoothstep(0.25, 0.75, mask);
	Color = vec4(color.rgb, alpha);
}
