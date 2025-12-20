#version 330 core

in vec2 worldPos;           // interpolated world-space XY position of fragment
flat in float sphereRadius; // world-space radius
flat in vec2 sphereCenter;  // world-space center (XY)

out vec4 FragColor;

// parameters
uniform vec3 uColor;

void main()
{
    // vector from center to fragment in XY plane
    vec2 p = worldPos - sphereCenter;

    float dist = length(p);

    // outside the circle: discard
    if(dist > sphereRadius){
        discard;
    }

    FragColor = vec4(uColor, 1.0);
}
