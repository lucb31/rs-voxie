#version 330 core

in vec3 vLocalPos;      // Local position of fragment in the cube (from -0.5 to 0.5)
in vec3 vNormal;        // Normal of the fragment
out vec4 FragColor;

uniform vec3 uColor = vec3(1.0, 0.8, 0.6); // Base color of the cube
uniform float edgeWidth = 0.05;               // Thickness of the border

void main() {
    // Convert local position to 0..1 range
    vec3 p = vLocalPos + vec3(0.5);

    // Find distance to closest face edge in each axis
    float edgeX = min(p.x, 1.0 - p.x);
    float edgeY = min(p.y, 1.0 - p.y);
    float edgeZ = min(p.z, 1.0 - p.z);

    // Choose the correct plane based on the dominant normal axis
    float edge = 1.0;
    if (abs(vNormal.x) > 0.9) {
        edge = edgeY < edgeZ ? edgeY : edgeZ;
    } else if (abs(vNormal.y) > 0.9) {
        edge = edgeX < edgeZ ? edgeX : edgeZ;
    } else if (abs(vNormal.z) > 0.9) {
        edge = edgeX < edgeY ? edgeX : edgeY;
    }

    // If fragment is close to edge, darken it
    if (edge < edgeWidth) {
        FragColor = vec4(0.0, 0.0, 0.0, 1.0); // Black border
    } else {
        FragColor = vec4(uColor, 1.0);     // Regular face color
    }
}
