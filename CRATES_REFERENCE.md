# Rust Crates Reference Guide

A comprehensive reference of Rust crates commonly used in game engines and renderers.
Organized by category for easy navigation.

---

## Table of Contents
- [🎨 Graphics & Rendering](#-graphics--rendering)
- [🖼️ UI Libraries](#️-ui-libraries)
- [🎮 Game Engine / ECS](#-game-engine--ecs)
- [🔢 Math & Linear Algebra](#-math--linear-algebra)
- [⚛️ Physics](#️-physics)
- [🖼️ Image & Texture](#️-image--texture)
- [🔤 Text & Fonts](#-text--fonts)
- [📁 File Formats](#-file-formats)
- [🔊 Audio](#-audio)
- [🌐 Networking](#-networking)
- [🔐 Cryptography](#-cryptography)
- [⚡ Async & Concurrency](#-async--concurrency)
- [📊 Profiling & Debugging](#-profiling--debugging)
- [🛠️ Utilities](#️-utilities)
- [📝 Serialization](#-serialization)
- [🪟 Windowing & Input](#-windowing--input)
- [💻 System & Platform](#-system--platform)
- [📜 Parsing & Text](#-parsing--text)
- [🔧 Proc Macros & Derive](#-proc-macros--derive)
- [🎲 Random & Noise](#-random--noise)

---

## 🎨 Graphics & Rendering

### Vulkan
| Crate | Description | Use Case |
|-------|-------------|----------|
| **ash** | Low-level Vulkan bindings for Rust | Direct Vulkan API access |
| **ash-window** | Window surface creation for ash | Connect Vulkan to windows |
| **gpu-allocator** | GPU memory allocation | Efficient VRAM management |
| **gpu-profiler** | GPU timing/profiling | Measure GPU performance |
| **vk-sync-fork** | Vulkan synchronization helpers | Simplify barriers/sync |

### Shaders & SPIR-V
| Crate | Description | Use Case |
|-------|-------------|----------|
| **rspirv** | SPIR-V parsing and building | Shader manipulation |
| **spirv** | SPIR-V types and constants | Shader tooling |
| **spirq** | SPIR-V reflection | Extract shader info |
| **shader-prepper** | Shader preprocessing | Include directives, etc. |

### Rendering Techniques
| Crate | Description | Use Case |
|-------|-------------|----------|
| **kajiya** | Hybrid rendering research renderer | Reference implementation |
| **GPU-Raytracer** | GPU raytracing examples | Learning raytracing |
| **FidelityFX** | AMD FidelityFX effects | FSR, CAS, etc. |
| **XeGTAO** | Intel XeGTAO ambient occlusion | Screen-space AO |
| **intel_tex_2** | Intel texture compression | BC1-BC7 compression |

### Software Rendering
| Crate | Description | Use Case |
|-------|-------------|----------|
| **softbuffer** | Software framebuffer | CPU rendering fallback |
| **tiny-skia** | 2D software rasterizer | CPU 2D graphics |
| **tiny-skia-path** | Path rendering for tiny-skia | Vector graphics |

---

## 🖼️ UI Libraries

### Immediate Mode UI
| Crate | Description | Use Case |
|-------|-------------|----------|
| **egui** | Easy-to-use immediate mode GUI | Debug panels, tools |
| **egui-winit** | egui + winit integration | Window event handling |
| **egui_extras** | Extra widgets for egui | Tables, images, etc. |
| **epaint** | 2D painting for egui | Low-level rendering |
| **emath** | Math types for egui | Vectors, rects |
| **ecolor** | Color types for egui | Color manipulation |

### Retained Mode UI
| Crate | Description | Use Case |
|-------|-------------|----------|
| **iced** | Elm-inspired GUI library | Full applications |
| **iced_core** | Core types for iced | Building blocks |
| **iced_widget** | Widget library for iced | Buttons, text, etc. |
| **iced_winit** | iced + winit integration | Window handling |
| **iced_graphics** | Graphics backend for iced | Rendering |
| **iced_tiny_skia** | CPU rendering for iced | Software fallback |

### Layout
| Crate | Description | Use Case |
|-------|-------------|----------|
| **taffy** | Flexbox/Grid layout engine | UI layout |
| **kurbo** | 2D curve library | Beziers, paths |

---

## 🎮 Game Engine / ECS

### Bevy (Modular Game Engine)
| Crate | Description | Use Case |
|-------|-------------|----------|
| **bevy_app** | Application framework | Main loop, plugins |
| **bevy_ecs** | Entity Component System | Game objects |
| **bevy_core** | Core types | Time, names, etc. |
| **bevy_input** | Input handling | Keyboard, mouse, gamepad |
| **bevy_math** | Math types (glam-based) | Transforms, vectors |
| **bevy_time** | Time and timers | Delta time, stopwatch |
| **bevy_tasks** | Async task pool | Parallel processing |
| **bevy_log** | Logging setup | Debug output |
| **bevy_reflect** | Runtime reflection | Serialization, editors |
| **bevy_state** | State machines | Game states |
| **bevy_utils** | Utility types | HashMaps, etc. |
| **bevy_ptr** | Pointer utilities | Unsafe helpers |

---

## 🔢 Math & Linear Algebra

### Core Math
| Crate | Description | Use Case |
|-------|-------------|----------|
| **glam** | Fast SIMD math library | Vectors, matrices, quaternions |
| **nalgebra** | Full linear algebra | Complex math, physics |
| **nalgebra-macros** | Macros for nalgebra | Matrix construction |
| **mint** | Math interop types | Library compatibility |
| **euclid** | 2D/3D geometry | Typed geometry |
| **approx** | Approximate comparisons | Float equality |
| **simba** | SIMD algebra traits | Generic math |

### Specialized Math
| Crate | Description | Use Case |
|-------|-------------|----------|
| **half** | 16-bit floats | GPU data, compression |
| **ordered-float** | Orderable floats | Sorting, HashMaps |
| **num-traits** | Numeric traits | Generic numbers |
| **num-complex** | Complex numbers | Signal processing |
| **num-rational** | Rational numbers | Exact fractions |
| **num-integer** | Integer utilities | GCD, LCM, etc. |
| **lerp** | Linear interpolation | Animation, blending |
| **constgebra** | Const linear algebra | Compile-time math |

---

## ⚛️ Physics

### Physics Engines
| Crate | Description | Use Case |
|-------|-------------|----------|
| **rapier3d** | 3D physics engine | Rigid body simulation |
| **parry3d** | 3D collision detection | Shapes, queries |

### Spatial Data Structures
| Crate | Description | Use Case |
|-------|-------------|----------|
| **rstar** | R*-tree spatial index | Nearest neighbor queries |
| **obvhs** | BVH acceleration structure | Raytracing, collision |
| **spade** | Delaunay triangulation | Mesh generation |

---

## 🖼️ Image & Texture

### Image Loading/Saving
| Crate | Description | Use Case |
|-------|-------------|----------|
| **image** | Image loading/saving | PNG, JPEG, etc. |
| **png** | PNG codec | PNG files |
| **jpeg-decoder** | JPEG decoder | JPEG loading |
| **gif** | GIF codec | Animated GIFs |
| **tiff** | TIFF codec | TIFF files |
| **qoi** | QOI codec | Fast lossless format |
| **webp** | WebP support | Web images |
| **exr** | OpenEXR HDR images | HDR lighting |
| **ddsfile** | DDS texture format | GPU textures |
| **color_quant** | Color quantization | Palette reduction |
| **kamadak-exif** | EXIF metadata | Photo info |

### Image Processing
| Crate | Description | Use Case |
|-------|-------------|----------|
| **fast-srgb8** | Fast sRGB conversion | Color space |
| **zune-jpeg** | Fast JPEG decoder | Performance |
| **zune-core** | Image processing core | Filters |
| **zune-inflate** | Zlib decompression | PNG, etc. |

---

## 🔤 Text & Fonts

### Font Loading
| Crate | Description | Use Case |
|-------|-------------|----------|
| **fontdb** | Font database | System fonts |
| **ttf-parser** | TrueType parser | Font loading |
| **owned_ttf_parser** | Owned TTF parser | Font ownership |
| **read-fonts** | Font reading | Glyph data |
| **font-types** | Font type definitions | Shared types |

### Text Shaping & Rendering
| Crate | Description | Use Case |
|-------|-------------|----------|
| **cosmic-text** | Text layout engine | Multi-line text |
| **rustybuzz** | Text shaping (HarfBuzz) | Complex scripts |
| **swash** | Font rendering | Glyph rasterization |
| **ab_glyph** | Glyph rendering | Simple text |
| **ab_glyph_rasterizer** | Glyph rasterizer | Font rendering |
| **zeno** | 2D rasterization | Path filling |

### Unicode
| Crate | Description | Use Case |
|-------|-------------|----------|
| **unicode-bidi** | Bidirectional text | RTL languages |
| **unicode-normalization** | Unicode normalization | Text processing |
| **unicode-segmentation** | Word/grapheme splitting | Text editing |
| **unicode-linebreak** | Line breaking | Text wrapping |
| **unicode-script** | Script detection | Language detection |

---

## 📁 File Formats

### Archives & Compression
| Crate | Description | Use Case |
|-------|-------------|----------|
| **zip** | ZIP archives | Asset packaging |
| **flate2** | DEFLATE compression | ZIP, PNG |
| **zstd** | Zstandard compression | Fast compression |
| **lzma-rs** | LZMA compression | High ratio |
| **brotli** | Brotli compression | Web assets |
| **miniz_oxide** | Pure Rust DEFLATE | No C deps |
| **lzxd** | LZX decompression | CAB files |
| **cab** | CAB archives | Windows installers |
| **ruzstd** | Pure Rust Zstd | No C deps |

### Data Formats
| Crate | Description | Use Case |
|-------|-------------|----------|
| **ron** | Rusty Object Notation | Config files |
| **toml** | TOML parser | Config files |
| **yaml-rust2** | YAML parser | Config files |
| **csv** | CSV parsing | Data import |
| **pdb2** | PDB debug info | Symbol debugging |

---

## 🔊 Audio

| Crate | Description | Use Case |
|-------|-------------|----------|
| **libfmod** | FMOD bindings | Professional audio |

---

## 🌐 Networking

### HTTP
| Crate | Description | Use Case |
|-------|-------------|----------|
| **reqwest** | HTTP client | API calls |
| **hyper** | HTTP implementation | Low-level HTTP |
| **hyper-rustls** | TLS for hyper | HTTPS |
| **http** | HTTP types | Headers, methods |
| **http-body** | HTTP body trait | Streaming |
| **httparse** | HTTP parsing | Low-level |
| **tiny_http** | Simple HTTP server | Dev servers |

### TLS/SSL
| Crate | Description | Use Case |
|-------|-------------|----------|
| **rustls** | Pure Rust TLS | Secure connections |
| **rustls-pemfile** | PEM file parsing | Certificates |
| **webpki-roots** | Root certificates | TLS trust |
| **ring** | Cryptographic primitives | TLS backend |

### Other
| Crate | Description | Use Case |
|-------|-------------|----------|
| **url** | URL parsing | Web addresses |
| **percent-encoding** | URL encoding | Query strings |
| **socket2** | Low-level sockets | Networking |
| **tokio** | Async runtime | Network I/O |
| **mio** | I/O event loop | Low-level async |
| **steamworks** | Steam SDK bindings | Steam integration |

---

## 🔐 Cryptography

| Crate | Description | Use Case |
|-------|-------------|----------|
| **sha2** | SHA-2 hashes | File integrity |
| **blake3** | BLAKE3 hash | Fast hashing |
| **crc** | CRC checksums | Data validation |
| **crc32fast** | Fast CRC32 | Checksums |
| **xxhash-rust** | xxHash | Fast non-crypto hash |
| **digest** | Hash traits | Generic hashing |
| **base64** | Base64 encoding | Data encoding |
| **hex** | Hex encoding | Debug output |

---

## ⚡ Async & Concurrency

### Async Runtimes
| Crate | Description | Use Case |
|-------|-------------|----------|
| **tokio** | Full async runtime | Network, I/O |
| **tokio-util** | Tokio utilities | Codecs, etc. |
| **async-executor** | Simple executor | Lightweight async |
| **async-task** | Task abstraction | Async building block |
| **futures** | Futures utilities | Async combinators |
| **futures-lite** | Lightweight futures | Minimal async |

### Channels & Sync
| Crate | Description | Use Case |
|-------|-------------|----------|
| **crossbeam** | Concurrent tools | Channels, queues |
| **crossbeam-channel** | Multi-producer channel | Thread communication |
| **crossbeam-deque** | Work-stealing deque | Task scheduling |
| **flume** | Fast MPMC channel | Message passing |
| **async-channel** | Async channels | Async communication |
| **parking_lot** | Fast mutexes | Synchronization |
| **spin** | Spinlocks | Low-latency locking |

### Thread Pools
| Crate | Description | Use Case |
|-------|-------------|----------|
| **rayon** | Data parallelism | Parallel iterators |
| **rayon-core** | Rayon thread pool | Work stealing |

---

## 📊 Profiling & Debugging

### Profiling
| Crate | Description | Use Case |
|-------|-------------|----------|
| **puffin** | Frame profiler | Game profiling |
| **puffin_http** | Puffin web viewer | Remote profiling |
| **tracing** | Instrumentation | Spans, events |
| **tracing-subscriber** | Tracing output | Logging backend |

### Crash Reporting
| Crate | Description | Use Case |
|-------|-------------|----------|
| **sentry** | Error tracking | Crash reports |
| **minidump** | Minidump parsing | Crash analysis |
| **minidump-writer** | Create minidumps | Crash capture |
| **crash-handler** | Crash handling | Error recovery |
| **backtrace** | Stack traces | Debugging |

### Symbol Resolution
| Crate | Description | Use Case |
|-------|-------------|----------|
| **addr2line** | DWARF symbols | Debug info |
| **gimli** | DWARF parsing | Debug sections |
| **debugid** | Debug identifiers | Symbol matching |
| **pdb-addr2line** | PDB symbols | Windows debugging |
| **wholesym** | Symbol resolution | Full symbolication |
| **framehop** | Frame unwinding | Stack walking |

---

## 🛠️ Utilities

### Collections
| Crate | Description | Use Case |
|-------|-------------|----------|
| **hashbrown** | Fast HashMap | Default in std |
| **indexmap** | Ordered HashMap | Insertion order |
| **slotmap** | Slot-based map | Entity storage |
| **smallvec** | Stack-allocated Vec | Small arrays |
| **arrayvec** | Fixed-capacity Vec | No allocation |
| **tinyvec** | Tiny vectors | Small data |
| **bitvec** | Bit vectors | Flags, sets |
| **rangemap** | Range maps | Interval data |

### Smart Pointers
| Crate | Description | Use Case |
|-------|-------------|----------|
| **arc-swap** | Atomic Arc swap | Lock-free updates |
| **triomphe** | Arc variants | Custom Arcs |
| **once_cell** | Lazy initialization | Singletons |
| **lazy_static** | Static initialization | Global data |

### Error Handling
| Crate | Description | Use Case |
|-------|-------------|----------|
| **anyhow** | Easy error handling | Applications |
| **thiserror** | Error derive | Libraries |
| **quick-error** | Error macros | Simple errors |

### Logging
| Crate | Description | Use Case |
|-------|-------------|----------|
| **log** | Logging facade | Debug output |
| **env_logger** | Environment logger | Development |
| **fern** | Logging configuration | Flexible logging |
| **pretty_env_logger** | Colored logging | Pretty output |
| **colored** | Terminal colors | CLI output |

---

## 📝 Serialization

| Crate | Description | Use Case |
|-------|-------------|----------|
| **serde** | Serialization framework | Data conversion |
| **serde_json** | JSON serialization | Web APIs |
| **serde_derive** | Serde macros | Auto-implement |
| **bincode** | Binary serialization | Fast, compact |
| **rkyv** | Zero-copy serialization | Fastest access |
| **speedy** | Fast serialization | Game data |
| **ron** | Rusty Object Notation | Config files |

---

## 🪟 Windowing & Input

| Crate | Description | Use Case |
|-------|-------------|----------|
| **winit** | Cross-platform windows | Window creation |
| **raw-window-handle** | Window handle trait | Graphics integration |
| **cursor-icon** | Cursor icons | Mouse cursors |
| **arboard** | Clipboard access | Copy/paste |
| **window_clipboard** | Clipboard for winit | Text clipboard |
| **rfd** | File dialogs | Open/save dialogs |
| **webbrowser** | Open URLs | External links |

---

## 💻 System & Platform

### Windows
| Crate | Description | Use Case |
|-------|-------------|----------|
| **windows** | Windows API bindings | Win32 |
| **windows-sys** | Raw Windows bindings | Low-level |
| **winapi** | Legacy Windows bindings | Older code |
| **winreg** | Windows registry | Settings |
| **known-folders** | Special folders | Documents, etc. |

### Cross-Platform
| Crate | Description | Use Case |
|-------|-------------|----------|
| **libc** | C library bindings | System calls |
| **dirs** | User directories | Config paths |
| **tempfile** | Temporary files | Scratch data |
| **walkdir** | Directory traversal | File scanning |
| **notify** | File watching | Hot reload |
| **hotwatch** | Simple file watching | Auto-reload |
| **filetime** | File timestamps | Modification time |
| **fs4** | File locking | Exclusive access |
| **memmap2** | Memory-mapped files | Large files |
| **hostname** | Get hostname | Network identity |
| **os_info** | OS detection | Platform checks |
| **raw-cpuid** | CPU feature detection | SIMD selection |
| **num_cpus** | CPU count | Thread pools |

---

## 📜 Parsing & Text

### Parsing
| Crate | Description | Use Case |
|-------|-------------|----------|
| **nom** | Parser combinators | Binary/text parsing |
| **winnow** | Fast parsing | Optimized parsers |
| **regex** | Regular expressions | Pattern matching |
| **pest** | PEG parser | Grammars |

### String Utilities
| Crate | Description | Use Case |
|-------|-------------|----------|
| **memchr** | Fast byte search | String scanning |
| **aho-corasick** | Multi-pattern search | Text search |
| **smol_str** | Small string | Inline storage |
| **ustr** | Unique strings | String interning |
| **heck** | Case conversion | snake_case, etc. |
| **itoa** | Integer to string | Fast formatting |
| **ryu** | Float to string | Fast formatting |

---

## 🔧 Proc Macros & Derive

| Crate | Description | Use Case |
|-------|-------------|----------|
| **syn** | Rust parser | Proc macros |
| **quote** | Rust code generation | Proc macros |
| **proc-macro2** | Proc macro utilities | Token streams |
| **derive_more** | Common derives | Boilerplate |
| **paste** | Identifier pasting | Macro helpers |
| **cfg-if** | Conditional compilation | Platform code |
| **bitflags** | Bit flag types | Option flags |
| **enum-map** | Enum to array | Fast lookup |
| **num_enum** | Enum from integers | Parsing |
| **strum** | Enum utilities | String conversion |

---

## 🎲 Random & Noise

| Crate | Description | Use Case |
|-------|-------------|----------|
| **rand** | Random numbers | General RNG |
| **rand_chacha** | ChaCha RNG | Cryptographic |
| **rand_xorshift** | XorShift RNG | Fast, simple |
| **fastrand** | Tiny fast RNG | Quick random |
| **bracket-noise** | Noise functions | Procedural generation |
| **bracket-random** | Game-oriented RNG | Dice, etc. |
| **perchance** | Simple RNG | Lightweight |

---

## 🎯 What You Might Want for Your Renderer

### Essential (You Have)
- ✅ `ash` - Vulkan bindings
- ✅ `winit` - Windowing
- ✅ `gpu-allocator` - Memory management
- ✅ `anyhow` - Error handling
- ✅ `log` / `env_logger` - Logging

### Recommended Additions
| Priority | Crate | Why |
|----------|-------|-----|
| ⭐⭐⭐ | `egui` + `egui-ash` | Debug UI panels |
| ⭐⭐⭐ | `glam` | Fast math (you might have) |
| ⭐⭐ | `puffin` | Frame profiling |
| ⭐⭐ | `image` | Texture loading |
| ⭐⭐ | `notify` | Shader hot-reload |
| ⭐ | `rspirv` | Shader reflection |
| ⭐ | `ddsfile` | Compressed textures |

---

## 📚 Learning Resources

- [crates.io](https://crates.io) - Search for any crate
- [lib.rs](https://lib.rs) - Better crate discovery
- [docs.rs](https://docs.rs) - Documentation for all crates
- [Are We Game Yet?](https://arewegameyet.rs/) - Game dev crate list

---

*Generated for my-renderer learning project*
