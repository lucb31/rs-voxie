#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
// Cube location in world space
layout(location = 2) in vec3 aTranslation;

uniform mat4 uView;
uniform mat4 uProjection;

out vec3 fragNormal;
out vec3 vLocalPos;
out vec3 vNormal;

void main() {
  mat4 vp = uProjection * uView;
  mat4 model = mat4(
      1.0, 0.0, 0.0, 0.0,  // Column 0
      0.0, 1.0, 0.0, 0.0,  // Column 1
      0.0, 0.0, 1.0, 0.0,  // Column 2
      aTranslation.x, aTranslation.y, aTranslation.z, 1.0  // Column 3
  );
  fragNormal = aNormal;
  vLocalPos = aPos;
  vNormal = mat3(transpose(inverse(model))) * aNormal;
  gl_Position = vp * model * vec4(aPos, 1.0);
}
