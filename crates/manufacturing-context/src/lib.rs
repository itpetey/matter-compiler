//! Manufacturing environment abstractions for the Matter compiler.
//! This crate is intentionally limited to manufacturing context concerns.

pub mod schema {
    /// Root manufacturing-environment type for future schema work.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct ManufacturingContext;
}

pub use schema::ManufacturingContext;

#[cfg(test)]
mod tests {
    use super::ManufacturingContext;

    #[test]
    fn scaffold_exports_root_context_type() {
        let _ = ManufacturingContext::default();
    }
}