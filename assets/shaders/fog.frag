#version 330 core
out vec4 FragColor;

in vec2 vTexCoords;

uniform sampler2D screenTexture;
uniform sampler2D depthTexture;

float near = 0.1;
float far  = 1000.0;

// "Sea blue"
uniform vec3 fog_color = vec3(0.0, 0.411, 0.58);

// Below this threshold no fog will be displayed at all
uniform float min_fog_distance = 30.0;
// Formula to calculate based on min & max distance 
// fog_density = ln(0.01) / (max_fog_distance - min_fog_distance)
// Default value here for min = 30, max = 100
uniform float fog_density = -0.06;

float LinearizeDepth(float depth)
{
    float z = depth * 2.0 - 1.0; // back to NDC
    return (2.0 * near * far) / (far + near - z * (far - near));
}

float calc_fog_factor(float linear_depth) {
  // Distance threshold
  float offset_depth = linear_depth - min_fog_distance;
  return clamp(1.0 - exp(fog_density * offset_depth), 0.0, 1.0);
}

void main()
{
  // Sample & linearize depth
  float raw_depth = texture(depthTexture, vTexCoords).r;
  // [0.1, 1000.0]
  float linear_depth = LinearizeDepth(raw_depth);
  float fog_factor = calc_fog_factor(linear_depth);

  // sample frame color
  vec3 frame_color = texture(screenTexture, vTexCoords).rgb;

  // blend frame color with fog color
  vec3 final_color = mix(frame_color, fog_color, fog_factor);
  FragColor = vec4(final_color, 1.0);
}
