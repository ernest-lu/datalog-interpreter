pub mod parse;
pub mod implem;

pub use implem::run_datalog;
pub use parse::{Program, Fact, Token};
