#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
// Cube location in world space
layout(location = 2) in vec3 aTranslation;
layout(location = 3) in vec2 aTexCoord;
layout(location = 4) in int aMaterialIndex;

uniform int u_atlasSize = 2;
uniform mat4 uView;
uniform mat4 uProjection;

out vec3 vPos;
out vec3 vNormal;
out vec2 vTexCoord;

// Calculate uv coord based on material position within atlas
vec2 vertex_uv_to_atlas_uv(vec2 uv) {
  int tileX = aMaterialIndex % u_atlasSize;
  int tileY = aMaterialIndex / u_atlasSize;
  float tileScale = 1.0 / float(u_atlasSize);
  vec2 baseUV = vec2(tileX, tileY) * tileScale;
  // Clamping is required to avoid filtering artifacts
  vec2 clamped_uv = clamp(uv, 0.01, 0.99);
  return baseUV + clamped_uv * tileScale;
}

void main() {
  mat4 vp = uProjection * uView;
  mat4 model = mat4(
      1.0, 0.0, 0.0, 0.0,  // Column 0
      0.0, 1.0, 0.0, 0.0,  // Column 1
      0.0, 0.0, 1.0, 0.0,  // Column 2
      aTranslation.x, aTranslation.y, aTranslation.z, 1.0  // Column 3
  );

  vPos = vec3(model * vec4(aPos, 1.0));
  // Calculate normals with inverse transpose
  mat3 modelInverseTranspose = mat3(transpose(inverse(model)));
  vNormal = modelInverseTranspose * aNormal;
  vTexCoord = vertex_uv_to_atlas_uv(aTexCoord);
  gl_Position = uProjection * uView * vec4(vPos, 1.0);
}
