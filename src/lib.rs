pub extern crate inventory;

pub use cohort_codegen::*;

pub fn query<Q>(_q: Q) -> Q {
    // `system` macro built-in
    unreachable!()
}
