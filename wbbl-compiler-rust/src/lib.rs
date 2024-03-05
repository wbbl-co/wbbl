pub mod compiler;
pub mod compiler_types;
pub mod constraint_solver;
pub mod constraint_solver_constraints;
pub mod data_types;
pub mod graph_functions;
pub mod graph_types;
mod utils;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
