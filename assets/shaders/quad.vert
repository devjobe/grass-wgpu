#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;
layout(location=2) in vec2 a_tex_coords;

layout(location=5) in vec4 model_matrix_0;
layout(location=6) in vec4 model_matrix_1;
layout(location=7) in vec4 model_matrix_2;
layout(location=8) in vec4 model_matrix_3;
layout(location=9) in vec4 a_color;

layout(location=0) out vec4 v_color;
layout(location=1) out vec3 v_world_position;
layout(location=2) out vec3 v_normal;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};


void main() {
    v_color = a_color;

    mat4 model_matrix = mat4(
        model_matrix_0,
        model_matrix_1,
        model_matrix_2,
        model_matrix_3
    );

    v_normal = mat3(transpose(inverse(model_matrix))) * a_normal;
    vec4 world_position  = model_matrix * vec4(a_position, 1.0);
    v_world_position = world_position.xyz / world_position.w;
    gl_Position = u_view_proj * world_position;
}
