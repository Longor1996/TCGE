#version 330 core

in vec3 position;
out vec4 Color;
uniform float time;

void main()
{
	float f = time * 1.0;

  Color = vec4(
    (sin(f + position.x) + 1.0f) * 0.5f,
    (sin(f + position.y) + 1.0f) * 0.5f,
    (sin(f + position.z) + 1.0f) * 0.5f,
    1.0f
  );
}