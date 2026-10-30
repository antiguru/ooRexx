use std::{collections::BTreeMap, path::PathBuf};
fn corpus_dir() -> PathBuf { PathBuf::from("/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus") }
mod table_c {
    #[path = "/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/tests/table_c/rows.rs"]
    pub mod rows;
    #[path = "/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/tests/table_c/probes.rs"]
    pub mod probes;
}
fn emit(relative: &str, text: String, count: &mut usize) {
    let file = corpus_dir().join(relative);
    let old = std::fs::read_to_string(&file).unwrap();
    assert_eq!(old.replace("crates/rexx-exec/tests/gate_table_c.rs", "crates/rexx-exec/tests/table_c/probes.rs"), text, "unintended body change in {relative}");
    if old != text { std::fs::write(file, text).unwrap(); *count += 1; }
}
fn main() {
    use table_c::{rows::*, probes::*};
    let classes = read_classes();
    let mut count = 0;
    for row in &classes { emit(&class_probe(&row.name), class_probe_text(&row.name), &mut count); }
    for edge in read_edges() { emit(&edge_probe(&edge.child, &edge.parent), edge_probe_text(&edge.child, &edge.parent), &mut count); }
    let methods = read_method_rows();
    let mut groups: BTreeMap<(&str, &str), Vec<&str>> = BTreeMap::new();
    for row in &methods { groups.entry((&row.class, &row.arm)).or_default().push(&row.method); }
    for ((class, arm), names) in groups {
        let row = classes.iter().find(|row| row.name == class).unwrap();
        emit(&method_probe(class, arm), method_probe_text(class, arm, &row.status, &row.reason, &row.construction, &names), &mut count);
    }
    println!("Regenerated {count} probes through the existing Table C derivation. All differences are the intended source path literal.");
}
