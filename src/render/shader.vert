#version 450

layout(location = 0) in vec2 vertex_position_2d;
layout(location = 1) in vec4 vertex_color;

layout(location = 0) out vec4 v_color;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

void main() {
    gl_Position = ViewProj * vec4(vertex_position_2d, 0.0, 1.0);
    v_color = vertex_color;
}