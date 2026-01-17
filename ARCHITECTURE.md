# High-Performance Renderer Architecture

**Design Philosophy:** Performance-first, Kajiya-inspired, incremental complexity

## Core Principles

1. **Zero-copy where possible** - Minimize CPU->GPU transfers
2. **Explicit synchronization** - No hidden overhead
3. **Memory pooling** - Reuse allocations
4. **GPU-driven rendering** - Minimize CPU bottlenecks
5. **Temporal techniques** - Amortize cost over frames

## Architecture Layers

```
┌─────────────────────────────────────────┐
│         Application Layer               │  ← High-level rendering logic
├─────────────────────────────────────────┤
│         Render Graph                    │  ← Automatic resource management
├─────────────────────────────────────────┤
│         Rendering Backend               │  ← Vulkan abstraction
├─────────────────────────────────────────┤
│         Raw Vulkan (ash)                │  ← Direct GPU control
└─────────────────────────────────────────┘
```

## Phase Roadmap

### Phase 1: Vulkan Foundation (Current)
- [x] Project setup
- [ ] Vulkan instance + device
- [ ] Swapchain + presentation
- [ ] Command buffers
- [ ] Synchronization primitives
- **Goal:** Clear screen at 60+ FPS, <1ms frame time

### Phase 2: Basic Rasterization
- [ ] Vertex/index buffers
- [ ] Graphics pipeline
- [ ] Descriptor sets
- [ ] Simple shaders (GLSL)
- **Goal:** Render 1M triangles at 144 FPS

### Phase 3: PBR Materials
- [ ] GGX BRDF
- [ ] Image-based lighting
- [ ] Texture streaming
- **Goal:** Physically accurate materials

### Phase 4: Compute Pipeline
- [ ] Compute shaders
- [ ] GPU culling
- [ ] Indirect rendering
- **Goal:** 10M triangles with culling

### Phase 5: Ray Tracing (Kajiya-style)
- [ ] Acceleration structures
- [ ] Ray tracing pipeline
- [ ] ReSTIR (reservoir sampling)
- [ ] Temporal accumulation
- **Goal:** Real-time path tracing

### Phase 6: Advanced GI
- [ ] Irradiance cache
- [ ] Screen-space techniques
- [ ] Denoising (SVGF-style)
- **Goal:** Multi-bounce GI in <16ms

## Performance Targets

| Phase | Triangle Count | FPS (1080p) | Frame Time |
|-------|---------------|-------------|------------|
| 1     | 0 (clear)     | Unlimited   | <0.1ms     |
| 2     | 1M            | 144+        | <7ms       |
| 3     | 1M            | 144+        | <7ms       |
| 4     | 10M           | 60+         | <16ms      |
| 5     | 1M            | 60          | <16ms      |
| 6     | 1M            | 60          | <16ms      |

## Key Differences from Kajiya

**Improvements:**
- Cleaner separation of concerns
- Better documented
- More modular render graph
- Easier to extend

**Simplifications (for now):**
- No rust-gpu shaders (use GLSL/HLSL)
- Simpler asset pipeline
- Windows-only initially
