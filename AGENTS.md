# AGENTS.md

This file provides guidance to coding agents working with code in this repository.

## Project overview

Kea is a hobby Vulkan ray-tracing project consisting of a Vulkan wrapper (`kea_gpu`) and a path tracer (`kea_renderer`) that uses it. Shaders are written in Rust and compiled to SPIR-V via [rust-gpu](https://github.com/Rust-GPU/rust-gpu).

Requires a GPU with Vulkan ray tracing extension support (`VK_KHR_ray_tracing_pipeline`, `VK_KHR_acceleration_structure`).

## Common commands

```bash
# Run the path tracer
cargo run -p kea_renderer

# Release build (host only — shader profile is set in build.rs)
cargo run -p kea_renderer --release

# Compile-check the workspace
cargo check
```

There are no tests in this repo.

The host workspace builds on **stable Rust**. Shader compilation is driven by `kea_renderer/build.rs` via [`cargo-gpu-install`](https://github.com/Rust-GPU/rust-gpu/tree/main/crates/cargo-gpu-install), which installs and uses its own pinned nightly toolchain in `~/.cache/rust-gpu`. The first build is multi-minute (downloads + builds `rustc_codegen_spirv`); subsequent builds are cached.

Logging is controlled by `RUST_LOG` (env_logger). Default filter is `info`.

## Workspace layout

- `kea_gpu` — host-side Vulkan wrapper built on Ash. Owns instances, devices, command buffers, descriptor sets, swapchain/presentation, ray-tracing pipelines and acceleration structures, and shader-module loading. This is where almost all `unsafe` Vulkan calls live. No rust-gpu deps.
- `kea_gpu_shaderlib` — types shared between host and shader code (`Ray`, `Aabb`, `Slot`/`SlotType` definitions, `ShaderGroup`). `#![no_std]` under `target_arch = "spirv"`, `std` otherwise.
- `kea_renderer_shaders` — the ray-gen / miss / hit / intersection shaders, plus the `SlotId`, `SHADERS`, and `SLOTS` tables. **Excluded from the workspace** (see root `Cargo.toml`'s `[workspace] exclude`) so its SPIR-V build doesn't fight workspace feature unification.
- `kea_renderer` — host-side binary. `build.rs` compiles the shader crate to SPIR-V (one module per entry point) and writes a generated `shader_modules.rs` with `include_bytes!` references. The binary loads those bytes at startup.

## Architecture

### Host bootstrap (`kea_gpu`)

`Kea::new` is the entry point: given a `Window` and a list of `Feature`s it creates the `VulkanInstance`, picks a physical device that supports both graphics and the window surface, builds a `Device`, and constructs a `Presenter` (swapchain). `PresentationFeature` is always added; `RayTracingFeature` and `DebugFeature` are opt-in via the caller.

`Feature` (`kea_gpu/src/features.rs`) is the extension/layer plug-in mechanism — each feature contributes instance extensions, device extensions, layers, and instance/device config tweaks. Anything that needs a Vulkan extension or validation layer should be a `Feature`.

### Shaders and slot binding

Slot binding is the key abstraction tying host code to shaders.

- `kea_renderer_shaders` defines an enum `SlotId` (e.g. `Scene`, `OutputImage`, `Spheres`, `Vertices`, …) and a const `SLOTS` table mapping each `SlotId` to a `Slot { slot_type, stages }`.
- `kea_renderer_shaders::SHADERS` enumerates the `ShaderGroup`s (raygen / miss / triangle hit / procedural hit) and names entry points by string (e.g. `"path_tracer::entrypoints::generate_rays"`).
- `kea_renderer/build.rs` invokes `cargo-gpu-install` to compile `kea_renderer_shaders` to SPIR-V at host build time, producing one `.spv` per entry point. The build script writes `$OUT_DIR/shader_modules.rs` containing `pub static SHADER_MODULES: &[(&str, &[u8])]` with `include_bytes!` of each `.spv`. `path_tracer.rs` includes it via `include!(concat!(env!("OUT_DIR"), "/shader_modules.rs"))`.
- `kea_gpu::shaders::ShaderModule::load_modules` takes that slice and produces the `HashMap<String, Arc<ShaderModule>>` consumed by `ShaderGroups::build`.
- `SlotLayout` turns the `SLOTS` table into Vulkan descriptor set layout bindings; `SlotBindings` is the typed, runtime-side handle used to bind images / buffers / acceleration structures by `SlotId`.

If you add or change a slot or shader entry point, you must update *both* the const tables in `kea_renderer_shaders/src/lib.rs` and the host-side binding code in `kea_renderer/src/path_tracer.rs`. The string entry-point name must exactly match the function path inside the shader crate.

### Frame loop

`PathTracer::draw` (in `kea_renderer/src/path_tracer.rs`) reuses a long-lived command pool + buffer per in-flight frame slot (one slot per `kea_gpu::presentation::FRAMES_IN_FLIGHT`). Each frame: take the slot's command buffer, reset its pool, re-record (bind pipeline + descriptor set, push the iteration counter, barrier `light_image`, `trace_rays`, blit `storage_image` into the acquired swapchain image), submit, and return the buffer to its slot. Submission goes through `Presenter::draw`, which signals a per-image binary semaphore for the present-engine wait *and* increments a timeline semaphore that gates the next frame's `get_swapchain_image`.

The frame counter (`Presenter::frame_index`) doubles as the shader's `iteration` push-constant for sample accumulation, so they stay in sync.

### Scenes

Scenes live in `kea_renderer/src/scenes/`. A `Scene` builds the bottom- and top-level acceleration structures via `kea_gpu::ray_tracing::scenes` and exposes `bind_data` to populate the relevant slots (scene TLAS, spheres, meshes). `examples::cornell_box` is the default scene loaded by `PathTracer::new`.

## Conventions and gotchas

- Sync uses Vulkan synchronization2 end-to-end: `vk::AccessFlags2` / `vk::PipelineStageFlags2` for barriers and `vkQueueSubmit2` (`VkSubmitInfo2`) for queue submission. Don't introduce v1 `VkSubmitInfo` or `VkPipelineStageFlags`. Use `vk::PipelineStageFlags2::NONE` for "no-op" stages instead of the deprecated `TOP_OF_PIPE` / `BOTTOM_OF_PIPE`.
- The presenter pipelines `FRAMES_IN_FLIGHT` (currently 2) frames using a timeline semaphore. Anything that touches GPU-side state shared across frames (e.g. `light_image`, which `trace_rays` reads-then-writes as an accumulator) needs an explicit barrier in the per-frame command buffer — the previous fence wait no longer provides implicit serialisation.
- Resource constructors call `Device::name_object` to tag the underlying Vulkan handle via `VK_EXT_debug_utils`, so RenderDoc / Nsight / validation messages show the wrapper's name. Pass meaningful names when constructing `Buffer`, `Image`, `Fence`, `Semaphore::new_named`, `TimelineSemaphore::new_named`, etc.
- The RT pipeline is created with a disk-backed `VkPipelineCache` at `$XDG_CACHE_HOME/kea-gfx/pipeline-cache.bin`. Failures are logged and non-fatal.
- Validation toggles go through `VK_EXT_layer_settings` (not the deprecated `VK_EXT_validation_features`). Setting names live in `kea_gpu::debug::feature::DebugFeature::configure_instance`.
- The path tracer's `storage_image` is `R32G32B32A32_SFLOAT` (matching the shader's `rgba32f` declaration) and is converted to the swapchain's `B8G8R8A8_UNORM` via `cmd_blit_image` on present, **not** `cmd_copy_image`.
- Avoid `f64` and other double-precision values in shader code. Some target GPUs (e.g. Intel) lack `Float64` capability; using doubles will silently fail to find intersections. (See commit `7f0ffcb`.)
- **Shaders build in debug profile (`builder.release = false` in `build.rs`).** rust-gpu 0.10.0-alpha.1's release-mode optimizer eliminates the `#[spirv(miss)]` entry point. Track upstream and switch to release once fixed.
- The shader crate must stay **`exclude`-d from the workspace**. Including it triggers feature unification that activates the `std` feature on `spirv-std-types` for the SPIR-V target build (because `rustc_codegen_spirv` enables it on the host side).
- Storage-image entry-point bindings use `&Image!(...)` (not `&mut`); writes go through `&self` methods on `Image`. The `&mut` form was removed in rust-gpu 0.10.
- Resource lifetime is RAII via `Arc`-wrapped wrappers. `Drop` impls call the corresponding `vk::destroy_*` — don't add explicit destruction calls.
- Most files do not have an existing test harness; prefer `cargo run -p kea_renderer` to validate changes end-to-end.

## Known issues

- `VK_KHR_ray_tracing_position_fetch` was prototyped (commit `50c2a8c`) and reverted (`5f245c5`) because the resulting rendering had the ceiling light invisible. SPIR-V codegen, capability/extension declaration, feature struct, and the `ALLOW_DATA_ACCESS` BLAS build flag were all wired up correctly. Most likely cause is that the implementation returns wrong values for the builtin on this driver, or that the `[Vec3; 3]` parameter ABI in rust-gpu 0.10 doesn't match the SPIR-V `Input` array layout. Worth a deeper look with RenderDoc to capture the actual `gl_HitTriangleVertexPositionsEXT` values.
