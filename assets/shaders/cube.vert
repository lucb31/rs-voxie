#version 330 core
uniform mat4 uMVP;
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;

out vec3 fragNormal;

void main() {
    fragNormal = aNormal;
    gl_Position = uMVP * vec4(aPos, 1.0);
}
