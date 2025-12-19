#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;

uniform mat4 uModel;
uniform mat3 uModelIV;
uniform mat4 uView;
uniform mat4 uProjection;

out vec3 vNormal;
out vec3 vPos;
out vec2 vTexCoord;

// Lighting calculation in **WORLD** space
void main() {
  vPos = vec3(uModel * vec4(aPos, 1.0));
  // Calculate normals with inverse transpose
  vNormal = uModelIV * aNormal;
  vTexCoord = aTexCoord;
  gl_Position = uProjection * uView * vec4(vPos, 1.0);
}
