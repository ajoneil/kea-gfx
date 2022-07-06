#[derive(Clone)]
pub struct Shader(pub &'static str);

#[derive(Clone)]
pub enum ShaderGroup {
    RayGeneration(Shader),
    Miss(Shader),
    TriangleHit(Shader),
    ProceduralHit { intersection: Shader, hit: Shader },
}
