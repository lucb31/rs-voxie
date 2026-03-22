#version 330 core
out vec4 FragColor;

in vec2 vTexCoords;

uniform sampler2D screenTexture;
uniform sampler2D depthTexture;

float near = 0.1;
float far  = 1000.0;

// "Sea blue"
uniform vec3 fog_color = vec3(0.0, 0.411, 0.58);
uniform float max_fog_at_distance = 50.0;
uniform float fog_density = 2.0;


float LinearizeDepth(float depth)
{
    float z = depth * 2.0 - 1.0; // back to NDC
    return (2.0 * near * far) / (far + near - z * (far - near));
}

float calc_fog_factor(float x) {
  // Distance threshold
  // float effective_depth = max(x - 0.01, 0.0);
  float effective_depth = x;
  return 1.0 - exp(-effective_depth * fog_density);
}

void main()
{
  // Sample & linearize depth
  float raw_depth = texture(depthTexture, vTexCoords).r;
  // [0.1, 1000.0]
  float linear_depth = LinearizeDepth(raw_depth);
  // [0, 1]
  float normalized_depth = linear_depth / max_fog_at_distance;
  float fog_factor = calc_fog_factor(normalized_depth);

  // sample frame color
  vec3 frame_color = texture(screenTexture, vTexCoords).rgb;

  // blend frame color with fog color
  vec3 final_color = mix(frame_color, fog_color, fog_factor);
  FragColor = vec4(final_color, 1.0);
}
