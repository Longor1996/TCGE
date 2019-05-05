#version 330 core

uniform vec4 color;
uniform vec3 camera;

in vec3 position;
out vec4 Color;

void main() {
	vec3 sky_color = vec3(0.0,0.0,0.0);

	if(position.y > 0) {
		float gradient = normalize(position).y;
		sky_color = mix(color.rgb * 1.1, color.rgb * 0.9, gradient);
	}

	Color = camera.y < 0
		? vec4(0.0,0.0,0.0, 1.0)
		: vec4(sky_color.rgb, 1.0);
}
