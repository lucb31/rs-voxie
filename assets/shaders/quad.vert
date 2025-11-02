#version 330 core

uniform mat4 uMVP;
layout(location = 0) in vec2 aPos;

out vec2 vUV;

void main() {
  vUV = (aPos.xy + 1.0) * 0.5;
  gl_Position = uMVP * vec4(aPos, 0.0, 1.0);
}
