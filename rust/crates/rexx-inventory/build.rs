//! Derives Rust tables from the C++ tree at build time.
//!
//! The C++ tree is the source of truth. Nothing here is hand-maintained, and
//! nothing generated is written into `src/` -- it all goes to `OUT_DIR` and is
//! `include!`d, so a stale copy cannot be committed by accident.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const MESSAGES_XML: &str = "../../../interpreter/messages/rexxmsg.xml";
const BUILTINS_CPP: &str = "../../../interpreter/expression/BuiltinFunctions.cpp";

fn main() {
    println!("cargo::rerun-if-changed={MESSAGES_XML}");
    println!("cargo::rerun-if-changed={BUILTINS_CPP}");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    std::fs::write(
        out.join("errors.rs"),
        generate_errors(Path::new(MESSAGES_XML)),
    )
    .expect("writing errors.rs");
    std::fs::write(
        out.join("builtins.rs"),
        generate_builtins(Path::new(BUILTINS_CPP)),
    )
    .expect("writing builtins.rs");
}

// ---------------------------------------------------------------- messages --

#[derive(Default, Clone)]
struct Message {
    major: u16,
    sub: u16,
    number: u16,
    symbol: String,
    text: String,
}

/// Which `<Message>`/`<SubMessage>` child we are currently accumulating.
#[derive(PartialEq)]
enum Field {
    None,
    Code,
    Subcode,
    Number,
    Symbol,
    Text,
}

fn generate_errors(path: &Path) -> String {
    let xml = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut reader = Reader::from_str(&xml);
    let mut messages: Vec<Message> = Vec::new();

    // A <Message> may contain <Subcodes><SubMessage>..., and a SubMessage has
    // the same child element names as its parent. Keep a stack so a
    // submessage's <Code> never overwrites the major's.
    let mut stack: Vec<Message> = Vec::new();
    let mut field = Field::None;
    // <Text> holds mixed content: character data interleaved with <q>, <sq/>,
    // <dq/> and <Sub/>. Accumulate rendered pieces rather than raw text.
    let mut depth_in_text = 0usize;

    loop {
        match reader.read_event() {
            Err(e) => panic!("malformed XML in {}: {e}", path.display()),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Message" | b"SubMessage" => {
                    stack.push(Message::default());
                    field = Field::None;
                }
                b"Code" => field = Field::Code,
                b"Subcode" => field = Field::Subcode,
                b"MessageNumber" => field = Field::Number,
                b"SymbolicName" => field = Field::Symbol,
                b"Text" => {
                    field = Field::Text;
                    depth_in_text = 0;
                }
                // Rendering rules copied from RexxErrorMessages.xsl:86-96.
                // <q> keeps its quotes -- it is not documentation-only markup.
                b"q" if field == Field::Text => {
                    depth_in_text += 1;
                    if let Some(m) = stack.last_mut() {
                        m.text.push('"');
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e)) if field == Field::Text => {
                let rendered = match e.name().as_ref() {
                    b"Sub" => {
                        let pos = e
                            .attributes()
                            .flatten()
                            .find(|a| a.key.as_ref() == b"position")
                            .map(|a| String::from_utf8_lossy(&a.value).into_owned())
                            .expect("every <Sub> carries a position");
                        format!("&{pos}")
                    }
                    b"sq" => "'".to_string(),
                    b"dq" => "\"".to_string(),
                    _ => String::new(),
                };
                if let Some(m) = stack.last_mut() {
                    m.text.push_str(&rendered);
                }
            }
            Ok(Event::Text(t)) => {
                let raw = t.unescape().expect("well-formed entity").into_owned();
                if let Some(m) = stack.last_mut() {
                    match field {
                        Field::Code => m.major = parse_num(&raw, "Code"),
                        Field::Subcode => m.sub = parse_num(&raw, "Subcode"),
                        Field::Number => m.number = parse_num(&raw, "MessageNumber"),
                        Field::Symbol => m.symbol.push_str(raw.trim()),
                        Field::Text => m.text.push_str(&raw),
                        Field::None => {}
                    }
                }
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"q" if field == Field::Text && depth_in_text > 0 => {
                    depth_in_text -= 1;
                    if let Some(m) = stack.last_mut() {
                        m.text.push('"');
                    }
                }
                b"Message" | b"SubMessage" => {
                    let done = stack.pop().expect("balanced start/end");
                    if done.symbol.is_empty() {
                        panic!("message {}.{:03} has no SymbolicName", done.major, done.sub);
                    }
                    if done.text.is_empty() {
                        panic!("message {}.{:03} has no Text", done.major, done.sub);
                    }
                    messages.push(done);
                    field = Field::None;
                }
                b"Code" | b"Subcode" | b"MessageNumber" | b"SymbolicName" | b"Text" => {
                    field = Field::None;
                }
                _ => {}
            },
            _ => {}
        }
    }

    if messages.is_empty() {
        panic!("no messages parsed from {}", path.display());
    }
    // A silently colliding key would let a later phase report false
    // conformance, so refuse to emit a table that has one.
    let mut keys: Vec<(u16, u16)> = messages.iter().map(|m| (m.major, m.sub)).collect();
    keys.sort_unstable();
    for pair in keys.windows(2) {
        if pair[0] == pair[1] {
            panic!("duplicate message key {}.{:03}", pair[0].0, pair[0].1);
        }
    }

    let mut out = String::from(
        "// Generated by build.rs from interpreter/messages/rexxmsg.xml. Do not edit.\n\
         pub struct Message {\n\
         \x20   pub major: u16,\n\
         \x20   pub sub: u16,\n\
         \x20   pub number: u16,\n\
         \x20   pub symbol: &'static str,\n\
         \x20   pub text: &'static str,\n\
         }\n\n\
         pub fn lookup(major: u16, sub: u16) -> Option<&'static Message> {\n\
         \x20   MESSAGES.iter().find(|m| m.major == major && m.sub == sub)\n\
         }\n\n\
         pub static MESSAGES: &[Message] = &[\n",
    );
    for m in &messages {
        writeln!(
            out,
            "    Message {{ major: {}, sub: {}, number: {}, symbol: {:?}, text: {:?} }},",
            m.major, m.sub, m.number, m.symbol, m.text
        )
        .expect("writing to a String cannot fail");
    }
    out.push_str("];\n");
    out
}

fn parse_num(raw: &str, what: &str) -> u16 {
    raw.trim()
        .parse()
        .unwrap_or_else(|e| panic!("{what} {raw:?} is not a number: {e}"))
}

// ---------------------------------------------------------------- builtins --

fn generate_builtins(path: &Path) -> String {
    let src = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let start = src
        .find("pbuiltin LanguageParser::builtinTable[] =")
        .expect("the builtin table is still declared in BuiltinFunctions.cpp");
    let table = &src[start..];
    let end = table.find("\n};").expect("the builtin table is terminated");

    let names: Vec<&str> = table[..end]
        .lines()
        .filter_map(|line| line.trim().strip_prefix("&builtin_function_"))
        .map(|rest| rest.trim_end_matches(&[' ', ',', '\t'][..]).trim())
        .collect();

    // 81 entries as of 8c880bdd. The floor catches a broken parse without
    // tripping on the real count; it is deliberately well below it.
    if names.len() < 50 {
        panic!(
            "only {} builtin names parsed -- the table format changed",
            names.len()
        );
    }

    let mut out = String::from(
        "// Generated by build.rs from interpreter/expression/BuiltinFunctions.cpp.\n\
         // Do not edit. Order is table order, which is the index the parser\n\
         // resolves builtins through -- do not sort it.\n\
         pub static NAMES: &[&str] = &[\n",
    );
    for n in &names {
        writeln!(out, "    {n:?},").expect("writing to a String cannot fail");
    }
    out.push_str("];\n");
    out
}
