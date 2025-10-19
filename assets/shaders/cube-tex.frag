#version 330 core

in vec2 vTexCoord;
out vec4 FragColor;

uniform sampler2D tex;

void main() {
    FragColor = vec4(texture(tex, vTexCoord));
}
