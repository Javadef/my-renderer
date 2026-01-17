#version 450

// Input from vertex shader
layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec3 fragColor;
layout(location = 2) in vec3 fragWorldPos;

// Output color
layout(location = 0) out vec4 outColor;

void main() {
    // Normalize the interpolated normal
    vec3 normal = normalize(fragNormal);
    
    // Light direction (from top-right-front)
    vec3 lightDir = normalize(vec3(1.0, 1.0, 1.0));
    
    // Ambient light
    float ambient = 0.15;
    
    // Diffuse lighting (Lambertian)
    float diffuse = max(dot(normal, lightDir), 0.0);
    
    // Add a secondary fill light from the opposite side
    vec3 fillLightDir = normalize(vec3(-0.5, 0.3, -0.5));
    float fillDiffuse = max(dot(normal, fillLightDir), 0.0) * 0.3;
    
    // Combine lighting
    float lighting = ambient + diffuse * 0.7 + fillDiffuse;
    
    // Apply lighting to color
    vec3 finalColor = fragColor * lighting;
    
    // Slight gamma correction for better appearance
    finalColor = pow(finalColor, vec3(1.0 / 2.2));
    
    outColor = vec4(finalColor, 1.0);
}
