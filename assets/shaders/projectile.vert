#version 330 core

layout (location = 0) in vec3 aPos;

out vec3 worldPos;
flat out float sphereRadius;
flat out vec3 camPos;
flat out vec3 sphereCenter;

uniform mat4 uModel;
uniform mat4 uModelIV;
uniform mat4 uView;
uniform mat4 uProjection;

void main() {
  mat4 invView = inverse(uView);
  camPos = invView[3].xyz;
  vec4 wPos = uModel * vec4(aPos, 1.0);
  worldPos = vec3(wPos);

  // assuming uniform scale
  sphereRadius = length(uModel[0].xyz) / 2.0;
  sphereCenter = uModel[3].xyz;

  gl_Position = uProjection * uView * wPos;
}
