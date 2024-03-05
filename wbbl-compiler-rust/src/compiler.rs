use std::collections::{HashMap, HashSet};

use crate::{
    compiler_types::{CompilationOutput, CompilationStage},
    data_types::ComputationDomain,
    graph_types::BranchedMultiGraph,
};

pub fn compile_to_naga_ir(
    branched_multi_graph: &BranchedMultiGraph,
    computation_domains: &HashMap<u128, HashSet<ComputationDomain>>,
) -> CompilationOutput {
    let mut output: Vec<CompilationStage> = vec![];
    CompilationOutput(output)
}
