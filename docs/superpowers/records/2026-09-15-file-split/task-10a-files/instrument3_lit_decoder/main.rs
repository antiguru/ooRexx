use std::env;
use std::fs;

use syn::visit::{self, Visit};
use syn::Lit;

struct LitCollector {
    out: Vec<String>,
}

impl<'ast> Visit<'ast> for LitCollector {
    fn visit_lit(&mut self, lit: &'ast Lit) {
        let line = match lit {
            Lit::Str(l) => format!("STR {:?}", l.value()),
            Lit::ByteStr(l) => format!("BYTESTR {:?}", l.value()),
            Lit::Byte(l) => format!("BYTE {:?}", l.value()),
            Lit::Char(l) => format!("CHAR {:?}", l.value()),
            Lit::Int(l) => format!("INT {:?}", l.base10_digits()),
            Lit::Float(l) => format!("FLOAT {:?}", l.base10_digits()),
            Lit::Bool(l) => format!("BOOL {:?}", l.value),
            Lit::Verbatim(l) => format!("VERBATIM {:?}", l.to_string()),
            _ => format!("OTHER {:?}", lit),
        };
        self.out.push(line);
        visit::visit_lit(self, lit);
    }
}

fn main() {
    let path = env::args().nth(1).expect("usage: lit-decoder <file.rs>");
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    // Each input file holds exactly one top-level item's source text
    // (a fn, struct, const, or impl block), extracted verbatim from the
    // surrounding file. Parsed as a standalone syn::File since that also
    // accepts a bare sequence of items with no wrapping required.
    let file: syn::File = syn::parse_str(&src)
        .unwrap_or_else(|e| panic!("{path}: syn parse error: {e}\n---\n{src}\n---"));
    let mut collector = LitCollector { out: Vec::new() };
    collector.visit_file(&file);
    for line in collector.out {
        println!("{line}");
    }
}
