use std::{collections::BTreeMap, error::Error, io::Write, path::PathBuf};

use anyhow::Context;

use cargo_metadata::Target;
use clap::Parser;

mod dead_code;
pub use dead_code::remove_dead_code;

/// COmpetitive BUndler for Rust
#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Binary to bundle
    #[arg(long)]
    pub bin: Option<String>,

    /// Package
    #[arg(short, long)]
    pub package: Option<String>,

    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    pub manifest_path: Option<PathBuf>,

    #[arg(long, value_parser = parse_key_val::<String, PathBuf>, value_delimiter = ',')]
    pub libs: Vec<(String, PathBuf)>,

    /// Output directory path
    #[arg(short, long)]
    pub out_dir: PathBuf,
}

/// Copied from https://github.com/clap-rs/clap/blob/2920fb082c987acb72ed1d1f47991c4d157e380d/examples/typed-derive.rs#L48
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

pub fn rustfmt(src: &str) -> anyhow::Result<String> {
    let mut command = std::process::Command::new("rustfmt")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = command
        .stdin
        .take()
        .context("Failed to open rustfmt stdin")?;
    stdin.write_all(src.as_bytes())?;
    stdin.flush()?;
    drop(stdin);

    let output = command.wait_with_output()?;
    let stdout_string = String::from_utf8(output.stdout)?;

    Ok(stdout_string)
}

fn expand_libs(libs: &BTreeMap<String, PathBuf>, src: String) -> Result<String, std::io::Error> {
    libs.iter()
        .map(|(name, path)| {
            std::fs::read_to_string(path).map(|contents| format!("mod {name} {{\n{}\n}}", contents))
        })
        .chain(std::iter::once(Ok(src)))
        .collect()
}

pub fn cli(args: Args) -> anyhow::Result<()> {
    let libs: BTreeMap<String, PathBuf> = args.libs.clone().into_iter().collect();
    assert_eq!(libs.len(), args.libs.len(), "no duplicate lib names");

    let cwd_manifest_path = std::env::current_dir()?.join("Cargo.toml");
    let manifest_path = args.manifest_path.as_ref().unwrap_or(&cwd_manifest_path);
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()?;

    let package = if let Some(package) = &args.package {
        metadata
            .packages
            .iter()
            .find(|p| p.name == *package)
            .context(format!("Package {package} not found"))?
    } else {
        metadata.root_package().context("Root package not found")?
    };

    let bins: Vec<Target> = package
        .targets
        .iter()
        .filter(|t| t.is_bin())
        .cloned()
        .collect();

    let bins: Vec<Target> = if let Some(bin) = args.bin {
        vec![bins
            .into_iter()
            .find(|t| t.name == bin)
            .context(format!("Binary {} not found", bin))?]
    } else {
        bins
    };

    if !args.out_dir.exists() {
        std::fs::create_dir(&args.out_dir)?;
    }
    assert!(args.out_dir.is_dir());
    for bin in bins {
        let src = std::fs::read_to_string(bin.src_path)?;
        let src = expand_libs(&libs, src)?;
        let src = remove_dead_code(src)?;
        let src = rustfmt(&src)?;
        let mut f = std::fs::File::create(args.out_dir.join(bin.name).with_extension("rs"))?;
        f.write_all(src.as_bytes())?;
    }

    Ok(())
}

pub fn parse_cli(concat_args: Option<&[String]>) -> anyhow::Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    if let Some(concat_args) = concat_args {
        args.extend_from_slice(concat_args);
    };
    let args = Args::parse_from(args);

    cli(args)
}
