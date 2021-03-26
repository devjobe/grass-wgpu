#version 450

layout(location=0) in vec4 v_color;
layout(location=1) in vec3 v_world_position;
layout(location=2) in vec3 v_normal;
layout(location=0) out vec4 f_color;

const vec3 LIGHT_COLOR = vec3(1.0, 1.0, 1.0);
const float AMBIENT_STRENGTH = 0.1;
const vec3 LIGHT_DIR = vec3(0.0, 1.0, 1.0);

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
    vec4 u_view_position;
    float u_time;
};

float LinearizeDepth(float depth, float near, float far) 
{
    float z = depth * 2.0 - 1.0; // back to NDC 
    return (2.0 * near * far) / (far + near - z * (far - near));	
}

void main() {
    vec3 light_dir = normalize(LIGHT_DIR);
    vec3 normal = normalize(v_normal);
    float luminance = max(dot(normal, light_dir), 0.0) + AMBIENT_STRENGTH;
    //f_color = vec4(vec3(luminance), 1.0);
    
    vec3 view_dir = normalize(u_view_position.xyz - v_world_position);
    vec3 half_dir = normalize(view_dir + light_dir);
    float specular_strength = pow(max(dot(view_dir, half_dir), 0.0), 32);
    luminance += specular_strength;
    //f_color = vec4(vec3(luminance), 1.0);
    f_color = vec4(v_color.xyz * luminance, v_color.w);
    //f_color = vec4(vec3(LinearizeDepth(gl_FragCoord.z, 0.1, 100.0)), 1.0);

    /*  float intensity = luminance;

 	if (intensity > 0.9) {
 		intensity = 1.1;
 	}
 	else if (intensity > 0.5) {
 		intensity = 0.7;
 	}
 	else {
 		intensity = 0.5;
  }

    f_color = vec4(v_color * intensity, 1.0);


/*
      float intensity = 0.6 * diffuse + 0.4 * spec;

 	if (intensity > 0.9) {
 		intensity = 1.1;
 	}
 	else if (intensity > 0.5) {
 		intensity = 0.7;
 	}
 	else {
 		intensity = 0.5;
  }

	gl_FragColor = gl_Color * intensity;*/
}