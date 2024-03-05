use std::collections::HashSet;

use naga::Module;

use crate::data_types::ComputationDomain;

pub struct ComputeRasterizerShader {
    pub shader: Module,
}

pub struct VertexFragmentShader {
    pub vertex: Module,
    pub fragment: Module,
}

pub enum Shader {
    ComputeRasterizer(ComputeRasterizerShader),
    VertexFragment(VertexFragmentShader),
}

pub struct CompilationStage {
    pub id: u128,
    pub shaders: Shader,
    pub domain: HashSet<ComputationDomain>,
    pub dependencies: HashSet<u128>,
    pub dependants: HashSet<u128>,
}

pub struct CompilationOutput(pub Vec<CompilationStage>);
