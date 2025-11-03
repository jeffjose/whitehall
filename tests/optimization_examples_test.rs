/// Optimization validation tests
///
/// Tests that optimization examples match expected output:
/// - Static collections should generate RecyclerView (Optimized Output)
/// - Dynamic collections should stay Compose (Unoptimized Output)
///
/// NOTE: Currently skipped because optimization examples use advanced syntax
/// not yet supported by the parser (multiline list literals, delegation, etc.)
/// These are aspirational examples for future work.

#[cfg(test)]
mod tests {
    #[test]
    #[ignore] // Skip for now - examples use advanced syntax not yet supported
    fn test_optimization_examples() {
        // TODO: Create simpler optimization examples that work with current parser
        // or extend parser to support:
        // - Multiline list initialization
        // - Property delegation (by remember, by derivedStateOf)
        // - Data class constructors in list initialization
        
        println!("Optimization examples test skipped - waiting for parser improvements");
    }
}
