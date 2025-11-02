#version 330 core

// Vert shader for cube incl tangent space transformation
// Transformation required for normal map lighting calculations

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;
layout(location = 3) in vec3 aTangent;
layout(location = 4) in vec3 aBitangent;

uniform mat4 uModel;
uniform mat3 uModelInverseTranspose;
uniform mat4 uView;
uniform mat4 uProjection;
// Light direction in WORLD space
uniform vec3 uLightDir = vec3(0.0, 0.0, -1.0);
uniform vec3 uViewPos;

out vec3 vPos;
out vec2 vTexCoord;
out vec3 TangentViewPos;
out vec3 TangentFragPos;
out vec3 TangentLightDir;

// Lighting calculation in **WORLD** space
void main() {
  vPos = vec3(uModel * vec4(aPos, 1.0));
  vTexCoord = aTexCoord;

  // Calc tangent space matrix
  vec3 T = normalize(uModelInverseTranspose * aTangent);
  vec3 N = normalize(uModelInverseTranspose * aNormal);
  T = normalize(T - dot(T, N) * N);
  vec3 B = cross(N, T);
  mat3 TBN = transpose(mat3(T,B,N));

  // Transform vectors to tangent space
  // Performed in vertex shader to improve performance
  TangentFragPos = TBN * vPos;
  TangentViewPos = TBN * uViewPos;
  TangentLightDir = TBN * uLightDir;

  gl_Position = uProjection * uView * vec4(vPos, 1.0);
}
