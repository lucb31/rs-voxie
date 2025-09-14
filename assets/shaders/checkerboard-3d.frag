#version 330 core

in vec2 vUV;            // UV coordinates from vertex shader
out vec4 FragColor;

uniform float checkerSize = 10.0;  // Number of checkers per unit (higher = smaller tiles)
uniform vec3 color1 = vec3(1.0);   // First color (white)
uniform vec3 color2 = vec3(0.0);   // Second color (black)

void main() {
    // Scale UVs to checkerboard grid
    vec2 scaledUV = vUV * checkerSize;

    // Floor to get checker cell indices
    int checkX = int(floor(scaledUV.x));
    int checkY = int(floor(scaledUV.y));

    // XOR the parity of the x and y indices to alternate colors
    bool isEven = (checkX + checkY) % 2 == 0;

    vec3 color = isEven ? color1 : color2;

    FragColor = vec4(color, 0.75);
}
