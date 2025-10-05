#version 330 core

in vec3 worldPos;
out vec4 fragColor;

uniform vec3 camPos;        // Camera position in world space
uniform vec3 sphereCenter;  // Sphere center in world space
uniform float sphereRadius; // Ensure that diameter does not get bigger than cube dimensions

// Arbitraty light direction
vec3 lightDir = vec3(1.0);

// Utilizing screen space ray tracing to render a perfect sphere 
// NOTE: Scaling does not work yet
// NOTE: Expensive! Don't use for many objects
void main() {
    // Ray origin and direction
    vec3 rayOrigin = camPos;
    vec3 rayDir = normalize(worldPos - rayOrigin);

    // Ray-sphere intersection
    vec3 oc = rayOrigin - sphereCenter;
    float b = dot(oc, rayDir);
    float c = dot(oc, oc) - sphereRadius * sphereRadius;
    float h = b * b - c;

    if (h < 0.0) {
        discard; // Missed the sphere
    }

    // Compute intersection
    h = sqrt(h);
    float t = -b - h;
    vec3 hitPos = rayOrigin + rayDir * t;
    vec3 normal = normalize(hitPos - sphereCenter);

    // Simple lighting
    vec3 lightDirNormalized = normalize(lightDir);
    float diff = max(dot(normal, lightDirNormalized), 0.0);
    fragColor = vec4(vec3(0.1, 0.6, 1.0) * diff, 1.0);
}
