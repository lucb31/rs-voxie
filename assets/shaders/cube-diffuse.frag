#version 330 core

// Simple ambient + diffuse lighting with a **single** light source
// Support both point lights (uLightPos) and directional lights (uLightDir)
// By default will use a point light at the origin

in vec3 vNormal;
in vec3 vPos;
in vec2 vTexCoord;
out vec4 FragColor;

// Light settings
uniform vec3 uAmbientLightColor = vec3(0.15);
// Position of point light in **world** coordinates
uniform vec3 uLightPos = vec3(0.0);
// Direction of directional light in **world** coordinates
uniform vec3 uLightDir = vec3(0.0);
uniform vec3 uLightColor = vec3(1);

uniform sampler2D diffuseMap;

// Calc lighting color in **World** space
void main() {
  // Diffuse lighting
  vec3 lightDir = uLightDir;
  // Calculate light dir for point light if no lightDir explicitly specified
  if (dot(lightDir, lightDir) < 1e-8) {
    lightDir = normalize(uLightPos - vPos);
  }
  vec3 norm = normalize(vNormal);
  float diff = max(dot(norm, lightDir), 0.0);
  vec3 diffuse = diff * uLightColor;

  // Combine diffuse with ambient light
  vec3 objectColor = texture(diffuseMap, vTexCoord).xyz;
  vec3 result = (uAmbientLightColor + diffuse) * objectColor;
  gl_FragColor = vec4(result, 1.0);
}
