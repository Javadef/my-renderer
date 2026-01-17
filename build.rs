// Build script to compile GLSL shaders to SPIR-V

use std::process::Command;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=shaders/");
    
    // Compile shaders using glslc (part of Vulkan SDK)
    compile_shader("shaders/cube.vert", "shaders/cube.vert.spv");
    compile_shader("shaders/cube.frag", "shaders/cube.frag.spv");
}

fn compile_shader(input: &str, output: &str) {
    let input_path = Path::new(input);
    let output_path = Path::new(output);
    
    // Check if glslc is available
    let result = Command::new("glslc")
        .arg(input_path)
        .arg("-o")
        .arg(output_path)
        .status();
    
    match result {
        Ok(status) if status.success() => {
            println!("Compiled {} -> {}", input, output);
        }
        Ok(status) => {
            panic!("Failed to compile {}: exit code {:?}", input, status.code());
        }
        Err(e) => {
            eprintln!("Warning: glslc not found ({})", e);
            eprintln!("Shaders will not be compiled. Install Vulkan SDK or compile manually:");
            eprintln!("  glslc {} -o {}", input, output);
        }
    }
}
