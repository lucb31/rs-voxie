#version 330 core

in vec3 fragNormal;

uniform mat3 uMvInverseTranspose;
// Light direction in VIEW space
uniform vec3 uLightDir;
uniform vec3 uColor;

out vec4 frag_color;

// Light settings
vec3 lightColor = vec3(1);
vec3 ambientLightColor = vec3(0.05);
// Material settings
vec3 K_s = vec3(1); // Specular reflection factor
float alpha = 32.0; // Shininess
// Texture

void main() {
    gl_FragColor = vec4(uColor, 1.0);
    // TODO: We dont care about lighting yet
    return;
    vec4 texColor = vec4(uColor, 1.0);
    // Lighting
    // Calculate normals with inverse transpose
    vec3 n = normalize(uMvInverseTranspose * fragNormal);
    // No need to normalize light direction. Uniform will be normalized
    float geometryTerm = max(dot(n, uLightDir), 0.0);
    // Diffuse lighting
    vec3 diffuse = geometryTerm * texColor.xyz;

    // Specular lighting
    // Light perfect reflection direction
    vec3 r = 2.0 * dot(uLightDir, n)*n - uLightDir;
    // viewing direction: negative z
    vec3 v = vec3(0,0,-1);
    vec3 specular = K_s * pow(max(dot(v, r), 0.0), alpha);

    // Ambient light
    vec3 ambient = ambientLightColor * texColor.xyz;
    gl_FragColor = vec4(lightColor * (diffuse + specular) + ambient, 1);
}
