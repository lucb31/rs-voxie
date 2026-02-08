#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;

layout(std140) uniform FrameUniforms {
    float u_time;
};

uniform mat4 uModel;
uniform mat3 uModelIV;
uniform mat4 uView;
uniform mat4 uProjection;

out vec3 vNormal;
out vec3 vPos;
out vec2 vTexCoord;

vec3 pulse(vec3 pos) {
// Min z: -29.4
// Max z: 30.9
// Body: z 3 - 15

  // Normalize z value in body region to interval [0;1]
//  float t = clamp((pos.z - 3.0) / (15.0 - 3.0), 0.0, 1.0);
  // Bell shaped intensity: Maximum in middle of interval
//  float vertexIntensity = 4.0 * t * (1.0 - t);

  float pulseSpeed = 1.0;
  float pulseAmplitude = 0.05;
  float pulse = 1.0 + sin(u_time * pulseSpeed) * pulseAmplitude;
  vec3 localPos = pos;
  localPos.z *= pulse;
  return localPos;
}


vec3 tentacle_animation(vec3 pos) {
  float tentacleWeight = clamp(pos.z / 30.0, 0.0, 1.0);

  // Use weight to create traveling wave from base to tip
  float frequency = 0.3;     // number of waves along tentacle
  float amplitude = 1.0;     // maximum displacement
  float speed     = 4.0;     // wave speed
  vec3 localPos = pos; // position in object space

  float wave = sin(localPos.z * frequency - u_time * speed) * amplitude * tentacleWeight;
  localPos.x += wave;  // sideways sway
  localPos.y += wave * 0.5; // optional vertical sway

  return localPos;
}

void main() {
  // Calculate vertex position
  vec3 displacedVertex = tentacle_animation(aPos);
  displacedVertex = pulse(displacedVertex);
  vPos = vec3(uModel * vec4(displacedVertex, 1.0));

  // Calculate normals with inverse transpose
  vNormal = uModelIV * aNormal;
  vTexCoord = aTexCoord;
  gl_Position = uProjection * uView * vec4(vPos, 1.0);
}
