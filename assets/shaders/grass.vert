#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;
layout(location=2) in vec2 a_tex_coords;

layout(location=5) in vec4 model_matrix_0;
layout(location=6) in vec4 model_matrix_1;
layout(location=7) in vec4 model_matrix_2;
layout(location=8) in vec4 model_matrix_3;

layout(location=0) out vec4 v_color;
layout(location=1) out vec3 v_world_position;
layout(location=2) out vec3 v_normal;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
    vec4 u_view_position;
    float u_time;
};

// https://www.shadertoy.com/view/XdXBRH
vec2 hash( in vec2 x )  // replace this by something better
{
    const vec2 k = vec2( 0.3183099, 0.3678794 );
    x = x*k + k.yx;
    return fract( 16.0 * k*fract( x.x*x.y*(x.x+x.y)) );
}

// https://en.wikipedia.org/wiki/Worley_noise
float worley(vec2 p) {

    vec2 i_p = floor(p);
    vec2 f_p = fract(p);

    float dist = 1.0;
    for(int y = -1;y <= 1;++y) {
        for(int x = -1;x <= 1;++x) {
            vec2 n = vec2(float(x), float(y));
            vec2 diff = n + hash(i_p + n) - f_p;
            dist = min(dist, length(diff));
        }
    }
    return dist;
}

const vec3 UP = vec3(0.0, 1.0, 0.0);
const vec3 WIND_DIRECTION = vec3(0.0, 0.0, -1.0);
const float MAX_PITCH = 1.39;
const float MAX_YAW = 0.785;
const float VARYING_ANGLE = 0.1745;

const vec3 COLOR_BOTTOM = vec3(20.0/255.0,40.0/255.0,0);
const vec3 COLOR_TOP = vec3(40.0/255.0,80.0/255.0,0);

const float TIME_SCALE = 1.0;
const float WIND_SCALE = 4.0;

// https://en.wikipedia.org/wiki/Rodrigues'_rotation_formula
mat3 mat3_from_axis_angle(float angle, vec3 axis) {
	float s = sin(angle);
	float c = cos(angle);
	float t = 1.0 - c;
	float x = axis.x;
	float y = axis.y;
	float z = axis.z;
    // Matrix: v * c + cross(axis,v) * s + axis*dot(axis,v) * t
	return mat3(
		vec3(t*x*x+c,t*x*y-s*z,t*x*z+s*y),
		vec3(t*x*y+s*z,t*y*y+c,t*y*z-s*x),
		vec3(t*x*z-s*y,t*y*z+s*z,t*z*z+c)
	);
}


void main() {
    mat4 model_matrix = mat4(
        model_matrix_0,
        model_matrix_1,
        model_matrix_2,
        model_matrix_3
    );

    float time = u_time * TIME_SCALE;

    vec2 uv =  (model_matrix * vec4(a_position, -1.0)).xz;

    float wind = pow(worley(uv * WIND_SCALE + WIND_DIRECTION.xz * time), 2.0) * a_tex_coords.y;
    
    float object_influence = sign(abs(min(length(uv) - 0.25, 0.0)));
    vec2 object_dir = normalize(uv);

    mat3 to_model = inverse(mat3(model_matrix));
    vec3 wind_forward = normalize(to_model * WIND_DIRECTION);
    vec3 wind_right = normalize(cross(wind_forward, UP));

    vec2 rot = (hash(uv * 123.3) * VARYING_ANGLE + vec2(abs(sin(time)) * MAX_YAW, MAX_PITCH)) * wind * (1 - object_influence) 
            + MAX_PITCH * object_influence * object_dir;

    mat3 rot_mat = mat3_from_axis_angle(rot.y, wind_right) * mat3_from_axis_angle(rot.x, wind_forward);
    vec3 r_position = rot_mat * a_position;

    vec4 world_position = model_matrix * vec4(r_position, 1.0);
    v_world_position = world_position.xyz / world_position.w;
    gl_Position = u_view_proj * world_position;
    
    v_normal = to_model * inverse(rot_mat) * a_normal;

    float intensity = a_tex_coords.y;

    v_color = vec4(mix(COLOR_BOTTOM, COLOR_TOP, intensity), 1.0);
}
