#[cfg(test)]
mod tests {
    #[test]
    fn test_docstring() {
        uniffi_dart::testing::run_test("docstring", "src/api.udl", None).unwrap();
    }
}
