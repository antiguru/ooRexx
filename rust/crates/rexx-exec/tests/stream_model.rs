/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! The stream model, `corpus/model/stream.rex`, against this crate's native
//! streams on real files: the named cases in `tests/stream_model/` and
//! generated operation sequences. Each scenario runs natively and through the
//! model in one program, and the two answers are compared item by item and
//! by the final file bytes. On this crate they agree except where a
//! [`REPORTED`] rule says the crate and the documentation part; ANSI X3.274
//! section 9.7's two properties hold of every native run; and on the oracle,
//! with every stream opened `NOBUFFER`, the native answer differs from the
//! model exactly where [`ORACLE_DEPARTURES`] and
//! `tests/stream_model/departures-oracle.tsv` say. Both recorded sets,
//! `departures-crate.tsv` and `departures-oracle.tsv`, are rewritten by
//! `REXX_STREAM_MODEL_REFRESH=1`; `REXX_STREAM_MODEL_DUMP=<dir>` appends
//! every answer to `<dir>/crate.tsv` and `<dir>/oracle.tsv`.
//!
//! An operation is one of these, `f` the scenario's stream:
//! * `O opts` `stream(f,'c','open' opts)`, `X` `stream(f,'c','close')`,
//!   `CX` `charout(f)`, `LX` `lineout(f)`;
//! * `CO s [n]` `charout(f,s[,n])`, `CE` `charout(f,'')`, `CP n`
//!   `charout(f,,n)`;
//! * `LO s [n]` `lineout(f,s[,n])`, `LE` `lineout(f,'')`, `LP n`
//!   `lineout(f,,n)`;
//! * `CI [k [n]]` `charin(f,[n],[k])`, `LI [n [k]]` `linein(f,[n],[k])`,
//!   `LZ` `linein(f,,0)`;
//! * `CH` `chars(f)`, `LN` `lines(f)`, `LC` `lines(f,'C')`;
//! * `S spec` `stream(f,'c','seek' spec)`, `Q spec`
//!   `stream(f,'c','query position' spec)`.

#![expect(
    clippy::disallowed_methods,
    reason = "this harness times or bounds real runs"
)]

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use rexx_exec::{Invocation, run_program};
use support::oracle::{did_not_finish, locate};

const CASES: &str = "tests/stream_model/cases";

/// The rules by which this crate and the oracle both depart from the
/// model, which follows the documentation. Phase 9d Task 6.15 reports each
/// to Moritz; none is ruled:
/// * `write-only`: a stream opened WRITE reads (funct.xml:4302 opens it
///   "for writing only");
/// * `line-position`: a line SEEK, LINEIN or LINEOUT counts lines from a
///   tracked line number that character operations zero or leave behind,
///   not from the line the position is in (iostrms.xml:463-490);
/// * `line-query`: QUERY POSITION ... LINE answers from the same tracker,
///   and for a write position inside the stream one line too many
///   (funct.xml:4550-4555);
/// * `lines-count`: LINES Count answers from a cached count that is one
///   short once a character operation has run (funct.xml:3306-3307).
const REPORTED: &[&str] = &["write-only", "line-position", "line-query", "lines-count"];

/// The named cases that show a [`REPORTED`] rule: this crate and the oracle
/// answer alike there, and differ from the model.
const REPORTED_CASES: &[(&str, &str)] = &[
    ("write-only-reads", "write-only"),
    ("line-seek-relative-after-charin", "line-position"),
    ("linein-line-from-mid-line", "line-position"),
    ("write-line-query", "line-query"),
    ("read-line-at-end", "line-query"),
    ("read-line-after-positioning", "line-query"),
    ("lines-count-repeated", "lines-count"),
];

/// The named cases whose native oracle answer differs from the model where
/// this crate's does not: the case, the Deviation that licenses it (0 for a
/// departure no Deviation covers, which Phase 9d Task 6.15 reports), the
/// rule, and the oracle's first item that differs.
const ORACLE_DEPARTURES: &[(&str, u32, &str, &str)] = &[
    (
        "record-close-pads-nothing",
        46,
        "pads-on-close",
        "[AAAAx___CCCCDDDD____]",
    ),
    (
        "seek-char-from-end",
        49,
        "seek-from-end",
        "S_<3_read=22 READY 19 22",
    ),
    (
        "seek-relative",
        49,
        "relative-seek",
        "S_+2_read=14 READY 12 14",
    ),
    (
        "transient-buffered-writes",
        48,
        "transient-no-seek",
        "CO_de=2 ERROR 1 1",
    ),
    (
        "transient-counts",
        0,
        "transient-counts-read",
        "CI_2=de READY 1 1",
    ),
];

/// A file's content before a scenario, by name.
fn initial(name: &str) -> &'static [u8] {
    match name {
        "text3" => b"line1\nline2\nline3\n",
        "tail" => b"ab\ncd\nef",
        "rec16" => b"AAAABBBBCCCCDDDD",
        "empty" => b"",
        other => panic!("unknown initial content {other}"),
    }
}

/// One scenario: its name, its initial content's name, whether the stream is
/// a FIFO, and its operations.
#[derive(Clone)]
struct Scenario {
    name: String,
    initial: String,
    transient: bool,
    ops: Vec<String>,
}

/// A deterministic generator: xorshift64*.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).expect("below fits")
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }

    fn within(&mut self, low: usize, high: usize) -> usize {
        low + self.below(high - low + 1)
    }
}

/// The open options a generated scenario starts with, each with the initial
/// contents it may take.
const OPENS: &[(&str, &[&str])] = &[
    ("both", &["text3", "tail", "empty"]),
    ("read", &["text3", "tail"]),
    ("write", &["text3", "tail"]),
    ("both append", &["text3", "tail"]),
    ("write append", &["text3"]),
    ("both replace", &["text3"]),
    ("write replace", &["tail"]),
    ("both binary reclength 4", &["rec16", "text3"]),
    ("write binary reclength 4", &["rec16"]),
    ("read binary reclength 4", &["rec16"]),
    ("both binary", &["rec16", "tail"]),
    ("both binary reclength 3 replace", &["rec16"]),
];

const STRINGS: &[&str] = &["x", "yz", "abc", "QQQQQ"];

/// One generated operation. A SEEK on a READ stream never names the write
/// position, which such a stream does not have (funct.xml:4299).
fn operation(rng: &mut Rng, read_only: bool) -> String {
    let s = rng.pick(STRINGS);
    match rng.below(22) {
        0 | 1 => format!("CO {s}"),
        2 => format!("CO {s} {}", rng.within(1, 21)),
        3 => format!("CP {}", rng.within(1, 21)),
        4 | 5 => format!("LO {s}"),
        6 => format!("LO {s} {}", rng.within(1, 5)),
        7 => format!("LP {}", rng.within(1, 5)),
        8 => "CI".to_string(),
        9 => format!("CI {}", rng.within(0, 5)),
        10 => format!("CI {} {}", rng.within(0, 5), rng.within(1, 21)),
        11 | 12 => "LI".to_string(),
        13 => format!("LI {}", rng.within(1, 5)),
        14 => "CH".to_string(),
        15 => rng.pick(&["LN", "LC"]).to_string(),
        16..=18 => {
            let style = rng.pick(&["=", "<", "+", "-"]);
            let offset = match style {
                "=" => rng.within(1, 6),
                _ => rng.within(0, 3),
            };
            let which = if read_only {
                rng.pick(&["", " read"])
            } else {
                rng.pick(&["", " read", " write"])
            };
            let unit = rng.pick(&["", " char", " line"]);
            format!("S {style}{offset}{which}{unit}")
        }
        19 | 20 => {
            let which = rng.pick(&["", "read", "write"]);
            let unit = rng.pick(&["", "char", "line"]);
            format!("Q {which} {unit}").trim_end().to_string()
        }
        _ => rng.pick(&["X", "CX", "LX", "CE", "LE", "LZ"]).to_string(),
    }
}

/// `count` scenarios from `seed`. A close is followed by an open with the
/// scenario's options, REPLACE dropped, so every stream stays `NOBUFFER`.
fn generate(seed: u64, count: usize) -> Vec<Scenario> {
    let mut rng = Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1);
    (0..count)
        .map(|index| {
            let (opts, contents) = OPENS[rng.below(OPENS.len())];
            let initial = rng.pick(contents).to_string();
            let reopen = opts.replace(" replace", "");
            let mut ops = vec![format!("O {opts} nobuffer")];
            for _ in 0..rng.within(1, 9) {
                let op = operation(&mut rng, opts.starts_with("read"));
                let closes = matches!(op.as_str(), "X" | "CX" | "LX");
                ops.push(op);
                if closes {
                    ops.push(format!("O {reopen} nobuffer"));
                }
            }
            Scenario {
                name: format!("g{seed}s{index}"),
                initial,
                transient: false,
                ops,
            }
        })
        .collect()
}

/// The seeds of the generated scenarios, and how many each makes.
const SEEDS: std::ops::Range<u64> = 1..17;
const PER_SEED: usize = 80;

/// The native half of every program: it runs a scenario's operations on the
/// real stream `f` and answers in the model's shape.
const NATIVE: &str = r"
native: procedure
  use arg f, initial, transient, ops
  if \transient then do
    call stream f, 'c', 'open write replace'
    call charout f, initial
    call stream f, 'c', 'close'
  end
  out = ''
  do i = 1 to ops~items
    op = ops[i]
    r = native_op(f, op)
    out = out || op~changestr(' ', '_') || '=' || show(r) stream(f, 'S') -
      stream(f, 'c', 'query position write') stream(f, 'c', 'query position read') || ';'
  end
  call stream f, 'c', 'close'
  if transient then return out '[-]'
  size = stream(f, 'c', 'query size')
  if size = 0 then s = ''
  else s = charin(f, 1, size)
  call stream f, 'c', 'close'
  return out '[' || show(s) || ']'

native_op: procedure
  parse arg f, code a b .
  signal on syntax name native_syntax
  select
    when code == 'O' then return stream(f, 'c', 'open' subword(arg(2), 2))
    when code == 'X' then return stream(f, 'c', 'close')
    when code == 'CX' then return charout(f)
    when code == 'LX' then return lineout(f)
    when code == 'CO' & b == '' then return charout(f, a)
    when code == 'CO' then return charout(f, a, b)
    when code == 'CE' then return charout(f, '')
    when code == 'CP' then return charout(f, , a)
    when code == 'LO' & b == '' then return lineout(f, a)
    when code == 'LO' then return lineout(f, a, b)
    when code == 'LE' then return lineout(f, '')
    when code == 'LP' then return lineout(f, , a)
    when code == 'CI' & a == '' then return charin(f)
    when code == 'CI' & b == '' then return charin(f, , a)
    when code == 'CI' then return charin(f, b, a)
    when code == 'LI' & a == '' then return linein(f)
    when code == 'LI' & b == '' then return linein(f, a)
    when code == 'LI' then return linein(f, a, b)
    when code == 'LZ' then return linein(f, , 0)
    when code == 'CH' then return chars(f)
    when code == 'LN' then return lines(f)
    when code == 'LC' then return lines(f, 'C')
    when code == 'S' then return stream(f, 'c', 'seek' subword(arg(2), 2))
    when code == 'Q' then return stream(f, 'c', 'query position' subword(arg(2), 2))
  end
native_syntax:
  return 'S' || condition('O')~code
";

/// A Rexx string literal for `bytes`.
fn literal(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "''".to_string();
    }
    let hex: String = bytes.iter().map(|byte| format!("{byte:02X}")).collect();
    format!("x2c('{hex}')")
}

/// The program that runs `scenarios` in `dir`, printing `NAME native: ...`
/// and `NAME model: ...` for each.
fn program(dir: &Path, scenarios: &[Scenario]) -> String {
    let dir = dir.to_str().expect("the run directory is UTF-8");
    let mut text = String::new();
    for scenario in scenarios {
        let ops: Vec<String> = scenario.ops.iter().map(|op| format!("'{op}'")).collect();
        text.push_str(&format!(
            "call run '{}', {}, {}, .array~of({})\n",
            scenario.name,
            literal(initial(&scenario.initial)),
            u8::from(scenario.transient),
            ops.join(", ")
        ));
    }
    text.push_str("exit\n");
    text.push_str(&format!(
        "run: procedure\n  use arg name, initial, transient, ops\n  f = '{dir}/' || name || '.dat'\n  \
         say name 'native:' native(f, initial, transient, ops)\n  \
         say name 'model:' stream_model(initial, transient, ops)\n  return\n"
    ));
    text.push_str(NATIVE);
    text.push_str("::requires 'stream.rex'\n");
    text
}

/// A fresh directory holding a copy of the model, with a FIFO for each
/// transient scenario.
fn fresh_dir(label: &str, scenarios: &[Scenario]) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "stream-model-{label}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    let model = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/model/stream.rex");
    fs::copy(&model, dir.join("stream.rex"))
        .unwrap_or_else(|e| panic!("cannot copy {}: {e}", model.display()));
    for scenario in scenarios.iter().filter(|scenario| scenario.transient) {
        let fifo = dir.join(format!("{}.dat", scenario.name));
        let status = Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .expect("mkfifo runs");
        assert!(status.success(), "mkfifo {} failed", fifo.display());
    }
    fs::canonicalize(&dir).unwrap_or_else(|e| panic!("cannot resolve {}: {e}", dir.display()))
}

/// The `native` and `model` answers of each scenario in one run's stdout.
fn answers(
    side: &str,
    scenarios: &[Scenario],
    stdout: &[u8],
    stderr: &[u8],
) -> BTreeMap<String, (String, String)> {
    assert!(
        stderr.is_empty(),
        "{side} wrote stderr: {}",
        String::from_utf8_lossy(stderr)
    );
    let text = String::from_utf8_lossy(stdout);
    let mut native = BTreeMap::new();
    let mut model = BTreeMap::new();
    for line in text.lines() {
        if let Some((name, rest)) = line.split_once(" native: ") {
            native.insert(name.to_string(), rest.to_string());
        } else if let Some((name, rest)) = line.split_once(" model: ") {
            model.insert(name.to_string(), rest.to_string());
        }
    }
    scenarios
        .iter()
        .map(|scenario| {
            let name = &scenario.name;
            let get = |map: &BTreeMap<String, String>, which: &str| {
                map.get(name)
                    .unwrap_or_else(|| panic!("[{name}] {side} printed no {which} line: {text}"))
                    .clone()
            };
            (name.clone(), (get(&native, "native"), get(&model, "model")))
        })
        .collect()
}

/// Appends each scenario's answers to `<side>.tsv` under the directory
/// `REXX_STREAM_MODEL_DUMP` names, when it names one.
fn dump(side: &str, scenarios: &[Scenario], answers: &BTreeMap<String, (String, String)>) {
    let Ok(dir) = std::env::var("REXX_STREAM_MODEL_DUMP") else {
        return;
    };
    use std::io::Write;
    let path = Path::new(&dir).join(format!("{side}.tsv"));
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .unwrap_or_else(|e| panic!("cannot open {}: {e}", path.display()));
    for scenario in scenarios {
        let (native, model) = &answers[&scenario.name];
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{native}\t{model}",
            scenario.name,
            scenario.initial,
            u8::from(scenario.transient),
            scenario.ops.join(" ; ")
        )
        .expect("the dump is writable");
    }
}

/// Runs `scenarios` on this crate.
fn run_crate(label: &str, scenarios: &[Scenario]) -> BTreeMap<String, (String, String)> {
    let dir = fresh_dir(&format!("{label}-crate"), scenarios);
    let text = program(&dir, scenarios);
    let file = dir.join("case.rex");
    fs::write(&file, &text).unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let outcome = run_program(
        file.to_str().expect("the case path is UTF-8"),
        text.into_bytes(),
        Invocation::none().with_deadline(support::oracle::RUN_DEADLINE),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "[{label}] this crate's exit status, stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    let answers = answers("this crate", scenarios, &outcome.stdout, &outcome.stderr);
    dump("crate", scenarios, &answers);
    answers
}

/// Runs `scenarios` on the oracle.
fn run_oracle(label: &str, scenarios: &[Scenario]) -> BTreeMap<String, (String, String)> {
    let oracle = locate();
    let dir = fresh_dir(&format!("{label}-oracle"), scenarios);
    let file = dir.join("case.rex");
    fs::write(&file, program(&dir, scenarios))
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let cpp = oracle.run_in(&file, &dir, &[]);
    assert!(
        !did_not_finish(&cpp),
        "[{label}] the oracle did not finish: {:?}",
        cpp.termination
    );
    let answers = answers("the oracle", scenarios, &cpp.stdout, &cpp.stderr);
    dump("oracle", scenarios, &answers);
    answers
}

/// The scenario a named case describes.
fn named_case(case: &datadriven::TestCase) -> Scenario {
    assert_eq!(
        case.directive, "case",
        "unknown directive {}",
        case.directive
    );
    let arg = |key: &str, default: &str| {
        case.args
            .get(key)
            .map_or(default.to_string(), |values| values[0].clone())
    };
    let scenario = Scenario {
        name: arg("name", ""),
        initial: arg("initial", "empty"),
        transient: arg("transient", "0") == "1",
        ops: case.input.lines().map(str::to_string).collect(),
    };
    assert!(!scenario.name.is_empty(), "a case without a name");
    scenario
}

/// The items of one answer: each operation's item, then the final bytes.
fn items(answer: &str) -> (Vec<&str>, &str) {
    let (ops, content) = answer
        .rsplit_once(" [")
        .unwrap_or_else(|| panic!("an answer without its content: {answer}"));
    (
        ops.split(';').filter(|item| !item.is_empty()).collect(),
        content.trim_end_matches(']'),
    )
}

/// The bytes a rendered content stands for.
fn unshow(text: &str) -> Vec<u8> {
    text.bytes()
        .map(|byte| match byte {
            b'|' => b'\n',
            b'@' => 0,
            b'_' => b' ',
            other => other,
        })
        .collect()
}

/// ANSI X3.274 section 9.7, of one native run: the file read from position 1
/// returns, at each place a write landed, what the last write there put,
/// and every place no write reached holds what it held before the scenario
/// (NUL past the old end). A write's place is where its write position
/// stood after it, less what it wrote. A run that drove a write below
/// position 1 is outside the property and passes.
fn check_reflective(scenario: &Scenario, answer: &str) -> Result<(), String> {
    let (results, content) = items(answer);
    let finale = unshow(content);
    let mut before = initial(&scenario.initial).to_vec();
    let mut overlay = before.clone();
    let mut covered = vec![false; before.len()];
    let mut record = 0usize;
    let mut previous_write: Option<i64> = None;
    for (op, item) in scenario.ops.iter().zip(&results) {
        let (_, after) = item.split_once('=').expect("an item names its operation");
        let fields: Vec<&str> = after.split(' ').collect();
        let result = fields[0];
        let write = fields.get(2).and_then(|w| w.parse::<i64>().ok());
        let words: Vec<&str> = op.split(' ').collect();
        let data: Option<Vec<u8>> = match words[0] {
            "CO" | "LO" => Some(words[1].as_bytes().to_vec()),
            "LE" => Some(Vec::new()),
            _ => None,
        };
        if words[0] == "O" {
            let lower = op.to_lowercase();
            if lower.contains("replace") {
                before.clear();
                overlay.clear();
                covered.clear();
            }
            record = lower.split_once("reclength ").map_or(0, |(_, rest)| {
                rest.split(' ').next().unwrap_or("0").parse().unwrap_or(0)
            });
            if lower.contains("binary") && record == 0 {
                record = overlay.len();
            }
        } else if let (Some(mut bytes), "0", Some(after_write)) = (data, result, write) {
            if words[0] != "CO" {
                if record > 0 {
                    let start = match (words.get(2), previous_write) {
                        (Some(line), _) => {
                            (line.parse::<i64>().expect("a line") - 1) * record as i64 + 1
                        }
                        (None, Some(p)) => p,
                        (None, None) => return Ok(()),
                    };
                    let room = after_write - start;
                    if room < bytes.len() as i64 {
                        return Err(format!("{op} landed in less room than it wrote"));
                    }
                    bytes.resize(usize::try_from(room).expect("room"), b' ');
                } else {
                    bytes.push(b'\n');
                }
            }
            let landed = after_write - bytes.len() as i64;
            if landed < 1 {
                return Ok(());
            }
            let landed = usize::try_from(landed).expect("landed") - 1;
            let end = landed + bytes.len();
            if overlay.len() < end {
                overlay.resize(end, 0);
                covered.resize(end, false);
            }
            overlay[landed..end].copy_from_slice(&bytes);
            covered[landed..end].fill(true);
        }
        previous_write = write;
    }
    for (index, byte) in finale.iter().enumerate() {
        if covered.get(index).copied().unwrap_or(false) {
            if *byte != overlay[index] {
                return Err(format!(
                    "reading from position 1 does not return what was written at {}",
                    index + 1
                ));
            }
        } else if *byte != before.get(index).copied().unwrap_or(0) {
            return Err(format!(
                "position {} lost data no write overwrote",
                index + 1
            ));
        }
    }
    if finale.len() != overlay.len() {
        return Err("the file's length is not what the writes left".to_string());
    }
    Ok(())
}

/// The index of the first item at which two answers part.
fn first_difference_at(native: &str, model: &str) -> Option<usize> {
    let (left, left_content) = items(native);
    let (right, right_content) = items(model);
    if let Some(at) = left.iter().zip(&right).position(|(a, b)| a != b) {
        return Some(at);
    }
    if left.len() != right.len() {
        return Some(left.len().min(right.len()));
    }
    (left_content != right_content).then_some(left.len())
}

/// The item at `at`, the final content standing after the operations.
fn item_at(answer: &str, at: usize) -> String {
    let (ops, content) = items(answer);
    ops.get(at)
        .map_or_else(|| format!("[{content}]"), |item| (*item).to_string())
}

/// The [`REPORTED`] rule a departure first seen at item `at` of `scenario`
/// falls under: any item of a stream opened WRITE, and otherwise the line
/// operation at `at`.
fn reported_rule(scenario: &Scenario, at: usize) -> Option<&'static str> {
    if scenario.ops[0].starts_with("O write") {
        return Some("write-only");
    }
    let op = scenario.ops.get(at)?;
    let words: Vec<&str> = op.split(' ').collect();
    let by_line = op.contains("line");
    match words[0] {
        "Q" if by_line => Some("line-query"),
        "LC" => Some("lines-count"),
        "S" if by_line => Some("line-position"),
        "LI" | "LP" if words.len() > 1 => Some("line-position"),
        "LO" if words.len() > 2 => Some("line-position"),
        _ => None,
    }
}

/// The Deviation and rule an oracle departure first seen at item `at` of
/// `scenario` falls under, by the operations up to it: a LINEOUT that
/// closes or only positions a binary stream (Deviation 46), a buffered
/// transient stream (Deviation 48), a relative SEEK, or a SEEK by
/// characters from the end (Deviation 49).
fn deviation_rule(scenario: &Scenario, at: usize) -> Option<(u32, &'static str)> {
    let before = &scenario.ops[..=at.min(scenario.ops.len() - 1)];
    if scenario.transient && !scenario.ops[0].contains("nobuffer") {
        return Some((48, "transient-no-seek"));
    }
    let binary = scenario.ops[0].contains("binary");
    if binary && before.iter().any(|op| op == "LX" || op.starts_with("LP ")) {
        return Some((46, "pads-on-close"));
    }
    if before
        .iter()
        .any(|op| op.starts_with("S +") || op.starts_with("S -"))
    {
        return Some((49, "relative-seek"));
    }
    if before
        .iter()
        .any(|op| op.starts_with("S <") && !op.contains("line"))
    {
        return Some((49, "seek-from-end"));
    }
    None
}

/// The recorded departures of one side, `scenario\tlabel` per line, and
/// whether `REXX_STREAM_MODEL_REFRESH=1` asks for them to be rewritten.
fn recorded_departures(side: &str) -> (PathBuf, BTreeMap<String, String>, bool) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/stream_model/departures-{side}.tsv"));
    let text = fs::read_to_string(&path).unwrap_or_default();
    let rows = text
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let (name, label) = line.split_once('\t').expect("a row has two fields");
            (name.to_string(), label.to_string())
        })
        .collect();
    let refresh = std::env::var("REXX_STREAM_MODEL_REFRESH").is_ok_and(|value| value == "1");
    (path, rows, refresh)
}

/// Compares a side's departures with its recorded set, or rewrites the set.
fn compare_recorded(side: &str, header: &str, actual: &BTreeMap<String, String>) {
    let (path, recorded, refresh) = recorded_departures(side);
    if refresh {
        let mut text = header.to_string();
        for (name, label) in actual {
            text.push_str(&format!("{name}\t{label}\n"));
        }
        fs::write(&path, text).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
        return;
    }
    let mut wrong = Vec::new();
    for (name, label) in actual {
        match recorded.get(name) {
            Some(listed) if listed == label => {}
            Some(listed) => wrong.push(format!("{name}: recorded {listed}, now {label}")),
            None => wrong.push(format!("{name}: departs ({label}) but is not recorded")),
        }
    }
    for name in recorded.keys().filter(|name| !actual.contains_key(*name)) {
        wrong.push(format!("{name}: recorded, but agrees"));
    }
    assert!(
        wrong.is_empty(),
        "{side}'s departures from the model are not {}:\n{}",
        path.display(),
        wrong.join("\n")
    );
}

/// Every named case on this crate: native and model answers.
fn named_runs() -> BTreeMap<String, (Scenario, String, String)> {
    let mut runs = BTreeMap::new();
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            let scenario = named_case(case);
            let answers = run_crate(&scenario.name, std::slice::from_ref(&scenario));
            let (native, model) = answers[&scenario.name].clone();
            let mut out = format!("model: {model}\n");
            if native != model {
                out.push_str(&format!("crate: {native}\n"));
            }
            assert!(
                runs.insert(scenario.name.clone(), (scenario, native, model))
                    .is_none(),
                "two cases with one name"
            );
            out
        });
    });
    runs
}

#[test]
fn the_model_answers_the_named_cases_as_this_crate_does() {
    let runs = named_runs();
    let mut wrong = Vec::new();
    for (name, (scenario, native, model)) in &runs {
        let reported = REPORTED_CASES.iter().find(|(case, _)| case == name);
        match (native == model, reported) {
            (true, None) => {}
            (true, Some(_)) => wrong.push(format!("{name}: listed in REPORTED_CASES, but agrees")),
            (false, None) => wrong.push(format!(
                "{name}: this crate departs from the model at {}",
                item_at(
                    native,
                    first_difference_at(native, model).expect("they part")
                )
            )),
            (false, Some((_, rule))) => {
                let at = first_difference_at(native, model).expect("they part");
                if reported_rule(scenario, at) != Some(*rule) {
                    wrong.push(format!("{name}: departs at item {at}, not by {rule}"));
                }
            }
        }
        if !scenario.transient
            && let Err(e) = check_reflective(scenario, native)
        {
            wrong.push(format!("{name}: 9.7: {e}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    for (name, ..) in ORACLE_DEPARTURES {
        assert!(
            runs.contains_key(*name),
            "ORACLE_DEPARTURES names no case {name}"
        );
    }
    for rule in REPORTED {
        assert!(
            REPORTED_CASES.iter().any(|(_, listed)| listed == rule),
            "no named case shows {rule}"
        );
    }
}

#[test]
fn on_the_oracle_the_named_cases_part_only_where_listed() {
    let runs = named_runs();
    let mut wrong = Vec::new();
    for (name, (scenario, native, model)) in &runs {
        let answers = run_oracle(name, std::slice::from_ref(scenario));
        let (oracle, oracle_model) = &answers[name];
        if oracle_model != model {
            wrong.push(format!(
                "{name}: the model answers differently on the oracle"
            ));
            continue;
        }
        let reported = REPORTED_CASES.iter().any(|(case, _)| case == name);
        let listed = ORACLE_DEPARTURES.iter().find(|(case, ..)| case == name);
        match (reported, listed) {
            (true, _) if oracle != native => wrong.push(format!(
                "{name}: a REPORTED case, but the oracle and this crate part at {}",
                item_at(
                    oracle,
                    first_difference_at(oracle, native).expect("they part")
                )
            )),
            (true, _) => {}
            (false, None) if oracle != model => wrong.push(format!(
                "{name}: the oracle departs unlisted at {}",
                item_at(
                    oracle,
                    first_difference_at(oracle, model).expect("they part")
                )
            )),
            (false, None) => {}
            (false, Some((_, deviation, rule, answer))) => match first_difference_at(oracle, model)
            {
                None => wrong.push(format!("{name}: listed, but agrees")),
                Some(at) => {
                    if item_at(oracle, at) != *answer {
                        wrong.push(format!(
                            "{name}: the oracle's answer moved to {}",
                            item_at(oracle, at)
                        ));
                    }
                    if *deviation != 0
                        && deviation_rule(scenario, at).map(|(d, _)| d) != Some(*deviation)
                    {
                        wrong.push(format!("{name}: {rule} is not Deviation {deviation}'s"));
                    }
                }
            },
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The generated scenarios, all seeds.
fn generated() -> Vec<Vec<Scenario>> {
    SEEDS.map(|seed| generate(seed, PER_SEED)).collect()
}

/// This crate's answers to every generated scenario, by name, with the
/// scenarios themselves.
fn crate_runs() -> BTreeMap<String, (Scenario, String, String)> {
    let mut runs = BTreeMap::new();
    for (index, scenarios) in generated().iter().enumerate() {
        let answers = run_crate(&format!("batch{index}"), scenarios);
        for scenario in scenarios {
            let (native, model) = answers[&scenario.name].clone();
            runs.insert(scenario.name.clone(), (scenario.clone(), native, model));
        }
    }
    runs
}

#[test]
fn the_model_answers_generated_scenarios_as_this_crate_does() {
    let mut wrong = Vec::new();
    let mut departures = BTreeMap::new();
    for (name, (scenario, native, model)) in crate_runs() {
        if let Err(e) = check_reflective(&scenario, &native) {
            wrong.push(format!("{name} {:?}: 9.7: {e}", scenario.ops));
        }
        let Some(at) = first_difference_at(&native, &model) else {
            continue;
        };
        match reported_rule(&scenario, at) {
            Some(rule) => {
                departures.insert(name, format!("reported {rule}"));
            }
            None => wrong.push(format!(
                "{name} {:?}: this crate departs from the model at {} (model {})",
                scenario.ops,
                item_at(&native, at),
                item_at(&model, at)
            )),
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    compare_recorded(
        "crate",
        "# Generated scenarios where this crate departs from the model, with the\n\
         # REPORTED rule of the first item that differs. Rewritten by\n\
         # REXX_STREAM_MODEL_REFRESH=1.\n",
        &departures,
    );
}

#[test]
fn on_the_oracle_the_generated_scenarios_part_only_where_the_deviations_say() {
    let crate_side = crate_runs();
    let mut wrong = Vec::new();
    let mut departures = BTreeMap::new();
    let mut padding_seen = 0;
    for (index, scenarios) in generated().iter().enumerate() {
        let answers = run_oracle(&format!("batch{index}"), scenarios);
        for scenario in scenarios {
            let name = &scenario.name;
            let (oracle, oracle_model) = &answers[name];
            let (_, native, model) = &crate_side[name];
            if oracle_model != model {
                wrong.push(format!(
                    "{name}: the model answers differently on the oracle"
                ));
                continue;
            }
            let Some(at) = first_difference_at(oracle, model) else {
                continue;
            };
            let crate_departs = first_difference_at(native, model).is_some();
            let label = match (deviation_rule(scenario, at), reported_rule(scenario, at)) {
                (Some((deviation, rule)), _) => format!("{deviation} {rule}"),
                (None, Some(rule)) if crate_departs => format!("reported {rule}"),
                _ => {
                    wrong.push(format!(
                        "{name} {:?}: the oracle departs from the model at {} (model {})",
                        scenario.ops,
                        item_at(oracle, at),
                        item_at(model, at)
                    ));
                    continue;
                }
            };
            if label.starts_with("46") && check_reflective(scenario, oracle).is_err() {
                padding_seen += 1;
            }
            departures.insert(name.clone(), label);
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert!(
        padding_seen > 0,
        "the 9.7 check sees none of the oracle's Deviation 46 padding"
    );
    compare_recorded(
        "oracle",
        "# Generated scenarios where the oracle departs from the model, with the\n\
         # Deviation and rule, or the REPORTED rule this crate shares, of the\n\
         # first item that differs. Rewritten by REXX_STREAM_MODEL_REFRESH=1.\n",
        &departures,
    );
}

/// Each numbered Deviation row of `phase-4-exclusions.txt`, by number: the
/// row's text from its ` NN. ` heading to the next one.
fn deviation_rows(text: &str) -> BTreeMap<u32, String> {
    let mut rows = BTreeMap::new();
    let mut current: Option<(u32, String)> = None;
    for line in text.lines() {
        let heading = line
            .strip_prefix(' ')
            .and_then(|rest| rest.split_once(". "))
            .and_then(|(number, _)| number.parse::<u32>().ok());
        if let Some(number) = heading {
            if let Some((done, body)) = current.take() {
                rows.insert(done, body);
            }
            current = Some((number, String::new()));
        }
        if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((done, body)) = current {
        rows.insert(done, body);
    }
    rows
}

/// Every Deviation that licenses a departure here names this test in its
/// row, so the licence and its witness cannot drift apart.
#[test]
fn every_licensing_deviation_names_this_test() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/superpowers/plans/phase-4-exclusions.txt");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let rows = deviation_rows(&text);
    let (_, recorded, _) = recorded_departures("oracle");
    let mut licensing: BTreeSet<u32> = ORACLE_DEPARTURES
        .iter()
        .map(|(_, deviation, ..)| *deviation)
        .filter(|deviation| *deviation != 0)
        .collect();
    licensing.extend(
        recorded
            .values()
            .filter_map(|label| label.split(' ').next()?.parse::<u32>().ok()),
    );
    let unnamed: Vec<u32> = licensing
        .into_iter()
        .filter(|deviation| {
            !rows
                .get(deviation)
                .is_some_and(|row| row.contains("stream_model"))
        })
        .collect();
    assert!(
        unnamed.is_empty(),
        "these Deviations license a stream_model departure but their row does not name \
         stream_model: {unnamed:?}"
    );
}
