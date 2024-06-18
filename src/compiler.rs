use std::collections::{HashMap, HashSet};

use crate::{
    compute_rasterizer::generate_compute_rasterizer,
    data_types::ComputationDomain::{self, ModelDependant},
    graph_types::BranchedMultiGraph,
    intermediate_compiler_types::Shader::*,
    intermediate_compiler_types::{BaseSizeMultiplier, IntermediateOutput, Stage},
};

pub fn compile_to_naga_ir(
    branched_multi_graph: &BranchedMultiGraph,
    computation_domains: &HashMap<u128, HashSet<ComputationDomain>>,
) -> IntermediateOutput {
    let mut output: Vec<Stage> = vec![];
    let empty_domain: HashSet<ComputationDomain> = HashSet::new();
    let model_dependant: ComputationDomain = ModelDependant;
    let model_dependent_subgraphs: HashSet<u128> = branched_multi_graph
        .subgraphs
        .keys()
        .filter(|subgraph| {
            let domains = computation_domains.get(*subgraph).unwrap_or(&empty_domain);
            domains.contains(&model_dependant)
        }).copied()
        .collect();
    if !model_dependent_subgraphs.is_empty() {
        let compute_rasterizer = generate_compute_rasterizer(BaseSizeMultiplier(2.0), true);
        let mut rasterizer_stage = Stage {
            id: 0,
            dependencies: vec![],
            dependants: model_dependent_subgraphs.clone(),
            shader: ComputeRasterizer(compute_rasterizer),
            domain: HashSet::new(),
        };
        rasterizer_stage.domain.insert(model_dependant);
        // TODO: Patch up dependencies
        output.push(rasterizer_stage);
    }
    IntermediateOutput(output)
}
