//! Forbidden standard-library modules in the core's sources.
//!
//! The core reaches files, the network, the environment, processes, I/O streams and
//! threads only through ports, so its sources must not name the `std` modules that
//! provide them. This complements the clippy `disallowed-*` lists in the core's
//! `clippy.toml`, which catch calls but not imports.

use std::path::Path;

use proc_macro2::Span;
use syn::UseTree;
use syn::visit::Visit;

use crate::finding::Finding;

/// The `std` modules the core must not name.
pub const FORBIDDEN_MODULES: [&str; 6] = ["fs", "net", "env", "process", "io", "thread"];

/// Checks one core source file for paths under [`FORBIDDEN_MODULES`].
///
/// # Errors
///
/// Fails when `source` is not valid Rust.
pub fn check_source(path: &Path, source: &str) -> Result<Vec<Finding>, syn::Error> {
    let file = syn::parse_file(source)?;
    let mut visitor = Visitor::default();
    visitor.visit_file(&file);
    Ok(visitor
        .hits
        .into_iter()
        .map(|(line, module)| Finding {
            path: path.to_path_buf(),
            line,
            message: format!(
                "core must not use `std::{module}`: reach the outside world through a port"
            ),
        })
        .collect())
}

#[derive(Default)]
struct Visitor {
    hits: Vec<(usize, &'static str)>,
}

impl Visitor {
    fn record(&mut self, segments: &[String], span: Span) {
        if let [first, second, ..] = segments
            && first == "std"
            && let Some(module) = FORBIDDEN_MODULES
                .iter()
                .find(|module| **module == second.as_str())
        {
            self.hits.push((span.start().line, module));
        }
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments: Vec<String> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let span = path
            .segments
            .first()
            .map_or_else(Span::call_site, |segment| segment.ident.span());
        self.record(&segments, span);
        syn::visit::visit_path(self, path);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        let mut paths = Vec::new();
        flatten(&item.tree, &mut Vec::new(), &mut paths);
        for (segments, span) in paths {
            self.record(&segments, span);
        }
    }
}

fn flatten(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<(Vec<String>, Span)>) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            flatten(&path.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(name) => out.push((extended(prefix, &name.ident), name.ident.span())),
        UseTree::Rename(rename) => out.push((extended(prefix, &rename.ident), rename.ident.span())),
        UseTree::Glob(glob) => out.push((prefix.clone(), glob.star_token.span)),
        UseTree::Group(group) => {
            for item in &group.items {
                flatten(item, prefix, out);
            }
        }
    }
}

fn extended(prefix: &[String], ident: &syn::Ident) -> Vec<String> {
    let mut path = prefix.to_vec();
    path.push(ident.to_string());
    path
}
