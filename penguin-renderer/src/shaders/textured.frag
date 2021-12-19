#version 460

layout (location = 0) in vec3 color;
layout (location = 1) in vec2 uv;

layout (location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform UniformFrameData {
    vec4 ambient_color;
} ufd;

layout(set = 2, binding = 0) uniform sampler2D tex0;


void main() {
    out_color = vec4(color, 1.0);
    out_color = vec4(uv.x, uv.y, 0.5, 1.0);
    //vec3 color = texture(tex0, uv).xyz;
    //out_color = vec4(color, 1.0);
}

