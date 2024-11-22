pub mod implem;
pub mod parse;
pub mod parse_bril;

pub use implem::run_datalog;
pub use parse::{Fact, Program, Token};
pub use parse_bril::parse_bril;
