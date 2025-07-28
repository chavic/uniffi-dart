#[cfg(test)]
mod tests {
    #[test]
    fn test_metadata() {
        uniffi_dart::testing::run_test("metadata", "src/api.udl", None).unwrap();
    }
}
