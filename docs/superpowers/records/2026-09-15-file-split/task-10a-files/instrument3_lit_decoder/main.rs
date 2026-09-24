use std::env;
use std::fs;
use std::str::FromStr;

use proc_macro2::{TokenStream, TokenTree};
use syn::Lit;

/// Every literal token in `stream`, in source order, decoded to its
/// semantic value. Walks the raw token stream rather than a parsed
/// `syn::File`/`syn::Item`: a typed AST does not descend into a macro
/// invocation's arguments (`writeln!(f, "...")`'s `"..."` is inside an
/// opaque `Macro::tokens` `TokenStream` that `syn::visit::Visit` never
/// looks inside), but at the raw-token level a macro's parenthesized
/// argument list is an ordinary `Group`, indistinguishable from any other,
/// so recursing into every `Group` reaches every literal regardless of
/// what syntactic position it sits in.
fn collect(stream: TokenStream, out: &mut Vec<String>) {
    for tree in stream {
        match tree {
            TokenTree::Group(g) => collect(g.stream(), out),
            TokenTree::Literal(lit) => {
                let decoded = match Lit::new(lit) {
                    Lit::Str(l) => format!("STR {:?}", l.value()),
                    Lit::ByteStr(l) => format!("BYTESTR {:?}", l.value()),
                    Lit::Byte(l) => format!("BYTE {:?}", l.value()),
                    Lit::Char(l) => format!("CHAR {:?}", l.value()),
                    Lit::Int(l) => format!("INT {:?}", l.base10_digits()),
                    Lit::Float(l) => format!("FLOAT {:?}", l.base10_digits()),
                    Lit::Bool(l) => format!("BOOL {:?}", l.value),
                    Lit::Verbatim(l) => format!("VERBATIM {:?}", l.to_string()),
                    other => format!("OTHER {other:?}"),
                };
                out.push(decoded);
            }
            TokenTree::Ident(_) | TokenTree::Punct(_) => {}
        }
    }
}

fn main() {
    let path = env::args().nth(1).expect("usage: lit-decoder <file.rs>");
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let stream = TokenStream::from_str(&src)
        .unwrap_or_else(|e| panic!("{path}: tokenize error: {e}\n---\n{src}\n---"));
    let mut out = Vec::new();
    collect(stream, &mut out);
    for line in &out {
        println!("{line}");
    }
    // A structural self-count, printed to stderr so it never mixes into the
    // literal list stdout carries: how many of the collected tokens are
    // Str/ByteStr/Char/Byte (the kinds instrument 3 is about -- the ones
    // with escape/continuation decoding to get right), for the shell
    // wrapper to compare against the independent lexer-level count.
    let decodable = out
        .iter()
        .filter(|l| {
            l.starts_with("STR ") || l.starts_with("BYTESTR ") || l.starts_with("CHAR ") || l.starts_with("BYTE ")
        })
        .count();
    eprintln!("decodable_count={decodable}");
}
