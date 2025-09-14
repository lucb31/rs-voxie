#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(std140) uniform InstanceData {
  mat4 modelMatrices[256];
};

uniform mat4 uView;
uniform mat4 uProjection;

out vec3 fragNormal;
out vec3 vLocalPos;
out vec3 vNormal;

void main() {
  mat4 vp = uProjection * uView;
  mat4 model = modelMatrices[gl_InstanceID];
  fragNormal = aNormal;
  vLocalPos = aPos;
  vNormal = mat3(transpose(inverse(model))) * aNormal;
  gl_Position = vp * model * vec4(aPos, 1.0);
}
