#version 330 core
precision mediump float;

uniform vec4 color = vec4(1.0, 1.0, 1.0, 1.0);
uniform vec4 outlineColor = vec4(0.0, 0.0, 0.0, 1.0);
uniform float outlineDistance = 0.4;
uniform float outlineTreshold = 0.5;
uniform float spread = 8.0;
uniform float scale = 16.0;
uniform sampler2D sdfmap;

in vec3 position;
in vec2 texcoord;

out vec4 Color;

void main() {
	float smoothing = 0.25 / (spread * scale);
	float distance = texture2D(sdfmap, texcoord).a;
	
	float outlineFactor = smoothstep(outlineTreshold - smoothing, outlineTreshold + smoothing, distance);
	vec4 fcolor = mix(outlineColor, color, outlineFactor);
	
	float alpha = smoothstep(outlineDistance - smoothing, outlineDistance + smoothing, distance);
	
	Color = vec4(fcolor.rgb, fcolor.a * alpha);
}
