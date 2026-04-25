use cargo_gpu_install::install::Install;
use cargo_gpu_install::spirv_builder::{Capability, SpirvMetadata};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shader_crate = manifest_dir.parent().unwrap().join("kea_renderer_shaders");

    let mut install = Install::from_shader_crate(shader_crate.clone());
    install.auto_install_rust_toolchain = true;
    install.build_script = true;
    let backend = install.run()?;

    let mut builder = backend.to_spirv_builder(&shader_crate, "spirv-unknown-vulkan1.2");
    builder.build_script.defaults = true;
    // Release-mode shader optimization in rustc_codegen_spirv 0.10.0-alpha.1
    // eliminates the miss-shader entry point. Build shaders unoptimized until
    // upstream is fixed.
    builder.release = false;
    builder.multimodule = true;
    builder.capabilities = vec![Capability::RayTracingKHR, Capability::Int64];
    builder.extensions = vec![
        "SPV_KHR_ray_tracing".into(),
        "SPV_KHR_non_semantic_info".into(),
    ];
    builder.spirv_metadata = SpirvMetadata::Full;

    let result = builder.build()?;
    let modules = result.module.unwrap_multi();

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let mut generated = String::new();
    generated.push_str("pub static SHADER_MODULES: &[(&str, &[u8])] = &[\n");
    for (entry_point, spv_path) in modules {
        generated.push_str(&format!(
            "    ({:?}, include_bytes!({:?})),\n",
            entry_point,
            spv_path.to_string_lossy(),
        ));
    }
    generated.push_str("];\n");
    std::fs::write(out_dir.join("shader_modules.rs"), generated)?;

    Ok(())
}
