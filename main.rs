#[test]
fn run_tests() {
    memory_tests::tests::test_non_banked_memory();
    memory_tests::tests::test_banked_memory();
    memory_tests::tests::test_io_registers();
    memory_tests::tests::test_read_write_word();
}