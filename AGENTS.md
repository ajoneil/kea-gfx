# AGENTS.md

This file provides guidance to coding agents working with code in this repository.

## Project overview

Kea is a hobby Vulkan ray-tracing project consisting of a Vulkan wrapper (`kea_gpu`) and a path tracer (`kea_renderer`) that uses it. Shaders are written in Rust and compiled to SPIR-V via [rust-gpu](https://github.com/EmbarkStudios/rust-gpu).

Requires a GPU with Vulkan ray tracing extension support (`VK_KHR_ray_tracing_pipeline`, `VK_KHR_acceleration_structure`).

## Common commands

```bash
# Run the path tracer (debug build is the usual workflow)
cargo run -p kea_renderer

# Release build
cargo run -p kea_renderer --release

# Compile-check the whole workspace
cargo check

# Build a single crate
cargo build -p kea_gpu
```

There are no tests in this repo.

The Rust toolchain is pinned in `rust-toolchain.toml` to `nightly-2023-01-21` because rust-gpu requires that exact nightly. Do not change it without also updating rust-gpu, and the pinned channel must match [rust-gpu's rust-toolchain.toml](https://raw.githubusercontent.com/EmbarkStudios/rust-gpu/main/rust-toolchain.toml).

Logging is controlled by `RUST_LOG` (env_logger). Default filter is `info`.

## Workspace layout

Four crates form two layers — a host-side Vulkan wrapper and shaders that target SPIR-V. The `*_shaderlib` / `*_shaders` crates are `no_std` when compiled for `target_arch = "spirv"` and ordinary `std` libraries otherwise; they are compiled twice (once by host-side `cargo`, once by `spirv-builder`).

- `kea_gpu` — host-side Vulkan wrapper built on Ash. Owns instances, devices, command buffers, descriptor sets, swapchain/presentation, ray-tracing pipelines and acceleration structures, and shader compilation. This is where almost all `unsafe` Vulkan calls live.
- `kea_gpu_shaderlib` — types shared between host and shader code (`Ray`, `Aabb`, `Slot`/`SlotType` definitions, `ShaderGroup`). `dylib + lib`.
- `kea_renderer_shaders` — the actual ray-gen / miss / hit / intersection shaders, plus the `SlotId` and `SHADERS`/`SLOTS` tables that describe the pipeline. `dylib + lib`. Built by rust-gpu at runtime when the host launches.
- `kea_renderer` — host-side binary. Constructs `Kea`, builds a `RayTracingPipeline`, loads a scene, and per-frame records a command buffer that traces rays into a storage image and copies it into the swapchain.

## Architecture

### Host bootstrap (`kea_gpu`)

`Kea::new` is the entry point: given a `Window` and a list of `Feature`s it creates the `VulkanInstance`, picks a physical device that supports both graphics and the window surface, builds a `Device`, and constructs a `Presenter` (swapchain). `PresentationFeature` is always added; `RayTracingFeature` and `DebugFeature` are opt-in via the caller.

`Feature` (`kea_gpu/src/features.rs`) is the extension/layer plug-in mechanism — each feature contributes instance extensions, device extensions, layers, and instance/device config tweaks. Anything that needs a Vulkan extension or validation layer should be a `Feature`.

### Shaders and slot binding

Slot binding is the key abstraction tying host code to shaders.

- `kea_renderer_shaders` defines an enum `SlotId` (e.g. `Scene`, `OutputImage`, `Spheres`, `Vertices`, …) and a const `SLOTS` table mapping each `SlotId` to a `Slot { slot_type, stages }`.
- `kea_renderer_shaders::SHADERS` enumerates the `ShaderGroup`s (raygen / miss / triangle hit / procedural hit) and names entry points by string (e.g. `"path_tracer::entrypoints::generate_rays"`).
- On startup, `kea_gpu::shaders::ShaderModule::new_multimodule` invokes `spirv-builder` against the `kea_renderer_shaders` crate path, producing one SPIR-V module per entry point. Compilation runs every time the renderer starts.
- `SlotLayout` turns the `SLOTS` table into Vulkan descriptor set layout bindings; `SlotBindings` is the typed, runtime-side handle used to bind images / buffers / acceleration structures by `SlotId`.

If you add or change a slot or shader entry point, you must update *both* the const tables in `kea_renderer_shaders/src/lib.rs` and the host-side binding code in `kea_renderer/src/path_tracer.rs`. The string entry-point name must exactly match the function path inside the shader crate.

### Frame loop

`PathTracer::draw` (in `kea_renderer/src/path_tracer.rs`) records a fresh command buffer each frame. It binds the ray-tracing pipeline and descriptor set, pushes per-frame `PushConstants` (currently just an `iteration` counter for sample accumulation), calls `trace_rays`, then transitions and copies the storage image into the acquired swapchain image. The command-buffer caching path is currently commented out — recording every frame is intentional during development.

### Scenes

Scenes live in `kea_renderer/src/scenes/`. A `Scene` builds the bottom- and top-level acceleration structures via `kea_gpu::ray_tracing::scenes` and exposes `bind_data` to populate the relevant slots (spheres, meshes, vertices, indices). `examples::cornell_box` is the default scene loaded by `PathTracer::new`.

## Conventions and gotchas

- Sync uses Vulkan synchronization2 (`VK_KHR_synchronization2`). Use `vk::AccessFlags2` / `vk::PipelineStageFlags2` for new barrier code, not the v1 versions. (See commit `f013bad`.)
- Avoid `f64` and other double-precision values in shader code. Some target GPUs (e.g. Intel) lack `Float64` capability; using doubles will silently fail to find intersections. (See commit `7f0ffcb`.)
- Resource lifetime is RAII via `Arc`-wrapped wrappers. `Drop` impls call the corresponding `vk::destroy_*` — don't add explicit destruction calls.
- Most files do not have an existing test harness; prefer `cargo run -p kea_renderer` to validate changes end-to-end.
