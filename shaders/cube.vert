#version 450

// Input attributes
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec3 inColor;

// Output to fragment shader
layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec3 fragColor;
layout(location = 2) out vec3 fragWorldPos;

// Push constant for MVP matrix and model matrix for normals
layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;  // For transforming normals
} push;

void main() {
    gl_Position = push.mvp * vec4(inPosition, 1.0);
    
    // Transform normal to world space (using model matrix)
    fragNormal = mat3(push.model) * inNormal;
    fragColor = inColor;
    fragWorldPos = (push.model * vec4(inPosition, 1.0)).xyz;
}
