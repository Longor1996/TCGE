#version 330 core

out vec4 Color;
uniform float time;

void main()
{
	float f = time * 10.0;

  Color = vec4(
    (sin(f) + 1.0f) * 0.5f,
    (sin(f) + 1.0f) * 0.5f,
    (sin(f) + 1.0f) * 0.5f,
    1.0f
  );
}