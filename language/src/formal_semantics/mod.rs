pub mod fol;
pub mod type_checker;
pub use fol::{Formula, GenQKind, Term, to_fol};
pub use type_checker::{Type, TypeMismatch, type_check};
