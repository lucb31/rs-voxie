#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(std140) uniform InstanceData {
  mat4 modelMatrices[256];
};

uniform mat4 uViewProjection;

out vec3 fragNormal;

void main() {
    mat4 model = modelMatrices[gl_InstanceID];
    fragNormal = aNormal;
    gl_Position = uViewProjection * model * vec4(aPos, 1.0);
}
