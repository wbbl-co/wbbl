use std::collections::{HashMap, HashSet};

use crate::{
    data_types::ComputationDomain,
    data_types::ComputationDomain::ModelDependant,
    graph_types::BranchedMultiGraph,
    intermediate_compiler_types::{IntermediateOutput, Stage},
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
        })
        .map(|subgraph| *subgraph)
        .collect();
    if !model_dependent_subgraphs.is_empty() {
        // TODO: Add compute rasterizer stage and patch up dependencies
    }

    IntermediateOutput(output)
}
