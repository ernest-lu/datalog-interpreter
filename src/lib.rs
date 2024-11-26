pub mod bril_rs;
pub mod implem;
pub mod optimize_bril;
pub mod parse;
pub mod parse_bril;

pub use implem::run_datalog;
pub use parse::{Fact, Program, Token};
pub use parse_bril::parse_bril;
