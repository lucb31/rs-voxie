#version 330 core

uniform mat4 uTransform;
layout(location = 0) in vec2 aPos;

void main() {
    gl_Position = uTransform * vec4(aPos, 0.0, 1.0);
}
