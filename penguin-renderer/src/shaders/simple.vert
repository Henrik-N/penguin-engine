#version 460

// uniform buffers
// *
layout (set = 0, binding = 0) uniform GPUCameraData {
    vec4 data;
    mat4 proj_view;
} u_camera;

layout (set = 0, binding = 1) uniform GPUObjectDataOld {
    mat4 transform;
} u_object;


struct RenderObjectData {
    mat4 model_transform;
};

layout (std140, set = 1, binding = 0) readonly buffer GPUObjectDataNew {
    RenderObjectData objects[];
} object_buffer;


// vertices
// *
layout (location = 0) in vec3 v_position;
layout (location = 1) in vec3 v_normal;
layout (location = 2) in vec3 v_color;

layout (location = 0) out vec3 frag_color;


void main() {
    mat4 model_matrix = object_buffer.objects[gl_BaseInstance].model_transform;

    mat4 transform_matrix = (u_camera.proj_view * model_matrix);

    gl_Position = transform_matrix * vec4(v_position, 1.0);

    frag_color = v_color;
}
