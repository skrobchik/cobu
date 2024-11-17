use std::{
    collections::BTreeMap, error::Error, io::Write, ops::Range, path::PathBuf, time::Instant,
};

use anyhow::Context;

use cargo_metadata::diagnostic::{Diagnostic, DiagnosticSpan};
use clap::Parser;
use quote::quote;
use syn::{spanned::Spanned, Ident, Item};

/// COmpetitive BUndler for Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Binary to bundle
    #[arg(long)]
    bin: String,

    /// Package
    #[arg(short, long)]
    package: Option<String>,

    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    manifest_path: Option<PathBuf>,

    #[arg(long, value_parser = parse_key_val::<String, PathBuf>, value_delimiter = ',')]
    libs: Vec<(String, PathBuf)>,

    /// Output path
    #[arg(short, long)]
    output: Option<PathBuf>,
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

fn rustc_diagnostics(src: &str) -> anyhow::Result<Vec<Diagnostic>> {
    let mut command = std::process::Command::new("rustc")
        .args([
            "--error-format=json",
            "-C",
            "debuginfo=none",
            // "-C", "linkargs=/DEBUG:NONE", // TODO: Figure out how to not generate PDB with MSVC
            "-o",
            "-",
            "-",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = command.stdin.take().context("Failed to open rustc stdin")?;
    stdin.write_all(src.as_bytes())?;
    stdin.flush()?;
    drop(stdin);

    let output = command.wait_with_output()?;
    let stderr_string = String::from_utf8(output.stderr)?;

    let diagnostics: Vec<Diagnostic> = stderr_string
        .lines()
        .filter(|s: &&str| !s.is_empty())
        .map(serde_json::from_str::<serde_json::Value>)
        .filter_map(Result::ok)
        .filter_map(|mut v| {
            let o = v.as_object_mut()?;
            let v = o.get("$message_type")?;
            if v != "diagnostic" {
                return None;
            }
            o.remove("$message_type");
            Some(serde_json::to_string(o).unwrap())
        })
        .map(|s| serde_json::from_str(&s).unwrap())
        .collect();

    Ok(diagnostics)
}

pub fn remove_dead_code(src: String) -> anyhow::Result<String> {
    let dead_code_diagnostics: Vec<Diagnostic> = rustc_diagnostics(&src)?
        .into_iter()
        .filter(|d| d.code.as_ref().map_or(false, |c| c.code == "dead_code"))
        .collect();

    let dead_code_diagnostic_spans: Vec<DiagnosticSpan> = dead_code_diagnostics
        .into_iter()
        .map(|d| d.spans.into_iter().next().unwrap())
        .collect();

    let is_dead_code = |ident: &Ident| -> bool {
        dead_code_diagnostic_spans.iter().any(|diagnostic_span| {
            let span = ident.span();
            let span_range: Range<usize> = span.byte_range();
            let span_range = (span_range.start, span_range.end);
            let diagnostic_span_range = (
                usize::try_from(diagnostic_span.byte_start).unwrap(),
                usize::try_from(diagnostic_span.byte_end).unwrap(),
            );
            if span_range == diagnostic_span_range {
                let diagnostic_text = &diagnostic_span.text[0];
                assert_eq!(
                    diagnostic_text
                        .text
                        .get(diagnostic_text.highlight_start - 1..diagnostic_text.highlight_end - 1)
                        .unwrap(),
                    &quote!(#ident).to_string()
                );
                println!("Found dead code! {}", &quote!(#ident).to_string());
                true
            } else {
                false
            }
        })
    };

    let mut dead_bytes = vec![false; src.len()];

    let ast = syn::parse_file(&src)?;
    for item in ast.items {
        match item {
            Item::Fn(fun) => {
                if is_dead_code(&fun.sig.ident) {
                    for i in fun.span().byte_range() {
                        dead_bytes[i] = true;
                    }
                }
            }
            Item::Struct(stru) => {
                if is_dead_code(&stru.ident) {
                    for i in stru.span().byte_range() {
                        dead_bytes[i] = true;
                    }
                }
            }
            _ => (),
        }
    }

    let src: Vec<u8> = src
        .bytes()
        .enumerate()
        .filter(|(i, _b)| !dead_bytes[*i])
        .map(|(_i, b)| b)
        .collect();
    let src: String = String::from_utf8(src)?;

    Ok(src)
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

fn bin_src(args: &Args) -> anyhow::Result<String> {
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
        &metadata.packages[0]
    };
    let bin_src_path = PathBuf::from(
        &package
            .targets
            .iter()
            .find(|t| t.is_bin() && t.name == args.bin)
            .context(format!("Binary {} not found", args.bin))?
            .src_path,
    );
    let src = std::fs::read_to_string(bin_src_path)?;
    Ok(src)
}

fn expand_libs(libs: &BTreeMap<String, PathBuf>, src: String) -> Result<String, std::io::Error> {
    libs.iter()
        .map(|(name, path)| {
            std::fs::read_to_string(path).map(|contents| format!("mod {name} {{\n{}\n}}", contents))
        })
        .chain(std::iter::once(Ok(src)))
        .collect()
}

pub fn cli(concat_args: Option<&[String]>) -> anyhow::Result<()> {
    let start_instant_cli = std::time::Instant::now();

    let mut args: Vec<String> = std::env::args().collect();
    if let Some(concat_args) = concat_args {
        args.extend_from_slice(concat_args);
    };
    let args = Args::parse_from(args);

    let libs: BTreeMap<String, PathBuf> = args.libs.clone().into_iter().collect();
    assert_eq!(libs.len(), args.libs.len(), "no duplicate lib names");

    let src = bin_src(&args)?;
    let src = expand_libs(&libs, src)?;

    let start_instant_deadcode = Instant::now();
    let src = remove_dead_code(src)?;
    let duration_deadcode = start_instant_deadcode.elapsed();

    let start_instant_rustfmt = Instant::now();
    let src = rustfmt(&src)?;
    let duration_rustfmt = start_instant_rustfmt.elapsed();

    println!("{src}");

    let duration_cli = start_instant_cli.elapsed();
    println!("Done! Took: {} ms", duration_cli.as_millis());
    println!(
        "Dead code removal took: {} ms",
        duration_deadcode.as_millis()
    );
    println!("Rustfmt took: {} ms", duration_rustfmt.as_millis());
    Ok(())
}
