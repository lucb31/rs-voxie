#version 330 core

in vec3 vNormal;
in vec3 vPos;
in vec2 vTexCoord;
out vec4 FragColor;

uniform mat3 uModelInverseTranspose;

// Light settings
vec3 ambientLightColor = vec3(.1);
uniform vec3 uLightPos = vec3(0.0, 10.0, 0.0);
uniform vec3 uLightColor = vec3(1);

uniform sampler2D tex;

// Calc lighting color in **World** space
void main() {
  // Diffuse lighting
  vec3 lightDir = normalize(uLightPos - vPos);
  vec3 norm = normalize(vNormal);
  float diff = max(dot(norm, lightDir), 0.0);
  vec3 diffuse = diff * uLightColor;

  // Combine diffuse with ambient light
  vec3 objectColor = texture(tex, vTexCoord).xyz;
  vec3 result = (ambientLightColor + diffuse) * objectColor;
  gl_FragColor = vec4(result, 1.0);
}
