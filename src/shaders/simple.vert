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
    //gl_Position = ub.model * vec4(v_position, 1.0);
    gl_Position = ub.render_matrix * vec4(v_position, 1.0);
    //gl_Position = ub.proj * ub.view * ub.model * vec4(v_position, 1.0);

    frag_color = v_color;
}



// vec3 camera_loc = vec3(0.0, 0.0, 3.0);
// vec3 camera_target = vec3(0.0, 0.0, 0.0);
// vec3 camera_dir = normalize(vec3(camera_loc - camera_target));

// vec3 up = vec3(0.0, 1.0, 0.0);
// vec3 camera_right = normalize(cross(up, camera_dir));

// vec3 camera_up = cross(camera_dir, camera_right);

// mat4 view1 = mat4(
//         camera_right.x, camera_right.y, camera_right.z, 0.0,
//         camera_up.x, camera_up.x, camera_up.z, 0.0,
//         camera_dir.x, camera_dir.y, camera_dir.z, 0.0,
//         0.0, 0.0, 0.0, 1.0);

// mat4 pos_mat = mat4(
//         1.0, 0.0, 0.0, -camera_loc.x,
//         0.0, 1.0, 0.0, -camera_loc.y,
//         1.0, 0.0, 1.0, -camera_loc.z,
//         0.0, 0.0, 0.0, 0.0);

// mat4 view = view1 * pos_mat;
// mat4 mvp = view;

