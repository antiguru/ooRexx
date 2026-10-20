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
//! by the final file bytes. On this crate they agree everywhere, and ANSI
//! X3.274 section 9.7's two properties hold of every native run. On the
//! oracle, with every stream opened `NOBUFFER`, each departure from the model
//! is judged by [`oracle_rule`] from the answers themselves, and the judged
//! set must be [`ORACLE_DEPARTURES`] and
//! `tests/stream_model/departures-oracle.tsv`.
//! `REXX_STREAM_MODEL_REFRESH=1` rewrites that file, and
//! `REXX_STREAM_MODEL_DUMP=<dir>` appends every answer to `<dir>/crate.tsv`
//! and `<dir>/oracle.tsv`.
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
//! * `S spec` `stream(f,'c','seek' spec)`, `P spec`
//!   `stream(f,'c','position' spec)`, `Q spec`
//!   `stream(f,'c','query position' spec)`;
//! * `CB`, in the model only: the blanks from the write position to the end
//!   of its record, the oracle's padding Deviation 46 licenses.

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

/// How long a crate run may take before the test fails rather than hangs:
/// a read of a transient stream that waits is not interrupted by the run's
/// own deadline.
const CRATE_RUN_LIMIT: std::time::Duration = std::time::Duration::from_secs(120);

/// The named cases whose native oracle answer differs from the model: the
/// case, the Deviation that licenses it (0 for a rule Phase 9d Task 6.15
/// proposes as a new Deviation), the rule, and the oracle's first item that
/// differs. [`oracle_rule`] must name the same rule from the answers.
const ORACLE_DEPARTURES: &[(&str, u32, &str, &str)] = &[
    (
        "ctrlz-append-kept",
        0,
        "ctrlz-is-data",
        "O_both_append_nobuffer=READY: READY 6 1",
    ),
    (
        "ctrlz-is-data",
        0,
        "ctrlz-is-data",
        "O_both_nobuffer=READY: READY 6 1",
    ),
    (
        "line-seek-relative-after-charin",
        0,
        "line-position",
        "S_+1_read_line=1 READY 19 1",
    ),
    (
        "linein-line-from-mid-line",
        0,
        "line-position",
        "LI_2=ne2 READY 19 13",
    ),
    ("lines-count-repeated", 0, "lines-count", "LC=2 READY 19 3"),
    (
        "read-line-after-positioning",
        0,
        "line-query",
        "Q_read_line=1 READY 19 7",
    ),
    (
        "read-line-at-end",
        0,
        "line-query",
        "Q_read_line=3 READY 19 19",
    ),
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
    ("seek-invalid", 0, "invalid-seek", "S_-9_read=0 ERROR 19 -1"),
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
    (
        "write-line-query",
        0,
        "line-query",
        "Q_write_line=3 READY 8 1",
    ),
    ("write-only-reads", 0, "write-only", "CI_1_3=n READY 19 4"),
];

/// A file's content before a scenario, by name.
fn initial(name: &str) -> &'static [u8] {
    match name {
        "text3" => b"line1\nline2\nline3\n",
        "tail" => b"ab\ncd\nef",
        "rec16" => b"AAAABBBBCCCCDDDD",
        "empty" => b"",
        "ctrlz" => b"ab\ncd\x1a",
        other => panic!("unknown initial content {other}"),
    }
}

/// One scenario: its name, its initial content's name, whether the stream is
/// a transient stream, which device stands behind it, and its operations.
/// A named case may say the oracle cannot run it, and why.
#[derive(Clone)]
struct Scenario {
    name: String,
    initial: String,
    transient: bool,
    device: Device,
    oracle_skip: Option<String>,
    ops: Vec<String>,
}

/// What a transient scenario's stream name stands for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Device {
    Fifo,
    DevNull,
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
    ("both", &["text3", "tail", "empty", "ctrlz"]),
    ("read", &["text3", "tail"]),
    ("write", &["text3", "tail"]),
    ("both append", &["text3", "tail", "ctrlz"]),
    ("write append", &["text3"]),
    ("both replace", &["text3"]),
    ("write replace", &["tail"]),
    ("both binary reclength 4", &["rec16", "text3"]),
    ("write binary reclength 4", &["rec16"]),
    ("read binary reclength 4", &["rec16"]),
    ("both binary", &["rec16", "tail"]),
    ("both binary reclength 3 replace", &["rec16"]),
    ("both binary reclength 4 append", &["rec16"]),
];

const STRINGS: &[&str] = &["x", "yz", "abc", "QQQQQ"];

/// One generated operation.
fn operation(rng: &mut Rng) -> String {
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
            let which = rng.pick(&["", " read", " write"]);
            let unit = rng.pick(&["", " char", " line"]);
            let command = rng.pick(&["S", "S", "P"]);
            format!("{command} {style}{offset}{which}{unit}")
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
                let op = operation(&mut rng);
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
                device: Device::Fifo,
                oracle_skip: None,
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
    when code == 'P' then return stream(f, 'c', 'position' subword(arg(2), 2))
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

/// A Rexx array of operations.
fn array(ops: &[String]) -> String {
    let quoted: Vec<String> = ops.iter().map(|op| format!("'{op}'")).collect();
    format!(".array~of({})", quoted.join(", "))
}

/// The program that runs `scenarios` in `dir`, printing `NAME native: ...`
/// and `NAME model: ...` for each, and `NAME variant: ...`, the model with
/// the oracle's Deviation 46 padding, where [`padding_variant`] makes one.
fn program(dir: &Path, scenarios: &[Scenario]) -> String {
    let dir = dir.to_str().expect("the run directory is UTF-8");
    let mut text = String::new();
    for scenario in scenarios {
        let variant = padding_variant(scenario).map_or(".nil".to_string(), |ops| array(&ops));
        text.push_str(&format!(
            "call run '{}', {}, {}, {}, {variant}\n",
            scenario.name,
            literal(initial(&scenario.initial)),
            u8::from(scenario.transient),
            array(&scenario.ops)
        ));
    }
    text.push_str("exit\n");
    text.push_str(&format!(
        "run: procedure\n  use arg name, initial, transient, ops, variant\n  \
         f = '{dir}/' || name || '.dat'\n  \
         say name 'native:' native(f, initial, transient, ops)\n  \
         say name 'model:' stream_model(initial, transient, ops)\n  \
         if variant \\== .nil then say name 'variant:' stream_model(initial, transient, variant)\n  \
         return\n"
    ));
    text.push_str(NATIVE);
    text.push_str("::requires 'stream.rex'\n");
    text
}

/// A fresh directory holding a copy of the model, with a FIFO, or a link to
/// `/dev/null`, for each transient scenario.
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
        let name = dir.join(format!("{}.dat", scenario.name));
        if scenario.device == Device::DevNull {
            std::os::unix::fs::symlink("/dev/null", &name)
                .unwrap_or_else(|e| panic!("cannot link {}: {e}", name.display()));
            continue;
        }
        let status = Command::new("mkfifo")
            .arg(&name)
            .status()
            .expect("mkfifo runs");
        assert!(status.success(), "mkfifo {} failed", name.display());
    }
    fs::canonicalize(&dir).unwrap_or_else(|e| panic!("cannot resolve {}: {e}", dir.display()))
}

/// One scenario's answers: native, through the model, and through the
/// padding variant where there is one.
#[derive(Clone)]
struct Answers {
    native: String,
    model: String,
    variant: Option<String>,
}

/// The answers of each scenario in one run's stdout.
fn answers(
    side: &str,
    scenarios: &[Scenario],
    stdout: &[u8],
    stderr: &[u8],
) -> BTreeMap<String, Answers> {
    assert!(
        stderr.is_empty(),
        "{side} wrote stderr: {}",
        String::from_utf8_lossy(stderr)
    );
    let text = String::from_utf8_lossy(stdout);
    let mut lines: BTreeMap<(String, &str), String> = BTreeMap::new();
    for line in text.lines() {
        for which in ["native", "model", "variant"] {
            if let Some((name, rest)) = line.split_once(&format!(" {which}: ")) {
                lines.insert((name.to_string(), which), rest.to_string());
            }
        }
    }
    scenarios
        .iter()
        .map(|scenario| {
            let name = &scenario.name;
            let get = |which: &str| {
                lines
                    .get(&(name.clone(), which))
                    .unwrap_or_else(|| panic!("[{name}] {side} printed no {which} line: {text}"))
                    .clone()
            };
            let answer = Answers {
                native: get("native"),
                model: get("model"),
                variant: lines.get(&(name.clone(), "variant")).cloned(),
            };
            (name.clone(), answer)
        })
        .collect()
}

/// Appends each scenario's answers to `<side>.tsv` under the directory
/// `REXX_STREAM_MODEL_DUMP` names, when it names one.
fn dump(side: &str, scenarios: &[Scenario], answers: &BTreeMap<String, Answers>) {
    use std::io::Write;
    let Ok(dir) = std::env::var("REXX_STREAM_MODEL_DUMP") else {
        return;
    };
    let mut text = String::new();
    for scenario in scenarios {
        let Answers { native, model, .. } = &answers[&scenario.name];
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\t{native}\t{model}\n",
            scenario.name,
            scenario.initial,
            u8::from(scenario.transient),
            scenario.ops.join(" ; ")
        ));
    }
    let path = Path::new(&dir).join(format!("{side}.tsv"));
    // One write per batch, so tests running at once do not interleave lines.
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .unwrap_or_else(|e| panic!("cannot append to {}: {e}", path.display()));
}

/// Runs `scenarios` on this crate.
fn run_crate(label: &str, scenarios: &[Scenario]) -> BTreeMap<String, Answers> {
    let dir = fresh_dir(&format!("{label}-crate"), scenarios);
    let text = program(&dir, scenarios);
    let file = dir.join("case.rex");
    fs::write(&file, &text).unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let path = file.to_str().expect("the case path is UTF-8").to_string();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let outcome = run_program(
            &path,
            text.into_bytes(),
            Invocation::none().with_deadline(support::oracle::RUN_DEADLINE),
        );
        let _ = sender.send(outcome);
    });
    let outcome = receiver.recv_timeout(CRATE_RUN_LIMIT).unwrap_or_else(|_| {
        panic!("[{label}] this crate did not finish within {CRATE_RUN_LIMIT:?}")
    });
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
fn run_oracle(label: &str, scenarios: &[Scenario]) -> BTreeMap<String, Answers> {
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
        transient: arg("transient", "0") != "0",
        device: if arg("transient", "0") == "devnull" {
            Device::DevNull
        } else {
            Device::Fifo
        },
        oracle_skip: case.args.get("oracle").map(|values| values.join(" ")),
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
            b'^' => 0x1a,
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
    let mut append = false;
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
            append = lower.contains("append");
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
                        _ if append => overlay.len() as i64 + 1,
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

/// One item's fields: the result, the state word, and the write and read
/// positions.
struct Fields<'a> {
    result: &'a str,
    state: &'a str,
    write: &'a str,
    read: &'a str,
}

fn fields(item: &str) -> Fields<'_> {
    // The operation's name may hold `=` (`S_=3`), and a rendered result
    // holds no blank, so the result starts after the last `=` before the
    // first blank.
    let head = item.split(' ').next().unwrap_or("");
    let start = head.rfind('=').map_or(0, |at| at + 1);
    let mut parts = item[start..].split(' ');
    let mut next = || parts.next().unwrap_or("");
    Fields {
        result: next(),
        state: next(),
        write: next(),
        read: next(),
    }
}

/// The variant answer's items with the `CB` padding items taken out, so they
/// line up with the scenario's own operations.
fn without_padding(answer: &str) -> String {
    let (ops, content) = items(answer);
    let kept: Vec<&str> = ops
        .into_iter()
        .filter(|item| !item.starts_with("CB="))
        .collect();
    format!("{}; [{content}]", kept.join(";"))
}

/// The rule an oracle departure at item `at` falls under, judged from the
/// observed fields of the oracle's item and the model's there: the
/// Deviation, or 0 for a rule proposed as a new one, and the rule's name.
/// Each rule checks that the operation at `at` is the one it describes and
/// that the field it describes is what differs.
fn oracle_rule(
    scenario: &Scenario,
    at: usize,
    oracle: &str,
    model: &str,
) -> Option<(u32, &'static str)> {
    let op = scenario.ops.get(at)?;
    let words: Vec<&str> = op.split(' ').collect();
    let opened = scenario.ops[..=at]
        .iter()
        .rev()
        .find(|op| op.starts_with("O "))
        .map_or("", String::as_str);
    let write_only = opened.starts_with("O write");
    let (oracle, model) = (fields(oracle), fields(model));
    let same_positions = oracle.write == model.write && oracle.read == model.read;
    let by_line = op.contains("line");
    if scenario.transient {
        let counted = scenario.ops[..at]
            .iter()
            .any(|op| matches!(op.as_str(), "CH" | "LN" | "LC"));
        return match words[0] {
            _ if !opened.contains("nobuffer") => Some((48, "transient-no-seek")),
            "CI" | "LI" if counted => Some((0, "transient-counts-read")),
            _ => None,
        };
    }
    let appending = opened.contains("append");
    let ctrlz = scenario.initial == "ctrlz";
    match words[0] {
        "O" if ctrlz
            && oracle.write.parse::<i64>().ok()
                == model.write.parse::<i64>().ok().map(|w| w - 1) =>
        {
            Some((0, "ctrlz-is-data"))
        }
        "LO" | "LE" if appending && opened.contains("binary") => Some((0, "append-record")),
        "S" | "P" if model.result == "0" && model.state == "NOTREADY" => Some((0, "invalid-seek")),
        "S" | "P" if write_only && by_line && model.result == "0" && oracle.result != "0" => {
            Some((0, "write-only"))
        }
        "S" | "P" if !by_line && (op.contains('+') || op.contains(" -")) => {
            Some((49, "relative-seek"))
        }
        "S" | "P" if op.contains('<') && !by_line => Some((49, "seek-from-end")),
        "S" | "P" if by_line => Some((0, "line-position")),
        "CI" | "LI" if write_only && (!oracle.result.is_empty() || oracle.state != model.state) => {
            Some((0, "write-only"))
        }
        "CH" | "LN" | "LC" if write_only && oracle.result != "0" => Some((0, "write-only")),
        "Q" if write_only
            && words.len() < 2
            && oracle.result == oracle.read
            && model.result == model.write =>
        {
            Some((0, "write-only"))
        }
        "LI" | "LP" if words.len() > 1 => Some((0, "line-position")),
        "LO" if words.len() > 2 => Some((0, "line-position")),
        "Q" if by_line && same_positions && oracle.state == model.state => Some((0, "line-query")),
        "LC" if same_positions
            && (oracle.result != model.result || oracle.state != model.state) =>
        {
            Some((0, "lines-count"))
        }
        _ => None,
    }
}

/// The label of an oracle answer that differs from the model's, or why it
/// is unlicensed. With a variant, the model run with the oracle's padding by
/// a LINEOUT that closes or only positions a binary stream (Deviation 46),
/// the oracle must match the variant up to a difference another rule
/// licenses.
fn oracle_label(
    scenario: &Scenario,
    oracle: &str,
    model: &str,
    variant: Option<&str>,
) -> Result<String, String> {
    let at = first_difference_at(oracle, model).expect("the answers part");
    if let Some(rule) = oracle_rule(scenario, at, &item_at(oracle, at), &item_at(model, at)) {
        return Ok(format!("{} {}", label_of(rule.0), rule.1));
    }
    if let Some(variant) = variant.map(without_padding) {
        let Some(at) = first_difference_at(oracle, &variant) else {
            return Ok("46 pads-on-close".to_string());
        };
        if let Some(rule) = oracle_rule(scenario, at, &item_at(oracle, at), &item_at(&variant, at))
        {
            return Ok(format!(
                "46 pads-on-close + {} {}",
                label_of(rule.0),
                rule.1
            ));
        }
    }
    Err(format!(
        "the oracle departs from the model at {} (model {})",
        item_at(oracle, at),
        item_at(model, at)
    ))
}

fn label_of(deviation: u32) -> String {
    if deviation == 0 {
        "new".to_string()
    } else {
        deviation.to_string()
    }
}

/// Whether a scenario's oracle answer can carry Deviation 46: a LINEOUT
/// that closes or only positions a binary stream. The variant puts the
/// oracle's padding, `CB`, before each.
fn padding_variant(scenario: &Scenario) -> Option<Vec<String>> {
    if !scenario.ops[0].contains("binary") {
        return None;
    }
    if !scenario
        .ops
        .iter()
        .any(|op| op == "LX" || op.starts_with("LP "))
    {
        return None;
    }
    let mut ops = Vec::new();
    for op in &scenario.ops {
        if op == "LX" || op.starts_with("LP ") {
            ops.push("CB".to_string());
        }
        ops.push(op.clone());
    }
    Some(ops)
}

/// Every named case on this crate.
fn named_runs() -> BTreeMap<String, (Scenario, Answers)> {
    let mut runs = BTreeMap::new();
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            let scenario = named_case(case);
            let answers = run_crate(&scenario.name, std::slice::from_ref(&scenario));
            let answer = answers[&scenario.name].clone();
            let mut out = format!("model: {}\n", answer.model);
            if answer.native != answer.model {
                out.push_str(&format!("crate: {}\n", answer.native));
            }
            assert!(
                runs.insert(scenario.name.clone(), (scenario, answer))
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
    for (name, (scenario, answer)) in &runs {
        if let Some(at) = first_difference_at(&answer.native, &answer.model) {
            wrong.push(format!(
                "{name}: this crate departs from the model at {} (model {})",
                item_at(&answer.native, at),
                item_at(&answer.model, at)
            ));
        }
        if !scenario.transient
            && let Err(e) = check_reflective(scenario, &answer.native)
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
}

#[test]
fn on_the_oracle_the_named_cases_part_only_where_listed() {
    let runs = named_runs();
    let mut wrong = Vec::new();
    for (name, (scenario, answer)) in &runs {
        if scenario.oracle_skip.is_some() {
            continue;
        }
        let answers = run_oracle(name, std::slice::from_ref(scenario));
        let oracle = &answers[name];
        if oracle.model != answer.model {
            wrong.push(format!(
                "{name}: the model answers differently on the oracle"
            ));
            continue;
        }
        let listed = ORACLE_DEPARTURES.iter().find(|(case, ..)| case == name);
        let at = first_difference_at(&oracle.native, &oracle.model);
        match (listed, at) {
            (None, None) => {}
            (None, Some(at)) => wrong.push(format!(
                "{name}: the oracle departs unlisted at {}",
                item_at(&oracle.native, at)
            )),
            (Some(_), None) => wrong.push(format!("{name}: listed, but agrees")),
            (Some((_, deviation, rule, item)), Some(at)) => {
                if item_at(&oracle.native, at) != *item {
                    wrong.push(format!(
                        "{name}: the oracle's answer moved to {}",
                        item_at(&oracle.native, at)
                    ));
                }
                let label = oracle_label(
                    scenario,
                    &oracle.native,
                    &oracle.model,
                    oracle.variant.as_deref(),
                );
                let expected = if *rule == "pads-on-close" {
                    "46 pads-on-close".to_string()
                } else {
                    format!("{} {rule}", label_of(*deviation))
                };
                if label.as_deref() != Ok(expected.as_str()) {
                    wrong.push(format!("{name}: listed as {expected}, judged {label:?}"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The generated scenarios, all seeds.
fn generated() -> Vec<Vec<Scenario>> {
    SEEDS.map(|seed| generate(seed, PER_SEED)).collect()
}

#[test]
fn the_model_answers_generated_scenarios_as_this_crate_does() {
    let mut wrong = Vec::new();
    for (index, scenarios) in generated().iter().enumerate() {
        let answers = run_crate(&format!("batch{index}"), scenarios);
        for scenario in scenarios {
            let answer = &answers[&scenario.name];
            if let Err(e) = check_reflective(scenario, &answer.native) {
                wrong.push(format!("{} {:?}: 9.7: {e}", scenario.name, scenario.ops));
            }
            if let Some(at) = first_difference_at(&answer.native, &answer.model) {
                wrong.push(format!(
                    "{} {:?}: this crate departs from the model at {} (model {})",
                    scenario.name,
                    scenario.ops,
                    item_at(&answer.native, at),
                    item_at(&answer.model, at)
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn on_the_oracle_the_generated_scenarios_part_only_where_the_deviations_say() {
    let mut wrong = Vec::new();
    let mut departures = BTreeMap::new();
    let mut padding_seen = 0;
    for (index, scenarios) in generated().iter().enumerate() {
        let answers = run_oracle(&format!("batch{index}"), scenarios);
        for scenario in scenarios {
            let name = &scenario.name;
            let oracle = &answers[name];
            if first_difference_at(&oracle.native, &oracle.model).is_none() {
                continue;
            }
            match oracle_label(
                scenario,
                &oracle.native,
                &oracle.model,
                oracle.variant.as_deref(),
            ) {
                Ok(label) => {
                    if label.starts_with("46")
                        && check_reflective(scenario, &oracle.native).is_err()
                    {
                        padding_seen += 1;
                    }
                    departures.insert(name.clone(), label);
                }
                Err(e) => wrong.push(format!("{name} {:?}: {e}", scenario.ops)),
            }
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
         # Deviation (\"new\" for one proposed) and rule oracle_label judges from\n\
         # the answers. Rewritten by REXX_STREAM_MODEL_REFRESH=1.\n",
        &departures,
    );
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
            .flat_map(|label| label.split(' '))
            .filter_map(|word| word.parse::<u32>().ok()),
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
