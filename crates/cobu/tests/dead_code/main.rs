use std::{io::Write, path::PathBuf};
use cobu_macros::integration_test;

integration_test!(1);
integration_test!(2);
integration_test!(3);
integration_test!(4);
integration_test!(5);

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
