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

//! The committed row sets gate table C reads, from `corpus/docs/`.

use std::fs;

use rexx_extract::docs::classes::NO_PROGRAM;

use crate::corpus_dir;

/// Reads one committed row file as tab-separated fields.
fn read_table(name: &str, fields: usize) -> Vec<Vec<String>> {
    let path = corpus_dir().join("docs").join(name);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let row: Vec<String> = line.split('\t').map(str::to_string).collect();
        assert_eq!(
            row.len(),
            fields,
            "{}: row {line:?} has the wrong number of fields",
            path.display()
        );
        rows.push(row);
    }
    assert!(
        !rows.is_empty(),
        "{} named no rows -- that is a defect in the row file, not an empty pass",
        path.display()
    );
    rows
}

/// One `provide.xml` section, from `provide-sections.txt`.
pub(crate) struct Section {
    pub(crate) id: String,
    pub(crate) depth: String,
    pub(crate) line: String,
    pub(crate) parent: String,
    pub(crate) title: String,
}

pub(crate) fn read_sections() -> Vec<Section> {
    read_table("provide-sections.txt", 5)
        .into_iter()
        .map(|row| Section {
            id: row[0].clone(),
            depth: row[1].clone(),
            line: row[2].clone(),
            parent: row[3].clone(),
            title: row[4].clone(),
        })
        .collect()
}

/// What a method probe's instance arm binds `o` to.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Construction {
    /// The class is `covered`, and this is the committed expression the
    /// instance arm constructs with, together with the directive the probe
    /// carries below its readbacks for an expression that reads one.
    Constructs {
        program: String,
        directive: Option<String>,
    },
    /// The class is not `covered`, so the instance arm asks a bare `~new`
    /// whose raise is the row's evidence.
    Raises,
}

/// One documented class, from `class-set.txt`.
pub(crate) struct ClassRow {
    pub(crate) name: String,
    pub(crate) entry: String,
    pub(crate) cite: String,
    pub(crate) status: String,
    pub(crate) reason: String,
    pub(crate) construction: Construction,
    /// The row set's `method-owner` column: the phase that owes this class's
    /// method rows an `agree`.
    pub(crate) owner: String,
}

pub(crate) fn read_classes() -> Vec<ClassRow> {
    read_table("class-set.txt", 9)
        .into_iter()
        .map(|row| ClassRow {
            name: row[0].clone(),
            entry: row[1].clone(),
            cite: row[3].clone(),
            status: row[4].clone(),
            reason: row[5].clone(),
            construction: construction_of(&row[0], &row[4], &row[6], &row[7]),
            owner: row[8].clone(),
        })
        .collect()
}

/// The fields `covered` is spread across, read back as one value.
fn construction_of(name: &str, status: &str, program: &str, directive: &str) -> Construction {
    let directive = match directive {
        NO_PROGRAM => None,
        directive => Some(directive.to_string()),
    };
    match (status, program) {
        ("covered", NO_PROGRAM) => panic!(
            "class-set.txt records {name} as `covered` and carries no construction program \
             for it. `covered` is exactly the claim that one is committed"
        ),
        ("covered", program) => Construction::Constructs {
            program: program.to_string(),
            directive,
        },
        (_, NO_PROGRAM) => {
            assert!(
                directive.is_none(),
                "class-set.txt records {name} as `{status}` and carries a construction \
                 directive for it. Only a `covered` row has a route to an instance"
            );
            Construction::Raises
        }
        (status, program) => panic!(
            "class-set.txt records {name} as `{status}` and carries the construction program \
             {program:?} for it. Only a `covered` row has a route to an instance"
        ),
    }
}

/// One documented hierarchy edge, from `hierarchy-edges.txt`.
pub(crate) struct Edge {
    pub(crate) child: String,
    pub(crate) parent: String,
    pub(crate) line: String,
}

pub(crate) fn read_edges() -> Vec<Edge> {
    read_table("hierarchy-edges.txt", 4)
        .into_iter()
        .map(|row| Edge {
            child: row[0].clone(),
            parent: row[1].clone(),
            line: row[3].clone(),
        })
        .collect()
}

/// One documented (class, method, arm), from `class-methods.txt`.
pub(crate) struct MethodRow {
    pub(crate) class: String,
    pub(crate) method: String,
    pub(crate) arm: String,
    pub(crate) status: String,
    pub(crate) origin: String,
}

pub(crate) fn read_method_rows() -> Vec<MethodRow> {
    read_table("class-methods.txt", 7)
        .into_iter()
        .map(|row| MethodRow {
            class: row[0].clone(),
            method: row[1].clone(),
            arm: row[2].clone(),
            status: row[3].clone(),
            origin: row[5].clone(),
        })
        .collect()
}
