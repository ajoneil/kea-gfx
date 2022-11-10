# Kea

Kea is a simplified Vulkan wrapper, and a ray tracing renderer. These parts are
being developed in tandem.

This is a hobby project in early development.

## kea-gpu

Wraps Vulkan with an easier to use interface. This currently only exposes the
features that kea-renderer needs, but I plan for it to be generic enough to be
useful in other projects.

Uses [Ash](https://github.com/ash-rs/ash) for Vulkan bindings.

Uses [rust-gpu](https://github.com/EmbarkStudios/rust-gpu) to compile Rust code
to SPIR-V shaders.

## kea-renderer

A hardware accelerated path tracer. Requires a graphics card that supports the
Vulkan ray tracing extensions.

This is a naive implementation and very slow.

![Example image](example.avif)
