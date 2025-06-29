fn main() -> anyhow::Result<()> {
    cobu::parse_cli(None)
}

// use std::path::PathBuf;

// fn main() -> anyhow::Result<()> {
//     let manifest_dir = PathBuf::from("/home/robert/GitProjects/cobu/contests/problemset");
//     let dist_dir = manifest_dir.join("dist");
//     if !dist_dir.exists() {
//         std::fs::create_dir(&dist_dir)?;
//     }

//     cobu::cli(cobu::Args {
//         libs: vec![(
//             "crads".to_string(),
//             manifest_dir
//                 .join("..")
//                 .join("..")
//                 .join("crates")
//                 .join("crads")
//                 .join("src")
//                 .join("lib.rs"),
//         )],
//         manifest_path: Some(manifest_dir.join("Cargo.toml")),
//         out_dir: dist_dir,
//         ..Default::default()
//     })?;

//     Ok(())
// }
