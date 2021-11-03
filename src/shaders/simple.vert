#version 450


layout (set = 0, binding = 0) uniform UniformBufferGlobalData {
    vec4 data;
    mat4 render_matrix;
    // mat4 model;
    // mat4 view;
    // mat4 proj;
} ub;

layout (location = 0) in vec3 v_position;
layout (location = 1) in vec3 v_normal;
layout (location = 2) in vec3 v_color;


layout (location = 0) out vec3 frag_color;



void main() {
    gl_Position = ub.render_matrix * vec4(v_position, 1.0);

    frag_color = v_color;
}

