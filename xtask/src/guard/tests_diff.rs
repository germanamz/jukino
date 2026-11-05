//! Finds the test code in a Rust file and compares two versions of it.
//!
//! Test code is every `#[test]` function; inside a file under a `tests` directory or a
//! `#[cfg(test)]` module, it is also every other item (helpers, constants, macros) and
//! the attributes of the file and of each module. Additions are allowed; any change
//! to, or removal of, existing test code is reported.

use std::collections::BTreeMap;
use std::path::Path;

use quote::ToTokens as _;
use syn::{Attribute, Item, ItemFn, ItemMod};

/// What the comparison keeps about one piece of test code.
struct TestItem {
    line: usize,
    tokens: String,
    test_fn: Option<TestFlags>,
    /// File or module attributes: compared when both versions have them, never reported
    /// as deleted (the tests they held are reported one by one).
    attributes_only: bool,
}

/// The attributes that change what a `#[test]` function does.
#[derive(Clone, Copy)]
struct TestFlags {
    ignored: bool,
    should_panic: bool,
}

/// Compares the test code of the file at `path` in `base` with `current` (`None` when
/// the file was deleted) and returns `(line, message)` pairs, ordered by line.
/// Formatting and plain comments are not changes; any token change is.
///
/// # Errors
///
/// Fails when either version is not valid Rust.
pub fn compare(
    path: &str,
    base: &str,
    current: Option<&str>,
) -> Result<Vec<(usize, String)>, syn::Error> {
    let test_file = is_test_file(path);
    let before = test_code(&syn::parse_file(base)?, test_file);
    let Some(current) = current else {
        let deleted = (!before.is_empty()).then(|| (1, "test file deleted".to_owned()));
        return Ok(deleted.into_iter().collect());
    };
    let after = test_code(&syn::parse_file(current)?, test_file);
    let mut changes: Vec<(usize, String)> = before
        .iter()
        .filter_map(|(key, item)| change(key, item, after.get(key)))
        .collect();
    changes.sort();
    Ok(changes)
}

fn is_test_file(path: &str) -> bool {
    Path::new(path)
        .components()
        .any(|component| component.as_os_str() == "tests")
}

fn change(key: &str, before: &TestItem, after: Option<&TestItem>) -> Option<(usize, String)> {
    let label = |what: &str| match before.test_fn {
        Some(_) => format!("test {key} {what}"),
        None => format!("test code `{key}` {what}"),
    };
    let Some(after) = after else {
        return (!before.attributes_only).then(|| (before.line, label("deleted or renamed")));
    };
    if after.tokens == before.tokens {
        return None;
    }
    let what = match (before.test_fn, after.test_fn) {
        (Some(was), Some(now)) if now.ignored && !was.ignored => "gained #[ignore]",
        (Some(was), Some(now)) if now.should_panic && !was.should_panic => "gained #[should_panic]",
        _ => "changed",
    };
    Some((after.line, label(what)))
}

fn test_code(file: &syn::File, test_file: bool) -> BTreeMap<String, TestItem> {
    let mut items = BTreeMap::new();
    if test_file {
        items.insert(
            "file attributes".to_owned(),
            attributes_item(&file.attrs, 1),
        );
    }
    collect(&file.items, &mut Vec::new(), test_file, &mut items);
    items
}

fn collect(
    items: &[Item],
    modules: &mut Vec<String>,
    in_test_code: bool,
    out: &mut BTreeMap<String, TestItem>,
) {
    for item in items {
        if let Item::Mod(module) = item {
            collect_module(module, modules, in_test_code, out);
        } else if let Item::Fn(function) = item
            && has_attribute(&function.attrs, "test")
        {
            out.insert(
                qualified(modules, &function.sig.ident.to_string()),
                test_fn_item(function),
            );
        } else if in_test_code && let Some((kind, name, line)) = named(item) {
            let tokens = item.to_token_stream().to_string();
            out.insert(
                format!("{kind} {}", qualified(modules, &name)),
                TestItem {
                    line,
                    tokens,
                    test_fn: None,
                    attributes_only: false,
                },
            );
        }
    }
}

/// Records a module's attributes when it is, or holds, test code, so that disabling
/// it (for example by changing `cfg(test)`) is reported as a change.
fn collect_module(
    module: &ItemMod,
    modules: &mut Vec<String>,
    in_test_code: bool,
    out: &mut BTreeMap<String, TestItem>,
) {
    let content = module
        .content
        .as_ref()
        .map_or(&[][..], |(_, items)| items.as_slice());
    let test_module = in_test_code || is_cfg_test(&module.attrs);
    modules.push(module.ident.to_string());
    if test_module || holds_tests(content) {
        let line = module.ident.span().start().line;
        out.insert(
            format!("mod {}", modules.join("::")),
            attributes_item(&module.attrs, line),
        );
    }
    collect(content, modules, test_module, out);
    modules.pop();
}

fn holds_tests(items: &[Item]) -> bool {
    items.iter().any(|item| {
        if let Item::Fn(function) = item {
            has_attribute(&function.attrs, "test")
        } else if let Item::Mod(module) = item {
            module
                .content
                .as_ref()
                .is_some_and(|(_, inner)| holds_tests(inner))
        } else {
            false
        }
    })
}

fn qualified(modules: &[String], name: &str) -> String {
    let mut path = modules.to_vec();
    path.push(name.to_owned());
    path.join("::")
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "syn::Item is non-exhaustive; items without a name hold no test code to track"
)]
fn named(item: &Item) -> Option<(&'static str, String, usize)> {
    let (kind, ident) = match item {
        Item::Fn(item) => ("fn", &item.sig.ident),
        Item::Const(item) => ("const", &item.ident),
        Item::Static(item) => ("static", &item.ident),
        Item::Struct(item) => ("struct", &item.ident),
        Item::Enum(item) => ("enum", &item.ident),
        Item::Type(item) => ("type", &item.ident),
        Item::Trait(item) => ("trait", &item.ident),
        Item::Macro(item) => ("macro", item.ident.as_ref()?),
        _ => return None,
    };
    Some((kind, ident.to_string(), ident.span().start().line))
}

fn attributes_item(attributes: &[Attribute], line: usize) -> TestItem {
    let tokens = attributes
        .iter()
        .map(|attribute| attribute.to_token_stream().to_string())
        .collect();
    TestItem {
        line,
        tokens,
        test_fn: None,
        attributes_only: true,
    }
}

fn test_fn_item(function: &ItemFn) -> TestItem {
    TestItem {
        line: function.sig.ident.span().start().line,
        tokens: function.to_token_stream().to_string(),
        test_fn: Some(TestFlags {
            ignored: has_attribute(&function.attrs, "ignore"),
            should_panic: has_attribute(&function.attrs, "should_panic"),
        }),
        attributes_only: false,
    }
}

fn has_attribute(attributes: &[Attribute], name: &str) -> bool {
    attributes.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == name)
    })
}

fn is_cfg_test(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg") && attribute.to_token_stream().to_string().contains("test")
    })
}
