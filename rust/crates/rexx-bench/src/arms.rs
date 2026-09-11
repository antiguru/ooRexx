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

use std::fmt;
use std::path::{Path, PathBuf};

use crate::child::{Counted, Side, Wrapper, parse_counters, run};

/// Which engine arm of a build a run used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    Ir,
}

impl Arm {
    /// Every arm there is.
    pub const BOTH: [Arm; 1] = [Arm::Ir];

    pub fn label(self) -> &'static str {
        match self {
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
                // A bound of 0 or 1 has no distinct half, so there is no
                // second length to difference against.
                if *bound < 2 {
                    return Err(format!(
                        "{name}: its loop bound is {bound}, which has no distinct half to                          difference against"
                    ));
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
    fn rendered(&self, size: Size) -> (String, u64) {
        match self {
            Workload::Fixed { text, .. } => (text.clone(), 0),
            Workload::Scaled { text, bound, .. } => {
                let passes = match size {
                    Size::Small => bound / 2,
                    Size::Large => *bound,
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
pub struct Sitting {
    builds: Vec<Build>,
    axis: String,
    samples: Vec<Sample>,
    rounds: usize,
}

/// The order of the cells in round `round`, as indices into `cells`.
fn rotation(cells: usize, round: usize) -> Vec<usize> {
    (0..cells).map(|slot| (slot + round) % cells).collect()
}

impl Sitting {
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
pub struct ScaledSitting(pub Sitting);

impl ScaledSitting {
    /// One arm's cost per loop pass: the difference between the two lengths,
    /// divided by the difference in passes, per round, reduced.
    pub fn per_pass(&self, build: usize, arm: Arm, instrument: Instrument) -> Option<Figure> {
        let mut per_round = Vec::new();
        for round in 0..self.0.rounds {
            per_round.push(self.per_pass_in(build, arm, instrument, round)?);
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
pub const MINIMUM_ROUNDS: usize = 3;

/// Runs every cell of one sitting and answers the readings.
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
        let side = Side::rust(builds[cell.build].binary.clone(), cell.arm);
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
            assert_eq!(
                large_passes,
                *bound,
                "{}: the large length is not the program's own committed bound, so its                  whole-program ratio is not comparable with the published record",
                workload.name()
            );
            assert_eq!(small_passes, bound / 2);
            assert!(large_passes > small_passes);

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
            assert!(large.contains(&format!("n = {bound}")));
            assert!(small.contains(&format!("n = {}", bound / 2)));
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
        assert!(Workload::classify("one".into(), "n = 1\n".into()).is_err());
    }

    /// A fixed workload runs at one size, a scaled one at two.
    #[test]
    fn only_a_fixed_workload_runs_at_one_size() {
        let fixed = Workload::classify("fixed".into(), "say 1\n".into()).unwrap();
        assert_eq!(fixed.sizes(), &[Size::Small]);
        let scaled = Workload::classify("scaled".into(), "n = 5\n".into()).unwrap();
        assert_eq!(scaled.sizes(), &[Size::Small, Size::Large]);
    }
    /// No cell keeps its slot across the rounds.
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
