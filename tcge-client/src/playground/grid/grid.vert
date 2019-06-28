#version 330 core

 layout (location = 0) in vec3 Position;
 uniform mat4 transform;
 out vec3 position;

 void main() {
     gl_Position = transform * vec4(Position, 1.0);
     position = Position;
 }