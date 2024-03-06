use std::collections::HashSet;

use naga::Module;
use naga::StorageFormat;

use crate::data_types::ComputationDomain;
use crate::data_types::ComputeOutputType;
use crate::data_types::Dimensionality;

pub struct ComputeRasterizerShader {
    pub shader: Module,
    pub size_multiplier: BaseSizeMultiplier,
    pub generate_mip_maps: bool,
}

pub struct ComputeShader {
    pub shader: Module,
    pub dim: Dimensionality,
    pub output_format: StorageFormat,
    pub output_type: ComputeOutputType,
    pub output_size_multiplier: BaseSizeMultiplier,
    pub generate_mip_maps: bool,
}

pub struct VertexFragmentShader {
    pub vertex: Module,
    pub fragment: Module,
}

pub enum Shader {
    ComputeRasterizer(ComputeRasterizerShader),
    ComputeShader(ComputeShader),
    VertexFragment(VertexFragmentShader),
}

pub struct Stage {
    pub id: u128,
    pub shader: Shader,
    pub domain: HashSet<ComputationDomain>,
    pub dependencies: Vec<u128>,
    pub dependants: HashSet<u128>,
}

pub struct BaseSizeMultiplier(pub f32);

pub struct Output(pub Vec<Stage>);
