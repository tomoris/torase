pub mod first_order_logic;
pub mod type_checker;
pub use first_order_logic::{Formula, GenQKind, Term, to_fol};
pub use type_checker::{Type, TypeMismatch, type_check};
