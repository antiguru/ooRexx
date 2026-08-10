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

//! Comparing the two engine arms of one build, on both instruments, at two
//! problem sizes, inside one sitting.
//!
//! # Why this is a tool and not a paragraph
//!
//! Every rule below was already written down in this phase's plan when a task
//! broke it. A rule stated in prose is re-derived by each task that reads it;
//! a rule stated in a type is not available to be broken. The four defects
//! this module exists to make unexpressible, each of which has shipped or
//! nearly shipped here:
//!
//! * **A claim on one instrument.** [`Reading`] carries `instructions:u` and
//!   `cycles:u` together and has no constructor that takes one of them. The
//!   `perf stat` reply is refused whole when either event is missing, and the
//!   events asked for are the events parsed (`child::Counted::events`).
//! * **A comparison assembled from two sittings.** [`measure`] takes the
//!   builds and the workload and runs every cell itself. There is no entry
//!   point that takes two result sets, and [`Sitting`] cannot be built from
//!   outside this module.
//! * **A cross-binary arm comparison.** [`Sitting::arm_ratio`] takes one
//!   build's index and reads that build's own two arms. A ratio between two
//!   binaries is [`Sitting::across_builds`], which is a different method with
//!   a different name, because Task 4c built the same source twice differing
//!   by one comment and read 7.8% between them on a `.text` with the same
//!   sha256.
//! * **A bare number.** [`Figure`]'s fields are private and its only
//!   renderings -- [`Figure::fmt`] and [`Figure::columns`] -- carry the
//!   spread and the round count beside the median.
//!
//! # What two sizes buy, and the one workload that has none
//!
//! Running the same body at `n` and `2n` separates the per-pass cost from the
//! fixed cost exactly, because the fixed part cancels in the difference. That
//! is the question every promotion task in this phase asks, and it is what
//! settled "per promoted clause, zero per body entry".
//!
//! A program's size is **derived from the program**, not declared beside it:
//! [`Workload::classify`] reads the single `n = <integer>` line the benchmark
//! programs carry. A program with no such line has no size to vary, and gets
//! [`Workload::Fixed`] -- a variant with no per-pass reduction on it at all,
//! so a fixed workload cannot produce a per-pass figure rather than producing
//! a misleading one. A program with more than one such line is refused,
//! because which one is the loop bound is then a guess.
//!
//! # Scope
//!
//! In-phase, arm against arm. Phase 4f compares this binary against the C++
//! oracle on wall clock, which is a different comparison against a different
//! reference; `child::Side` already carries the oracle and this module
//! deliberately does not reach for it.

use std::fmt;
use std::path::{Path, PathBuf};

use crate::child::{Counted, Side, Wrapper, parse_counters, run};

/// Which engine arm of a build a run used.
///
/// **Both arms come from one binary**, selected through `REXX_ENGINE`, which
/// is why this is a value the harness sets per run rather than a second
/// binary it launches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    TreeWalker,
    Ir,
}

impl Arm {
    /// Both arms, in the order the ratio is stated: the denominator first.
    pub const BOTH: [Arm; 2] = [Arm::TreeWalker, Arm::Ir];

    pub fn label(self) -> &'static str {
        match self {
            Arm::TreeWalker => "tw",
            Arm::Ir => "ir",
        }
    }

    /// The `REXX_ENGINE` value that selects this arm. `rexx-run` rejects a
    /// value it does not recognise rather than defaulting, so a typo here is
    /// a failed run rather than an arm silently measured against itself.
    fn engine(self) -> &'static str {
        match self {
            Arm::TreeWalker => "tree-walker",
            Arm::Ir => "ir",
        }
    }
}

/// One of the two instruments every claim in this phase is stated on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instrument {
    Instructions,
    Cycles,
}

impl Instrument {
    pub const BOTH: [Instrument; 2] = [Instrument::Instructions, Instrument::Cycles];

    pub fn label(self) -> &'static str {
        match self {
            Instrument::Instructions => "instructions:u",
            Instrument::Cycles => "cycles:u",
        }
    }
}

/// What one run of one cell cost, on both instruments.
///
/// There is no constructor taking a single count. The only way to make one is
/// from a `perf stat` reply that carried both events, which is what makes
/// "reported on instructions alone" a thing this harness cannot do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
    instructions: u64,
    cycles: u64,
}

impl Reading {
    pub fn count(self, instrument: Instrument) -> u64 {
        match instrument {
            Instrument::Instructions => self.instructions,
            Instrument::Cycles => self.cycles,
        }
    }

    fn on(self, instrument: Instrument) -> f64 {
        match instrument {
            Instrument::Instructions => self.instructions as f64,
            Instrument::Cycles => self.cycles as f64,
        }
    }
}

/// Which of a scaled workload's two lengths a run used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Small,
    Large,
}

impl Size {
    pub fn label(self) -> &'static str {
        match self {
            Size::Small => "small",
            Size::Large => "large",
        }
    }
}

/// One build under test: a binary, and the label its rows carry.
///
/// **Both arms of a comparison come from one of these.** Several builds may
/// be measured in one sitting -- that is how a base and a head are compared
/// without assembling the comparison from two runs -- and the rotation covers
/// them, so a build never keeps a slot either.
#[derive(Clone, Debug)]
pub struct Build {
    pub label: String,
    pub binary: PathBuf,
}

impl Build {
    /// `LABEL=PATH`, with the path canonicalised so a row names the binary
    /// that ran rather than the spelling that reached the command line.
    pub fn parse(spec: &str) -> Result<Build, String> {
        let (label, path) = spec
            .split_once('=')
            .ok_or_else(|| format!("`{spec}` is not LABEL=PATH"))?;
        if label.is_empty() {
            return Err(format!("`{spec}` has an empty label"));
        }
        let binary = Path::new(path)
            .canonicalize()
            .map_err(|error| format!("{path} does not resolve: {error}"))?;
        Ok(Build {
            label: label.to_string(),
            binary,
        })
    }
}

/// A benchmark program, classified from its own text.
///
/// The classification is derived rather than declared, in both directions:
/// `the_benchmark_programs_all_classify` runs it over every program in
/// `bench-programs/`, so a program whose shape this cannot read is a red test
/// rather than a silently mis-sized measurement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Workload {
    /// A program with exactly one `n = <integer>` line, which is its loop
    /// bound. Measured at that bound and at twice it.
    Scaled {
        name: String,
        text: String,
        bound: u64,
    },
    /// A program with no `n = <integer>` line, so there is no size to vary.
    ///
    /// **This is the one exception to "two sizes or it refuses", and it is
    /// narrow by construction rather than by intent.** The rule exists so a
    /// whole-program figure is never quoted as a per-pass one; a `Fixed`
    /// workload produces no per-pass figure at all, because
    /// [`Sitting::per_pass`] lives on [`Scaled`] and this variant does not
    /// reach it. `bench-programs/startup.rex` is a program of this shape:
    /// `say 1`, whose whole content is the fixed cost the other axes cancel.
    ///
    /// [`Scaled`]: Measured::Scaled
    Fixed { name: String, text: String },
}

impl Workload {
    /// Reads `path` and classifies it.
    pub fn read(path: &Path) -> Result<Workload, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        Workload::classify(name, text)
    }

    /// The classification itself, separated from the read so it can be
    /// asserted on text rather than on files.
    pub fn classify(name: String, text: String) -> Result<Workload, String> {
        let bounds: Vec<u64> = text
            .lines()
            .filter_map(|line| line.trim().strip_prefix("n = "))
            .filter_map(|rest| rest.trim().parse::<u64>().ok())
            .collect();
        match bounds.as_slice() {
            [] => Ok(Workload::Fixed { name, text }),
            [bound] => {
                if *bound == 0 {
                    return Err(format!("{name}: its loop bound is 0, so it has no passes"));
                }
                Ok(Workload::Scaled {
                    name,
                    text,
                    bound: *bound,
                })
            }
            _ => Err(format!(
                "{name}: {} lines look like a loop bound, so which one scales the work is a \
                 guess rather than a reading",
                bounds.len()
            )),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Workload::Scaled { name, .. } | Workload::Fixed { name, .. } => name,
        }
    }

    /// The program text to run at `size`, and the number of passes it makes.
    ///
    /// The large length is exactly twice the small one, so the difference
    /// between the two readings is the cost of `bound` further passes and the
    /// fixed part cancels without an intercept having to be estimated.
    fn rendered(&self, size: Size) -> (String, u64) {
        match self {
            Workload::Fixed { text, .. } => (text.clone(), 0),
            Workload::Scaled { text, bound, .. } => {
                let passes = match size {
                    Size::Small => *bound,
                    Size::Large => bound * 2,
                };
                let rewritten = text
                    .lines()
                    .map(|line| {
                        if line
                            .trim()
                            .strip_prefix("n = ")
                            .is_some_and(|rest| rest.trim().parse::<u64>().is_ok())
                        {
                            format!("n = {passes}")
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                (rewritten + "\n", passes)
            }
        }
    }

    /// The sizes this workload is run at.
    fn sizes(&self) -> &'static [Size] {
        match self {
            Workload::Fixed { .. } => &[Size::Small],
            Workload::Scaled { .. } => &[Size::Small, Size::Large],
        }
    }
}

/// A measured quantity together with the spread of the rounds behind it.
///
/// **Private fields and no median accessor.** The two renderings this type
/// has both carry the spread, so a figure cannot be quoted into a brief or a
/// report stripped of the noise it was measured against -- which is how a
/// ratio taken once has twice been carried forward here as though it were
/// exact.
#[derive(Clone, Copy, Debug)]
pub struct Figure {
    median: f64,
    low: f64,
    high: f64,
    rounds: usize,
}

impl Figure {
    /// Reduces one round-per-element sample. `None` for an empty sample, so a
    /// cell that produced nothing reports that rather than a zero.
    fn of(samples: &mut [f64]) -> Option<Figure> {
        if samples.is_empty() {
            return None;
        }
        samples.sort_by(f64::total_cmp);
        Some(Figure {
            median: samples[samples.len() / 2],
            low: samples[0],
            high: samples[samples.len() - 1],
            rounds: samples.len(),
        })
    }

    /// The four columns a machine-readable row carries for this figure, in
    /// order. A fixed-length array so a column added to the header and not
    /// here does not compile.
    pub fn columns(&self) -> [String; 4] {
        [
            format!("{:.6}", self.median),
            format!("{:.6}", self.low),
            format!("{:.6}", self.high),
            self.rounds.to_string(),
        ]
    }

    /// The names of [`Figure::columns`], for a header row.
    pub fn column_names(prefix: &str) -> [String; 4] {
        ["median", "min", "max", "rounds"].map(|part| format!("{prefix}_{part}"))
    }

    /// The half-width of the sample as a fraction of the median, which is the
    /// figure a report quotes as `+/- x%`. `None` when the median is zero.
    pub fn spread_fraction(&self) -> Option<f64> {
        if self.median == 0.0 {
            return None;
        }
        Some(((self.high - self.median).max(self.median - self.low)) / self.median.abs())
    }
}

impl fmt::Display for Figure {
    /// Median, then the range, then how many rounds are behind it. Never the
    /// median alone.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:.5} [{:.5}..{:.5}] k={}",
            self.median, self.low, self.high, self.rounds
        )
    }
}

/// One cell of the sitting: which build, which arm, which size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cell {
    build: usize,
    arm: Arm,
    size: Size,
}

/// One reading of one cell in one round.
#[derive(Clone, Copy, Debug)]
struct Sample {
    cell: Cell,
    round: usize,
    reading: Reading,
    passes: u64,
}

/// Every reading of one sitting over one workload.
///
/// **Only [`measure`] can build one**, which is what makes "assembled from
/// two runs" unexpressible: there is no `Sitting::from_rows`, and the rotation
/// is applied inside `measure` rather than by whoever calls it.
pub struct Sitting {
    builds: Vec<Build>,
    axis: String,
    samples: Vec<Sample>,
    rounds: usize,
}

/// The order of the cells in round `round`, as indices into `cells`.
///
/// A left rotation by the round number. Over `k` rounds with `c` cells and
/// `k <= c` every cell occupies `k` distinct slots, so no cell keeps the slot
/// that carries whatever the previous run left in the caches --
/// `no_cell_keeps_a_slot` asserts exactly that rather than describing it.
fn rotation(cells: usize, round: usize) -> Vec<usize> {
    (0..cells).map(|slot| (slot + round) % cells).collect()
}

impl Sitting {
    /// The ratio of this build's IR arm to its own tree-walker arm, per round,
    /// reduced.
    ///
    /// **One build's index, and the two arms are read out of it.** The pairing
    /// is per round, so a round in which the machine was slower moves both
    /// arms and cancels rather than moving the ratio.
    pub fn arm_ratio(&self, build: usize, size: Size, instrument: Instrument) -> Option<Figure> {
        let mut per_round = Vec::new();
        for round in 0..self.rounds {
            let tw = self.find(build, Arm::TreeWalker, size, round)?;
            let ir = self.find(build, Arm::Ir, size, round)?;
            per_round.push(ir.reading.on(instrument) / tw.reading.on(instrument));
        }
        Figure::of(&mut per_round)
    }

    /// One arm's absolute reading, per round, reduced.
    pub fn absolute(
        &self,
        build: usize,
        arm: Arm,
        size: Size,
        instrument: Instrument,
    ) -> Option<Figure> {
        let mut per_round = Vec::new();
        for round in 0..self.rounds {
            per_round.push(self.find(build, arm, size, round)?.reading.on(instrument));
        }
        Figure::of(&mut per_round)
    }

    /// The same arm of two builds, as a ratio.
    ///
    /// **Named apart from [`Sitting::arm_ratio`] on purpose.** This is a
    /// comparison between two binaries, which Task 4c measured at 7.8% between
    /// two builds of the same source, and nothing here makes it more reliable
    /// than that -- interleaving removes the machine's drift from it, not the
    /// code placement's. A caller that wants the phase's headline ratio wants
    /// the other method.
    pub fn across_builds(
        &self,
        from: usize,
        to: usize,
        arm: Arm,
        size: Size,
        instrument: Instrument,
    ) -> Option<Figure> {
        let mut per_round = Vec::new();
        for round in 0..self.rounds {
            let a = self.find(from, arm, size, round)?;
            let b = self.find(to, arm, size, round)?;
            per_round.push(b.reading.on(instrument) / a.reading.on(instrument));
        }
        Figure::of(&mut per_round)
    }

    pub fn axis(&self) -> &str {
        &self.axis
    }

    pub fn builds(&self) -> &[Build] {
        &self.builds
    }

    pub fn rounds(&self) -> usize {
        self.rounds
    }

    /// Every individual reading, in the order it was taken.
    ///
    /// **Emitted so that an excursion can be looked at rather than inferred
    /// from a median's max.** One round of one cell here read 2.85% high on an
    /// instruction count that is otherwise stable to eight significant
    /// figures; the median absorbed it, which is the reduction working, and
    /// nothing in the reduced output said which run it was.
    pub fn runs(&self) -> impl Iterator<Item = (usize, &str, Arm, Size, Reading)> {
        self.samples.iter().map(|sample| {
            (
                sample.round,
                self.builds[sample.cell.build].label.as_str(),
                sample.cell.arm,
                sample.cell.size,
                sample.reading,
            )
        })
    }

    fn find(&self, build: usize, arm: Arm, size: Size, round: usize) -> Option<&Sample> {
        self.samples.iter().find(|sample| {
            sample.round == round
                && sample.cell.build == build
                && sample.cell.arm == arm
                && sample.cell.size == size
        })
    }
}

/// A sitting over a workload whose size can be varied.
///
/// The wrapper exists so that [`ScaledSitting::per_pass`] is reachable only
/// for a workload that has two sizes. A fixed workload's sitting is a plain
/// [`Sitting`] and there is no per-pass method on it to call.
pub struct ScaledSitting(pub Sitting);

impl ScaledSitting {
    /// One arm's cost per loop pass: the difference between the two lengths,
    /// divided by the difference in passes, per round, reduced.
    ///
    /// The fixed cost -- process start, source load, everything before the
    /// loop -- is identical in the two lengths and cancels exactly. This is
    /// the quantity a per-clause claim is made of.
    pub fn per_pass(&self, build: usize, arm: Arm, instrument: Instrument) -> Option<Figure> {
        let mut per_round = Vec::new();
        for round in 0..self.0.rounds {
            per_round.push(self.per_pass_in(build, arm, instrument, round)?);
        }
        Figure::of(&mut per_round)
    }

    /// The IR arm's per-pass cost less the tree-walker arm's, per round.
    ///
    /// The quantity every per-clause figure in this phase is stated in, and
    /// the reason it is a method rather than a subtraction at the call site:
    /// subtracting two medians is not the median of the differences, and the
    /// spread of the difference is the one a per-clause claim needs.
    pub fn per_pass_gap(&self, build: usize, instrument: Instrument) -> Option<Figure> {
        let mut per_round = Vec::new();
        for round in 0..self.0.rounds {
            let tw = self.per_pass_in(build, Arm::TreeWalker, instrument, round)?;
            let ir = self.per_pass_in(build, Arm::Ir, instrument, round)?;
            per_round.push(ir - tw);
        }
        Figure::of(&mut per_round)
    }

    /// One arm's cost that does not scale with the loop: the small reading
    /// less the per-pass cost times the small length.
    pub fn fixed(&self, build: usize, arm: Arm, instrument: Instrument) -> Option<Figure> {
        let mut per_round = Vec::new();
        for round in 0..self.0.rounds {
            let small = self.0.find(build, arm, Size::Small, round)?;
            let per_pass = self.per_pass_in(build, arm, instrument, round)?;
            per_round.push(small.reading.on(instrument) - per_pass * small.passes as f64);
        }
        Figure::of(&mut per_round)
    }

    fn per_pass_in(
        &self,
        build: usize,
        arm: Arm,
        instrument: Instrument,
        round: usize,
    ) -> Option<f64> {
        let small = self.0.find(build, arm, Size::Small, round)?;
        let large = self.0.find(build, arm, Size::Large, round)?;
        let passes = large.passes.checked_sub(small.passes)?;
        if passes == 0 {
            return None;
        }
        Some((large.reading.on(instrument) - small.reading.on(instrument)) / passes as f64)
    }
}

/// What [`measure`] answers, split by whether a per-pass figure exists at all.
pub enum Measured {
    Scaled(ScaledSitting),
    Fixed(Sitting),
}

impl Measured {
    pub fn sitting(&self) -> &Sitting {
        match self {
            Measured::Scaled(scaled) => &scaled.0,
            Measured::Fixed(sitting) => sitting,
        }
    }
}

/// How many rounds a figure needs before it is one.
///
/// Three is the smallest sample with a median that is not also an endpoint, so
/// a spread computed from fewer is the sample itself rather than a spread.
pub const MINIMUM_ROUNDS: usize = 3;

/// Runs every cell of one sitting and answers the readings.
///
/// **This function owns the rotation, the interleaving and the warm-up**, and
/// that is the whole reason it takes the builds and the workload rather than
/// taking results. A caller cannot run the arms in two passes, cannot run the
/// builds on two occasions, and cannot choose an order that leaves one arm
/// always first.
///
/// One untimed pass over every cell precedes the rounds, so the file cache and
/// the branch predictors are not part of whichever cell happens to be first.
pub fn measure(
    builds: &[Build],
    workload: &Workload,
    rounds: usize,
    workdir: &Path,
    pin: Option<String>,
    mut progress: impl FnMut(&str),
) -> Result<Measured, String> {
    if builds.is_empty() {
        return Err("no build to measure".to_string());
    }
    if rounds < MINIMUM_ROUNDS {
        return Err(format!(
            "{rounds} rounds is not a median with a spread; {MINIMUM_ROUNDS} is the least this \
             harness reports"
        ));
    }
    std::fs::create_dir_all(workdir).map_err(|error| format!("{}: {error}", workdir.display()))?;

    // The rendered programs, written once and reused by every cell, so that
    // two arms of one round run byte-identical source.
    let mut programs = Vec::new();
    for size in workload.sizes() {
        let (text, passes) = workload.rendered(*size);
        let path = workdir.join(format!("{}-{}.rex", workload.name(), size.label()));
        std::fs::write(&path, &text).map_err(|error| format!("{}: {error}", path.display()))?;
        programs.push((*size, path, passes));
    }

    let mut cells = Vec::new();
    for build in 0..builds.len() {
        for arm in Arm::BOTH {
            for (size, _, _) in &programs {
                cells.push(Cell {
                    build,
                    arm,
                    size: *size,
                });
            }
        }
    }

    let wrapper = Wrapper {
        pin,
        counters: Counted::User,
    };
    let events = wrapper
        .counters
        .events()
        .expect("Counted::User counts something");

    let run_cell = |cell: &Cell| -> Result<(Reading, u64), String> {
        let (_, path, passes) = programs
            .iter()
            .find(|(size, _, _)| *size == cell.size)
            .expect("every cell names a rendered size");
        let side = Side {
            label: "rust",
            binary: builds[cell.build].binary.clone(),
            env: vec![("REXX_ENGINE".to_string(), cell.arm.engine().to_string())],
        };
        let completed = run(&side, path, workdir, &wrapper)?;
        if !completed.succeeded() {
            return Err(format!(
                "{} {} {} exited {:?}: {}",
                builds[cell.build].label,
                cell.arm.label(),
                cell.size.label(),
                completed.exit_code,
                String::from_utf8_lossy(&completed.stderr).trim()
            ));
        }
        let counters = parse_counters(&completed.stderr, events).ok_or_else(|| {
            format!(
                "`perf stat` produced no {} and {} pair for {} {} {}",
                events[0],
                events[1],
                builds[cell.build].label,
                cell.arm.label(),
                cell.size.label()
            )
        })?;
        Ok((
            Reading {
                instructions: counters.instructions,
                cycles: counters.cycles,
            },
            *passes,
        ))
    };

    progress(&format!(
        "{}: warming {} cells",
        workload.name(),
        cells.len()
    ));
    for cell in &cells {
        run_cell(cell)?;
    }

    let mut samples = Vec::new();
    for round in 0..rounds {
        progress(&format!(
            "{}: round {}/{rounds}",
            workload.name(),
            round + 1
        ));
        for slot in rotation(cells.len(), round) {
            let cell = cells[slot];
            let (reading, passes) = run_cell(&cell)?;
            samples.push(Sample {
                cell,
                round,
                reading,
                passes,
            });
        }
    }

    let sitting = Sitting {
        builds: builds.to_vec(),
        axis: workload.name().to_string(),
        samples,
        rounds,
    };
    Ok(match workload {
        Workload::Scaled { .. } => Measured::Scaled(ScaledSitting(sitting)),
        Workload::Fixed { .. } => Measured::Fixed(sitting),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every program in `bench-programs/` classifies, and a scaled one really
    /// does differ from its own source in nothing but the bound.
    ///
    /// Both halves are asserted because either can fail silently. A program
    /// this cannot classify would be refused at run time with nothing on the
    /// record saying the harness had stopped covering it, and a rewrite that
    /// touched a second line would be measuring a different program at the
    /// large size than at the small one -- which reads exactly like a
    /// per-pass cost.
    #[test]
    fn the_benchmark_programs_all_classify() {
        let dir = crate::programs_dir();
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
        let mut seen = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "rex") {
                continue;
            }
            seen += 1;
            let workload = Workload::read(&path).unwrap_or_else(|e| panic!("{e}"));
            let Workload::Scaled { text, bound, .. } = &workload else {
                continue;
            };
            let (small, small_passes) = workload.rendered(Size::Small);
            let (large, large_passes) = workload.rendered(Size::Large);
            assert_eq!(small_passes, *bound);
            assert_eq!(large_passes, bound * 2);

            let without_bound = |source: &str| -> Vec<String> {
                source
                    .lines()
                    .filter(|line| !line.trim().starts_with("n = "))
                    .map(str::to_string)
                    .collect()
            };
            assert_eq!(
                without_bound(text),
                without_bound(&small),
                "{}: rewriting the bound changed another line",
                workload.name()
            );
            assert_eq!(
                without_bound(&small),
                without_bound(&large),
                "{}: the two lengths differ somewhere other than the bound",
                workload.name()
            );
            assert!(small.contains(&format!("n = {bound}")));
            assert!(large.contains(&format!("n = {}", bound * 2)));
        }
        assert!(seen > 0, "no benchmark program was classified");
    }

    /// A program with two lines that look like a loop bound is refused, and a
    /// program with none is fixed rather than guessed at.
    #[test]
    fn an_ambiguous_bound_is_refused_and_a_missing_one_is_not_invented() {
        assert!(matches!(
            Workload::classify("fixed".into(), "say 1\n".into()),
            Ok(Workload::Fixed { .. })
        ));
        let two = Workload::classify("two".into(), "n = 5\nsay 1\nn = 7\n".into());
        assert!(two.is_err(), "{two:?}");
        assert!(Workload::classify("zero".into(), "n = 0\n".into()).is_err());
    }

    /// A fixed workload runs at one size, a scaled one at two.
    ///
    /// This is the only place the harness runs a single size, and the type
    /// that permits it is the one with no per-pass reduction on it -- which is
    /// what stops the exception from being a way back to a single-size claim.
    #[test]
    fn only_a_fixed_workload_runs_at_one_size() {
        let fixed = Workload::classify("fixed".into(), "say 1\n".into()).unwrap();
        assert_eq!(fixed.sizes(), &[Size::Small]);
        let scaled = Workload::classify("scaled".into(), "n = 5\n".into()).unwrap();
        assert_eq!(scaled.sizes(), &[Size::Small, Size::Large]);
    }

    /// No cell keeps its slot across the rounds.
    ///
    /// Asserted on the permutation rather than trusted to the expression: an
    /// implementation that returned `0..cells` unchanged would still be "a
    /// rotation" by the loosest reading, and would put the same arm first in
    /// every round -- which is the ordering artifact the rule exists to
    /// remove.
    #[test]
    fn no_cell_keeps_a_slot() {
        for cells in 2..=12usize {
            for rounds in 2..=cells {
                for cell in 0..cells {
                    let slots: Vec<usize> = (0..rounds)
                        .map(|round| {
                            rotation(cells, round)
                                .iter()
                                .position(|which| *which == cell)
                                .expect("every cell appears in every round")
                        })
                        .collect();
                    let mut distinct = slots.clone();
                    distinct.sort_unstable();
                    distinct.dedup();
                    assert_eq!(
                        distinct.len(),
                        rounds,
                        "cell {cell} of {cells} repeated a slot over {rounds} rounds: {slots:?}"
                    );
                }
            }
        }
    }

    /// Every round holds every cell exactly once.
    #[test]
    fn a_round_is_a_permutation() {
        for cells in 1..=12usize {
            for round in 0..20 {
                let mut order = rotation(cells, round);
                assert_eq!(order.len(), cells);
                order.sort_unstable();
                assert_eq!(order, (0..cells).collect::<Vec<_>>());
            }
        }
    }

    /// A figure prints its spread and its round count, never the median
    /// alone.
    ///
    /// The degenerate implementation this rules out is a `Display` that
    /// forwards to the median's own: it would satisfy "a figure can be
    /// printed" and would put a bare number into every table.
    #[test]
    fn a_figure_carries_its_own_noise() {
        let figure = Figure::of(&mut [1.0, 1.5, 1.2]).expect("three samples");
        let printed = figure.to_string();
        assert!(printed.contains("1.20000"), "{printed}");
        assert!(printed.contains("1.00000"), "{printed}");
        assert!(printed.contains("1.50000"), "{printed}");
        assert!(printed.contains("k=3"), "{printed}");
        assert_eq!(figure.columns().len(), 4);
        assert_eq!(figure.columns()[3], "3");
        assert!(Figure::of(&mut []).is_none());
    }

    /// Fewer than three rounds is refused rather than reported.
    #[test]
    fn too_few_rounds_is_not_a_measurement() {
        let workload = Workload::classify("scaled".into(), "n = 5\n".into()).unwrap();
        let build = Build {
            label: "b".into(),
            binary: PathBuf::from("/bin/true"),
        };
        let refused = measure(
            std::slice::from_ref(&build),
            &workload,
            MINIMUM_ROUNDS - 1,
            &std::env::temp_dir(),
            None,
            |_| {},
        );
        assert!(refused.is_err(), "two rounds were accepted");
        let no_build = measure(
            &[],
            &workload,
            MINIMUM_ROUNDS,
            &std::env::temp_dir(),
            None,
            |_| {},
        );
        assert!(no_build.is_err(), "a sitting with no build was accepted");
    }
}
