pub(crate) mod compiler;
pub(crate) mod compiler_constants;
pub(crate) mod compute_rasterizer;
pub(crate) mod constraint_solver;
pub(crate) mod constraint_solver_constraints;
pub(crate) mod data_types;
pub(crate) mod graph_functions;
pub(crate) mod graph_types;
pub(crate) mod intermediate_compiler_types;
pub(crate) mod utils;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
