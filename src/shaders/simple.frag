#version 460

layout (location = 0) in vec3 in_color;

layout (location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform UniformFrameData {
    vec4 ambient_color;
} ufd;


void main() {
    out_color = vec4(in_color, 1.0);
}

