use std::{collections::BTreeSet, io::Write, ops::Range};

use anyhow::Context;

use cargo_metadata::diagnostic::{Diagnostic, DiagnosticSpan};
use proc_macro2::Span;
use quote::quote;
use syn::{spanned::Spanned, token, visit::Visit, Ident};

fn rustc_diagnostics(src: &str) -> anyhow::Result<Vec<Diagnostic>> {
    let mut command = std::process::Command::new("rustc")
        .args([
            "--edition", "2024",
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

struct DeadIdentifierVisitor {
    dead_code_diagnostic_spans: Vec<DiagnosticSpan>,
    output_dead_struct_identifiers: Vec<Ident>,
    output_dead_fn_identifiers: Vec<Ident>,
    output_dead_trait_identifiers: Vec<Ident>,
    output_dead_use_identifiers: Vec<Ident>,
}

impl DeadIdentifierVisitor {
    fn new(dead_code_diagnostic_spans: Vec<DiagnosticSpan>) -> Self {
        Self {
            dead_code_diagnostic_spans,
            output_dead_struct_identifiers: Vec::new(),
            output_dead_fn_identifiers: Vec::new(),
            output_dead_trait_identifiers: Vec::new(),
            output_dead_use_identifiers: Vec::new(),
        }
    }

    fn is_dead_code(&self, ident: Option<&Ident>, span: &Span) -> bool {
        self.dead_code_diagnostic_spans
            .iter()
            .any(|diagnostic_span| {
                if let Some(ident) = ident {
                    assert_eq!(ident.span().byte_range(), span.byte_range());
                }
                let span_range: Range<usize> = span.byte_range();
                let span_range = (span_range.start, span_range.end);
                let diagnostic_span_range = (
                    usize::try_from(diagnostic_span.byte_start).unwrap(),
                    usize::try_from(diagnostic_span.byte_end).unwrap(),
                );
                if span_range == diagnostic_span_range {
                    if let Some(ident) = ident {
                        let diagnostic_text = &diagnostic_span.text[0];
                        assert_eq!(
                            diagnostic_text
                                .text
                                .get(
                                    diagnostic_text.highlight_start - 1
                                        ..diagnostic_text.highlight_end - 1
                                )
                                .unwrap(),
                            &quote!(#ident).to_string()
                        );
                    }
                    true
                } else {
                    false
                }
            })
    }
}

impl<'ast> Visit<'ast> for DeadIdentifierVisitor {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        if self.is_dead_code(Some(&i.ident), &i.ident.span()) {
            self.output_dead_struct_identifiers.push(i.ident.clone());
        }
    }

    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        if self.is_dead_code(Some(&i.sig.ident), &i.sig.ident.span()) {
            self.output_dead_fn_identifiers.push(i.sig.ident.clone());
        }
    }

    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {
        if self.is_dead_code(Some(&i.ident), &i.ident.span()) {
            self.output_dead_trait_identifiers.push(i.ident.clone());
        }
    }

    fn visit_use_path(&mut self, i: &'ast syn::UsePath) {
        if self.is_dead_code(None, &i.span()) {
            self.output_dead_use_identifiers.push(i.ident.clone());
        }
    }
}

struct DeadCodeVisitor {
    dead_struct_identifiers: Vec<Ident>,
    dead_fn_identifiers: Vec<Ident>,
    dead_trait_identifiers: Vec<Ident>,
    dead_use_identifiers: Vec<Ident>,
    output_dead_spans: Vec<Span>,
}

impl DeadCodeVisitor {
    fn new(
        dead_struct_identifiers: Vec<Ident>,
        dead_fn_identifiers: Vec<Ident>,
        dead_trait_identifiers: Vec<Ident>,
        dead_use_identifiers: Vec<Ident>,
    ) -> Self {
        Self {
            dead_struct_identifiers,
            dead_fn_identifiers,
            dead_trait_identifiers,
            dead_use_identifiers,
            output_dead_spans: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for DeadCodeVisitor {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        if self.dead_struct_identifiers.contains(&i.ident) {
            self.output_dead_spans.push(i.span());
        }
    }

    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        if self.dead_fn_identifiers.contains(&i.sig.ident) {
            self.output_dead_spans.push(i.span());
        }
    }

    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {
        if self.dead_trait_identifiers.contains(&i.ident) {
            self.output_dead_spans.push(i.span());
        }
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        if let syn::Type::Path(type_path) = i.self_ty.as_ref() {
            if type_path.path.segments.len() != 1 {
                unimplemented!()
            }
            if self
                .dead_struct_identifiers
                .contains(&type_path.path.segments.last().unwrap().ident)
            {
                // TODO: Assuming that struct is in same module or that there's no name collisions
                self.output_dead_spans.push(i.span());
            }
        }
        if let Some(trait_) = i.trait_.as_ref() {
            let type_path = &trait_.1;
            if type_path.segments.len() != 1 {
                unimplemented!()
            }
            if self
                .dead_trait_identifiers
                .contains(&type_path.segments.last().unwrap().ident)
            {
                self.output_dead_spans.push(i.span());
            }
        }
    }

    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        match &i.tree {
            syn::UseTree::Path(use_path) => if self.dead_use_identifiers.contains(&use_path.ident) {
                self.output_dead_spans.push(i.span());
            } ,
            syn::UseTree::Name(_use_name) => todo!(),
            syn::UseTree::Rename(_use_rename) => todo!(),
            syn::UseTree::Glob(_use_glob) => todo!(),
            syn::UseTree::Group(_use_group) => todo!(),
        }
    }
}

fn is_test_module(item_mod: &syn::ItemMod) -> bool {
    item_mod
        .attrs
        .iter()
        .find(|attribute| {
            matches!(attribute.style, syn::AttrStyle::Outer)
                && match &attribute.meta {
                    syn::Meta::List(meta_list) => {
                        let mut tokens = meta_list.tokens.clone().into_iter();
                        match tokens.next() {
                            Some(proc_macro2::TokenTree::Ident(i)) => {
                                &i.to_string() == "test" && tokens.next().is_none()
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                }
                && attribute.path().is_ident("cfg")
        })
        .is_some()
}

#[derive(Default)]
struct TestModuleVisitor {
    output_test_module_spans: Vec<Span>,
}

impl<'ast> Visit<'ast> for TestModuleVisitor {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        if is_test_module(i) {
            self.output_test_module_spans.push(i.span());
        }
        syn::visit::visit_item_mod(self, i);
    }
}

pub fn remove_tests(src: String) -> anyhow::Result<String> {
    let ast = syn::parse_file(&src)?;
    let mut test_module_visitor = TestModuleVisitor::default();
    test_module_visitor.visit_file(&ast);
    let src= remove_spans(src, test_module_visitor.output_test_module_spans)?;
    Ok(src)
}

fn remove_spans(src: String, spans: Vec<Span>) -> anyhow::Result<String> {
    let mut dead_bytes = vec![false; src.len()];
    for span in spans {
        for i in span.byte_range() {
            dead_bytes[i] = true;
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

fn remove_dead_code_inner(src: String) -> anyhow::Result<String> {
    let dead_code_diagnostics: Vec<Diagnostic> = rustc_diagnostics(&src)?
        .into_iter()
        .filter(|d| d.code.as_ref().map_or(false, |c| c.code == "dead_code" || c.code == "unused_imports"))
        .collect();

    let dead_code_diagnostic_spans: Vec<DiagnosticSpan> = dead_code_diagnostics
        .into_iter()
        .map(|d| d.spans.into_iter().next().unwrap())
        .collect();

    let ast = syn::parse_file(&src)?;
    let mut visitor = DeadIdentifierVisitor::new(dead_code_diagnostic_spans);
    visitor.visit_file(&ast);

    let mut visitor = DeadCodeVisitor::new(
        visitor.output_dead_struct_identifiers,
        visitor.output_dead_fn_identifiers,
        visitor.output_dead_trait_identifiers,
        visitor.output_dead_use_identifiers,
    );
    visitor.visit_file(&ast);

    let src= remove_spans(src, visitor.output_dead_spans)?;

    // TODO: Remove empty modules

    Ok(src)
}

pub fn remove_dead_code(mut src: String) -> anyhow::Result<String> {
    loop {
        let new_src = remove_dead_code_inner(src.clone())?;
        if src == new_src {
            return Ok(src);
        } else {
            src = new_src;
        }
    }
}


#[derive(Default)]
/// 🍻
struct PubVisitor {
    output_pub_token_spans: Vec<Span>
}

impl<'ast> Visit<'ast> for PubVisitor {
    fn visit_visibility(&mut self, i: &'ast syn::Visibility) {
        match i {
            syn::Visibility::Public(token) => self.output_pub_token_spans.push(token.span()),
            _ => (),
        }
    }
}

pub fn replace_pub_with_pub_crate(src: String) -> anyhow::Result<String> {
    let ast = syn::parse_file(&src)?;
    let mut pub_visitor = PubVisitor::default();
    pub_visitor.visit_file(&ast);
    let output_pub_token_spans = pub_visitor.output_pub_token_spans;
    let span_start_bytes: BTreeSet<usize> = output_pub_token_spans.iter().map(|span| span.byte_range().start).collect();
    let span_bytes: BTreeSet<usize> = output_pub_token_spans.iter().map(|span| span.byte_range().into_iter()).flatten().collect();
    let mut new_src: Vec<u8> = Vec::new();
    for (i, b) in src.bytes().enumerate() {
        if span_start_bytes.contains(&i) {
            new_src.extend("pub(crate)".bytes());
        } else if span_bytes.contains(&i) {
            ()
        } else {
            new_src.push(b);
        }
    }
    let new_src = String::try_from(new_src)?;
    Ok(new_src)
}