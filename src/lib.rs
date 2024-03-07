pub mod compiler;
pub mod compiler_constants;
pub mod compute_rasterizer;
pub mod constraint_solver;
pub mod constraint_solver_constraints;
pub mod data_types;
pub mod graph_functions;
pub mod graph_types;
pub mod intermediate_compiler_types;
pub mod utils;
pub mod wbbl_physics;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
