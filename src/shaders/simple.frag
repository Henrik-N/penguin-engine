#version 450

layout (location = 0) in vec3 in_color;

layout (location = 0) out vec4 out_color;


void main() {
    out_color = vec4(in_color, 1.0);
    
    // vec3 light = vec3(-1.0, -1.0, -1.0);
    // vec4(clamp(dot(in_normal, -light), 0.0, 1.0) * vec3(1.0, 0.93, 0.56), 1.0);

    //outColor = vec4(inColor, 1.0);
    // outColor = vec4(1.0, 0.0, 0.0, 1.0);
}
