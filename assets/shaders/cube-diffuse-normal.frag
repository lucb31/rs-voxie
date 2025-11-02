#version 330 core

// Fragment shader for full lighting incl 
// - Ambient light
// - Diffuse lighting
// - Specular lighting
// - Normal map texture support
// Requires vert shader that provides tangent space transformations

in vec3 vNormal;
in vec3 vPos;
in vec2 vTexCoord;
in vec3 TangentLightDir;
in vec3 TangentFragPos;
in vec3 TangentViewPos;
out vec4 FragColor;

uniform mat3 uModelInverseTranspose;

// Light settings
vec3 ambientLightColor = vec3(.1);
uniform vec3 uLightColor = vec3(1);

uniform sampler2D diffuseMap;
uniform sampler2D normalMap;

float K_s = 32.0;
float specular_strength = 0.2;

// Calc lighting color in **TANGENT** space
void main() {
  // Diffuse lighting
  vec3 lightDir = TangentLightDir;
  vec3 normal = texture(normalMap, vTexCoord).rgb;
  // transform to normal in tangent space within range [-1,1]
  normal = normalize(normal * 2.0 - 1.0);
  float diff = max(dot(normal, lightDir), 0.0);
  vec3 diffuse = diff * uLightColor;

  // Specular
  vec3 viewDir = normalize(TangentViewPos - TangentFragPos);
  vec3 reflectDir = reflect(-lightDir, normal);
  float spec = pow(max(dot(viewDir, reflectDir), 0.0), K_s);
  vec3 specular = specular_strength * uLightColor * spec;

  // Combine diffuse with ambient light
  vec3 objectColor = texture(diffuseMap, vTexCoord).xyz;
  vec3 result = (ambientLightColor + diffuse + specular) * objectColor;
  gl_FragColor = vec4(result, 1.0);
}
