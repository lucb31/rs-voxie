#version 330 core

uniform mat4 uModel;
uniform mat4 uView;
uniform mat4 uProjection;

layout(location = 0) in vec2 aPos;

out vec2 vUV;

void main() {
  mat4 mvp = uProjection * uView * uModel;
  vUV = (aPos.xy + 1.0) * 0.5;
  gl_Position = mvp * vec4(aPos, 0.0, 1.0);
}
