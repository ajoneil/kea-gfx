#[derive(Clone)]
pub struct Shader(pub &'static str);

#[derive(Clone)]
pub enum ShaderGroup {
    RayGeneration(Shader),
    Miss(Shader),
    ProceduralHit { intersection: Shader, hit: Shader },
}
