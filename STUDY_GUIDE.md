# 🎮 Vulkan Renderer Study Guide

A practical guide to learning graphics programming through this project.
**Current Status:** Phase 1 Complete ✅ | Running at 5000+ FPS

---

## 📚 Quick Start

```powershell
# Run in release mode (validation layers disabled) - RECOMMENDED
cargo run --release

# Run in debug mode (requires Vulkan SDK for validation layers)
# Install from: https://vulkan.lunarg.com/sdk/home
cargo run

# Enable verbose logging
$env:RUST_LOG="debug"; cargo run --release
```

### Controls
| Key | Action |
|-----|--------|
| `ESC` | Exit application |
| Window resize | Automatic swapchain recreation |

---

## 🏗️ Project Architecture

### High-Level Overview
```
┌─────────────────────────────────────────────────────────────────────────┐
│                           APPLICATION (main.rs)                         │
│                                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  ┌──────────────┐  │
│  │   Window    │  │  Event Loop  │  │   Render    │  │     FPS      │  │
│  │  (winit)    │  │   Handling   │  │    Loop     │  │   Tracking   │  │
│  └──────┬──────┘  └──────────────┘  └──────┬──────┘  └──────────────┘  │
└─────────┼────────────────────────────────────┼──────────────────────────┘
          │                                    │
┌─────────┼────────────────────────────────────┼──────────────────────────┐
│         │           BACKEND (backend/)       │                          │
│  ┌──────▼──────┐  ┌──────────────┐  ┌───────▼──────┐  ┌─────────────┐  │
│  │   Surface   │  │   Device     │  │  Swapchain   │  │    Sync     │  │
│  │  (window    │  │  (GPU +      │  │  (images to  │  │  (fences,   │  │
│  │  connection)│  │   queues)    │  │   render to) │  │ semaphores) │  │
│  └─────────────┘  └──────────────┘  └──────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### File Structure
```
src/
├── main.rs              # 🎯 START HERE - All application logic (765 lines)
│   ├── App struct       # Holds all Vulkan resources
│   ├── init_vulkan()    # Setup: device, surface, swapchain
│   ├── render_frame()   # Hot path: acquire → submit → present
│   └── Drop impl        # Cleanup in reverse order
│
├── lib.rs               # Library exports (for future use)
│
└── backend/             # Vulkan abstraction layer
    ├── mod.rs           # Module exports
    ├── device.rs        # GPU selection, queues, validation (339 lines)
    ├── swapchain.rs     # Image management, present modes
    └── sync.rs          # Fences & semaphores
```

---

## 📖 Code Reading Order (Recommended)

### Step 1: Understand the Frame Loop (main.rs)
Read these functions in order:

| Order | Function | Lines | What You'll Learn |
|-------|----------|-------|-------------------|
| 1️⃣ | `main()` | 52-66 | Entry point, event loop creation |
| 2️⃣ | `App::new()` | 120-140 | What state the renderer needs |
| 3️⃣ | `init_vulkan()` | 163-240 | How Vulkan initializes |
| 4️⃣ | `render_frame()` | 462-590 | **THE HOT PATH** - runs every frame |
| 5️⃣ | `record_command_buffers()` | 360-450 | GPU commands (barriers, clear) |
| 6️⃣ | `Drop::drop()` | 710-760 | Cleanup order matters! |

### Step 2: Understand Synchronization (critical!)
```
Frame N:    [Acquire Image]──[Wait Fence]──[Submit]──[Present]
                  │                │           │          │
                  │                │           │          └─ Signal: render_finished
                  │                │           └─ Signal: in_flight_fence
                  │                └─ Wait: in_flight_fence (from N-2)
                  └─ Signal: image_available
```

### Step 3: Dive into Backend (when curious)
| File | Read When... | Key Insight |
|------|-------------|-------------|
| `device.rs` | "How does GPU selection work?" | Prefers discrete GPU over integrated |
| `swapchain.rs` | "What are present modes?" | IMMEDIATE=no vsync, FIFO=vsync |
| `sync.rs` | "Why fences AND semaphores?" | Different sync granularity |

---

## 🧠 Core Concepts Explained

### 1. Why "Frames in Flight"?
```
MAX_FRAMES_IN_FLIGHT = 2

Without pipelining:
  CPU: [Record]────────────────[Record]────────────────
  GPU: ────────[Render]────────────────[Render]────────
                       ↑ IDLE!              ↑ IDLE!

With 2 frames in flight:
  CPU: [Record 0][Record 1][Record 0][Record 1]
  GPU: ─────────[Render 0][Render 1][Render 0]
                    └── CPU and GPU work in parallel!
```

**Trade-off:**
- More frames = smoother but more latency
- Fewer frames = lower latency but potential stalls
- 2 is the sweet spot for most applications

### 2. Fences vs Semaphores
```
┌────────────────────────────────────────────────────────────┐
│ SEMAPHORE = GPU waits for GPU                              │
│                                                            │
│   Queue 1: ──[Render]──Signal───────────────               │
│   Queue 2: ─────────────────────Wait──[Present]            │
│                                                            │
│   Use case: "Don't present until rendering is done"        │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│ FENCE = CPU waits for GPU                                  │
│                                                            │
│   GPU: ──[Render]──Signal──────────                        │
│   CPU: ────────────────────Wait──[Reuse command buffer]    │
│                                                            │
│   Use case: "Don't reuse resources until GPU is done"      │
└────────────────────────────────────────────────────────────┘
```

### 3. Image Layout Transitions
Images must be in the correct "layout" for each operation:

```
UNDEFINED ──barrier──> TRANSFER_DST ──barrier──> PRESENT_SRC
    │                       │                         │
    │                       │                         └─ Ready to show on screen
    │                       └─ Ready for clear/copy operations
    └─ "I don't care what's in here"

WHY? GPU memory is organized differently for different operations.
     Layout transitions reorganize memory for optimal access.
```

### 4. Why Pre-recorded Command Buffers?
```
Recording commands has CPU cost:
  - Validation
  - Memory allocation
  - State tracking

For STATIC content (clearing screen):
  ✅ Record once at startup
  ✅ Submit same buffer every frame
  ✅ Zero per-frame recording cost

For DYNAMIC content (moving objects):
  ❌ Must re-record every frame
  💡 Or use secondary command buffers for dynamic parts
```

### 5. Present Modes Explained
```
┌─────────────────────────────────────────────────────────────┐
│ IMMEDIATE (current) - No VSync                              │
│   Frame: [1][2][3][4][5][6][7][8][9]...                     │
│   ✅ Lowest latency, highest FPS                            │
│   ❌ Screen tearing possible                                │
│   📊 Use for: Benchmarking, competitive gaming              │
├─────────────────────────────────────────────────────────────┤
│ MAILBOX - "Fast VSync"                                      │
│   Frame: [1]──[2]──[3]──  (shows latest ready frame)        │
│   ✅ No tearing, low latency                                │
│   ❌ Uses more power (renders frames that may not show)     │
│   📊 Use for: Action games                                  │
├─────────────────────────────────────────────────────────────┤
│ FIFO - True VSync                                           │
│   Frame: [1]──wait──[2]──wait──[3]──                        │
│   ✅ No tearing, power efficient                            │
│   ❌ Higher input latency                                   │
│   📊 Use for: Movies, turn-based games                      │
└─────────────────────────────────────────────────────────────┘
```

---

## 🚀 Learning Path

### ✅ Phase 1: Foundation (COMPLETE!)
- [x] Window creation with winit
- [x] Vulkan instance & device setup
- [x] Surface creation (Win32)
- [x] Swapchain creation
- [x] Command pool & buffers
- [x] Synchronization (fences + semaphores)
- [x] Clear screen to color
- [x] Window resize handling
- [x] FPS tracking
- [x] Pre-recorded command buffers
- [x] IMMEDIATE present mode (low latency)

**You achieved:** 5000+ FPS, 0.1ms frame time, ~2% GPU usage

### 📋 Phase 2: First Triangle (NEXT!)
- [ ] Vertex buffer with positions + colors
- [ ] Vertex shader (transform positions)
- [ ] Fragment shader (output colors)
- [ ] Shader compilation (GLSL → SPIR-V)
- [ ] Graphics pipeline
- [ ] Render pass & framebuffers
- [ ] Draw commands (`vkCmdDraw`)

### 📋 Phase 3: 3D Transforms
- [ ] Uniform buffers (MVP matrices)
- [ ] Descriptor sets (shader inputs)
- [ ] Depth buffer (z-sorting)
- [ ] Index buffers (efficient meshes)
- [ ] Camera controls (WASD + mouse)

### 📋 Phase 4: Texturing
- [ ] Image loading (PNG/JPG)
- [ ] Vulkan images & image views
- [ ] Samplers (filtering, wrapping)
- [ ] UV coordinates
- [ ] Combined image samplers

### 📋 Phase 5: Lighting
- [ ] Normal vectors
- [ ] Phong shading
- [ ] Multiple lights
- [ ] Shadow mapping basics

### 📋 Phase 6: Advanced (Kajiya-level)
- [ ] Deferred rendering
- [ ] PBR materials
- [ ] Ray tracing
- [ ] Global illumination

---

## 🔧 Common Tasks & Experiments

### Change the Clear Color
In `main.rs`, find `record_command_buffers()`:
```rust
let clear_color = vk::ClearColorValue {
    float32: [0.1, 0.2, 0.8, 1.0], // R, G, B, A (0.0-1.0)
};
```
Try: `[1.0, 0.0, 0.0, 1.0]` for red, `[0.0, 0.0, 0.0, 1.0]` for black

### Change Window Size
In `main.rs`, find `resumed()`:
```rust
.with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080)) // Full HD
```

### Enable VSync (limit to monitor refresh rate)
In `swapchain.rs`, find present mode selection:
```rust
// Change from IMMEDIATE to FIFO
.unwrap_or(vk::PresentModeKHR::FIFO)
```

### Add More Frames in Flight
In `main.rs`, change the constant:
```rust
const MAX_FRAMES_IN_FLIGHT: usize = 3; // Try 3 for comparison
```
Watch: Does latency feel different? Does FPS change?

---

## 🐛 Debugging Tips

### Enable Validation Layers
1. Install [Vulkan SDK](https://vulkan.lunarg.com/sdk/home)
2. Run WITHOUT `--release`: `cargo run`
3. Watch console for validation errors

### Common Errors & Fixes

| Error | Cause | Fix |
|-------|-------|-----|
| "Layer not found" | Vulkan SDK missing | Install SDK or use `--release` |
| "Swapchain out of date" | Window resized | Handled automatically ✅ |
| "Device lost" | GPU crash | Check memory access patterns |
| Black screen | Missing barrier | Verify image layout transitions |
| Frozen window | Infinite wait | Check fence/semaphore logic |
| Low FPS | Wrong present mode | Check IMMEDIATE is selected |

### Performance Monitoring
**MSI Afterburner + RivaTuner (recommended setup):**
- GPU Temperature
- GPU Usage (%)
- Core Clock (MHz)
- Framerate (FPS)
- Frametime (ms)
- Framerate 1% Low

**In-app (already implemented):**
- Window title shows FPS and frame time

### GPU Debugging Tools
- [RenderDoc](https://renderdoc.org/) - Free, cross-platform
- [NVIDIA Nsight](https://developer.nvidia.com/nsight-graphics) - NVIDIA GPUs
- [AMD Radeon GPU Profiler](https://gpuopen.com/rgp/) - AMD GPUs

---

## 📚 Resources

### This Project
| File | Description |
|------|-------------|
| `STUDY_GUIDE.md` | You're reading it! |
| `ARCHITECTURE.md` | High-level design decisions |
| `README.md` | Quick project overview |
| `LEARNING_LOG.md` | Track your progress |

### Official Documentation
- [Vulkan Specification](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/) - The source of truth
- [Vulkan Tutorial](https://vulkan-tutorial.com/) - Best beginner tutorial
- [Vulkan Guide](https://github.com/KhronosGroup/Vulkan-Guide) - Best practices

### Rust-Specific
- [Ash Documentation](https://docs.rs/ash/latest/ash/) - The Vulkan bindings we use
- [gpu-allocator](https://docs.rs/gpu-allocator/latest/gpu_allocator/) - Memory allocation
- [winit](https://docs.rs/winit/latest/winit/) - Windowing library

### Graphics Theory
- [Learn OpenGL](https://learnopengl.com/) - Concepts apply to Vulkan
- [Scratchapixel](https://www.scratchapixel.com/) - Math & theory
- [Real-Time Rendering](https://www.realtimerendering.com/) - The bible

### Video Resources
- [ThinMatrix Vulkan Tutorial](https://www.youtube.com/playlist?list=PLRIWtICgwaX0u7Rf9zkZhLoLuZVfUksDP) - Excellent visual explanations
- [Vulkan Lecture Series](https://www.youtube.com/playlist?list=PLmIqTlJ6KsE1Jx5HV4sd2jOe3V1KMHHgn) - Deep dives

---

## 💡 Learning Tips

### Do This ✅
1. **Read code top-to-bottom** - main.rs is designed to be read linearly
2. **Follow the comments** - They explain WHY, not just WHAT
3. **Experiment!** - Change values, break things, see what happens
4. **Use validation layers** - They catch 90% of bugs
5. **Draw diagrams** - Sketch the frame flow, sync timeline
6. **Track your progress** - Update LEARNING_LOG.md

### Avoid This ❌
1. Don't memorize API calls - Understand concepts instead
2. Don't skip synchronization - It's the hardest but most important
3. Don't ignore validation errors - They're always right
4. Don't optimize prematurely - Get it working first

### When Stuck
1. Re-read the comments in the code
2. Check validation layer output
3. Draw the sync timeline
4. Simplify - comment out code until it works
5. Compare with vulkan-tutorial.com

---

## 🎯 Next Steps

Ready for Phase 2? Here's what you'll add:

```
NEW FILES:
  src/shaders/
    ├── triangle.vert      # Vertex shader (GLSL)
    ├── triangle.frag      # Fragment shader (GLSL)
    ├── triangle.vert.spv  # Compiled vertex shader
    └── triangle.frag.spv  # Compiled fragment shader
  
  src/backend/
    ├── pipeline.rs        # Graphics pipeline
    └── buffer.rs          # Vertex/index buffers

MODIFIED:
  src/main.rs
    ├── Add render pass
    ├── Add framebuffers
    ├── Update command recording
    └── Add draw commands
```

**The goal:** See a colorful spinning triangle!

---

*Last updated: January 2026*
*Happy rendering! 🎨*
