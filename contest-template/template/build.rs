use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let dist_dir = manifest_dir.join("dist");
    if !dist_dir.exists() {
        std::fs::create_dir(&dist_dir)?;
    }

    let _ = cobu::cli(cobu::Args {
        libs: vec![(
            "crads".to_string(),
            manifest_dir
                .join("..")
                .join("..")
                .join("crads")
                .join("src")
                .join("lib.rs"),
        )],
        manifest_path: Some(manifest_dir.join("Cargo.toml")),
        out_dir: dist_dir,
        ..Default::default()
    })?;

    Ok(())
}
