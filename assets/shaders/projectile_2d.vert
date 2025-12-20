#version 330 core

layout (location = 0) in vec3 aPos;

out vec2 worldPos;
flat out float sphereRadius;
flat out vec2 sphereCenter;

uniform mat4 uModel;
uniform mat4 uModelIV;
uniform mat4 uView;
uniform mat4 uProjection;

void main() {
  vec4 wPos = uModel * vec4(aPos, 1.0);
  worldPos = wPos.xy;

  // assuming uniform scale
  sphereRadius = length(uModel[0].xyz) / 2.0;
  sphereCenter = uModel[3].xy;

  gl_Position = uProjection * uView * wPos;
}
