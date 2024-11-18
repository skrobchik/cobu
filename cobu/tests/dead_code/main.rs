const NUM_TESTS: u32 = 3;
seq!(N in 1..=3 {
    #[allow(dead_code)]
    mod input_~N;
    mod golden_~N;
});
mod output_2;

use seq_macro::seq;
use std::{io::Write, path::PathBuf};

#[test]
fn integration_tests() {
    for i in 1..=NUM_TESTS {
        integration_test(i);
    }
}

fn integration_test(test_index: u32) {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let dead_code_test_dir = manifest_dir.join("tests").join("dead_code");
    let input_path = dead_code_test_dir.join(format!("input_{test_index}.rs"));
    let golden_path = dead_code_test_dir.join(format!("golden_{test_index}.rs"));
    let input_contents = std::fs::read_to_string(input_path).unwrap();
    let golden_contents = std::fs::read_to_string(golden_path).unwrap();
    let output_contents = cobu::rustfmt(&cobu::remove_dead_code(input_contents).unwrap()).unwrap();
    let output_path = dead_code_test_dir.join(format!("output_{test_index}.rs"));
    std::fs::File::create(output_path)
        .unwrap()
        .write_all(output_contents.as_bytes())
        .unwrap();
    assert_eq!(
        output_contents, golden_contents,
        "Dead code test #{} failed",
        test_index
    );
}
