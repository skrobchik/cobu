fn main() -> anyhow::Result<()> {
    cobu::cli(Some(&[
        "--bin".to_string(),
        "contest".to_string(),
        "--manifest-path".to_string(),
        "C:\\Users\\Robert\\Desktop\\icpc\\contest\\Cargo.toml".to_string(),
    ]))
}
