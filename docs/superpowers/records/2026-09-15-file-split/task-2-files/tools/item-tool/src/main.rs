//! Lists the items of a Rust file as units a pure move preserves, one line
//! per item: key, first and last line, visibility, token stream, literals.
//!
//! Units: each top-level item; each member of an `impl` block; each row of a
//! `static` whose value is an array of tuples (the native method tables);
//! each item of an inline `mod` (recursively, keyed with the module path).
//! The token stream and literals are computed from the RAW source lines of
//! the unit, re-tokenized, so a literal inside a macro argument is seen.
//! A leading visibility (`pub`, `pub(crate)`, `pub(super)`, ...) is removed
//! from the token stream and reported in its own column.

use std::env;
use std::fs;
use std::str::FromStr;

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Expr, ImplItem, Item, Lit};

struct Unit {
    key: String,
    first: usize,
    last: usize,
}

fn attrs_first(attrs: &[syn::Attribute], fallback: usize) -> usize {
    attrs
        .iter()
        .map(|a| a.span().start().line)
        .chain(std::iter::once(fallback))
        .min()
        .unwrap()
}

fn ty_name(ty: &syn::Type) -> String {
    ty.to_token_stream().to_string().replace(' ', "")
}

fn item_name(item: &Item) -> String {
    match item {
        Item::Const(i) => format!("const {}", i.ident),
        Item::Enum(i) => format!("enum {}", i.ident),
        Item::Fn(i) => format!("fn {}", i.sig.ident),
        Item::Mod(i) => format!("mod {}", i.ident),
        Item::Static(i) => format!("static {}", i.ident),
        Item::Struct(i) => format!("struct {}", i.ident),
        Item::Type(i) => format!("type {}", i.ident),
        Item::Use(i) => format!("use {}", i.tree.to_token_stream().to_string().replace(' ', "")),
        Item::Trait(i) => format!("trait {}", i.ident),
        Item::Macro(i) => format!("macro {}", i.mac.path.to_token_stream()),
        other => format!("other@{}", other.span().start().line),
    }
}

fn item_attrs(item: &Item) -> &[syn::Attribute] {
    match item {
        Item::Const(i) => &i.attrs,
        Item::Enum(i) => &i.attrs,
        Item::Fn(i) => &i.attrs,
        Item::Mod(i) => &i.attrs,
        Item::Static(i) => &i.attrs,
        Item::Struct(i) => &i.attrs,
        Item::Type(i) => &i.attrs,
        Item::Use(i) => &i.attrs,
        Item::Trait(i) => &i.attrs,
        Item::Impl(i) => &i.attrs,
        Item::Macro(i) => &i.attrs,
        _ => &[],
    }
}

fn collect(items: &[Item], prefix: &str, out: &mut Vec<Unit>) {
    for item in items {
        let span = item.span();
        let first = attrs_first(item_attrs(item), span.start().line);
        let last = span.end().line;
        match item {
            Item::Impl(imp) => {
                let head = match &imp.trait_ {
                    Some((_, path, _)) => format!(
                        "impl {} for {}",
                        path.to_token_stream().to_string().replace(' ', ""),
                        ty_name(&imp.self_ty)
                    ),
                    None => format!("impl {}", ty_name(&imp.self_ty)),
                };
                for member in &imp.items {
                    let (name, attrs) = match member {
                        ImplItem::Fn(f) => (format!("fn {}", f.sig.ident), &f.attrs),
                        ImplItem::Const(c) => (format!("const {}", c.ident), &c.attrs),
                        ImplItem::Type(t) => (format!("type {}", t.ident), &t.attrs),
                        other => (format!("other@{}", other.span().start().line), &Vec::new()),
                    };
                    let ms = member.span();
                    out.push(Unit {
                        key: format!("{prefix}{head}::{name}"),
                        first: attrs_first(attrs, ms.start().line),
                        last: ms.end().line,
                    });
                }
            }
            Item::Mod(m) if m.content.is_some() => {
                let (_, inner) = m.content.as_ref().unwrap();
                collect(inner, &format!("{prefix}{}::", m.ident), out);
            }
            Item::Static(st) => {
                if let Expr::Reference(r) = &*st.expr {
                    if let Expr::Array(arr) = &*r.expr {
                        for elem in &arr.elems {
                            if let Expr::Tuple(t) = elem {
                                let lits: Vec<String> = t
                                    .elems
                                    .iter()
                                    .take(2)
                                    .map(|e| match e {
                                        Expr::Lit(l) => match &l.lit {
                                            Lit::Str(s) => s.value(),
                                            _ => "?".into(),
                                        },
                                        _ => "?".into(),
                                    })
                                    .collect();
                                let es = elem.span();
                                out.push(Unit {
                                    key: format!(
                                        "{prefix}row {}/{}/{:?}",
                                        st.ident, lits[0], lits[1]
                                    ),
                                    first: es.start().line,
                                    last: es.end().line,
                                });
                            }
                        }
                        continue;
                    }
                }
                out.push(Unit {
                    key: format!("{prefix}{}", item_name(item)),
                    first,
                    last,
                });
            }
            _ => out.push(Unit {
                key: format!("{prefix}{}", item_name(item)),
                first,
                last,
            }),
        }
    }
}

/// Flattened token texts, with each group's delimiters as tokens.
fn flatten(stream: TokenStream, out: &mut Vec<String>) {
    for tree in stream {
        match tree {
            TokenTree::Group(g) => {
                let (open, close) = match g.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::None => ("<none>", "</none>"),
                };
                out.push(open.into());
                flatten(g.stream(), out);
                out.push(close.into());
            }
            TokenTree::Punct(p) => out.push(format!(
                "{}{}",
                p.as_char(),
                if p.spacing() == Spacing::Joint { "+" } else { "" }
            )),
            // Whitespace-stripped, so a continued string literal re-indented
            // compares equal here; its value is instrument 3's subject.
            other => out.push(other.to_string().split_whitespace().collect()),
        }
    }
}

fn literals(stream: TokenStream, out: &mut Vec<String>) {
    for tree in stream {
        match tree {
            TokenTree::Group(g) => literals(g.stream(), out),
            TokenTree::Literal(lit) => out.push(match Lit::new(lit) {
                Lit::Str(l) => format!("STR {:?}", l.value()),
                Lit::ByteStr(l) => format!("BYTESTR {:?}", l.value()),
                Lit::Byte(l) => format!("BYTE {:?}", l.value()),
                Lit::Char(l) => format!("CHAR {:?}", l.value()),
                Lit::CStr(l) => format!("CSTR {:?}", l.value()),
                Lit::Int(l) => format!("INT {}", l),
                Lit::Float(l) => format!("FLOAT {}", l),
                Lit::Bool(l) => format!("BOOL {}", l.value),
                other => format!("OTHER {}", other.to_token_stream()),
            }),
            _ => {}
        }
    }
}

const ITEM_WORDS: &[&str] = &[
    "fn", "const", "static", "struct", "enum", "type", "mod", "use", "trait", "async", "unsafe",
];

/// Removes the first visibility that directly precedes an item keyword,
/// outside any attribute, and returns it.
fn strip_vis(tokens: &mut Vec<String>) -> String {
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i] == "#" {
            // skip `#` `[` ... `]`
            if i + 1 < tokens.len() && tokens[i + 1] == "[" {
                let mut depth = 0;
                let mut j = i + 1;
                loop {
                    if tokens[j] == "[" {
                        depth += 1
                    } else if tokens[j] == "]" {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                i = j + 1;
                continue;
            }
        }
        if tokens[i] == "pub" {
            let mut end = i + 1;
            if end < tokens.len() && tokens[end] == "(" {
                while tokens[end] != ")" {
                    end += 1;
                }
                end += 1;
            }
            if end < tokens.len() && ITEM_WORDS.contains(&tokens[end].as_str()) {
                let vis: Vec<String> = tokens.drain(i..end).collect();
                return vis.join("");
            }
        }
        return "private".into();
    }
    "private".into()
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t").replace('\r', "\\r")
}

fn main() {
    let path = env::args().nth(1).expect("usage: item-tool FILE");
    let src = fs::read_to_string(&path).unwrap();
    let file = syn::parse_file(&src).unwrap_or_else(|e| panic!("{path}: {e}"));
    let lines: Vec<&str> = src.split_inclusive('\n').collect();
    let mut units = Vec::new();
    collect(&file.items, "", &mut units);
    for u in &units {
        let text: String = lines[u.first - 1..u.last].concat();
        let stream = TokenStream::from_str(&text)
            .unwrap_or_else(|e| panic!("{path}:{}-{}: {e}", u.first, u.last));
        let mut toks = Vec::new();
        flatten(stream.clone(), &mut toks);
        let vis = strip_vis(&mut toks);
        let mut lits = Vec::new();
        literals(stream, &mut lits);
        let decodable = lits
            .iter()
            .filter(|l| {
                ["STR ", "BYTESTR ", "BYTE ", "CHAR ", "CSTR "]
                    .iter()
                    .any(|p| l.starts_with(p))
            })
            .count();
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            u.key,
            u.first,
            u.last,
            vis,
            decodable,
            esc(&toks.join("\u{1}")),
            esc(&lits.join("\u{1}"))
        );
    }
}
