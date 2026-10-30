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

use super::*;

/// One row's owner check.
pub fn check_row(row: &Row, context: &Context) -> Result<(), String> {
    if !context.groups.contains(&row.group) {
        return Err("no such group".to_string());
    }
    let text = read_lossy(&context.group_file(&row.group));
    let classes = source_classes(&text);
    let defined = classes
        .iter()
        .any(|class| class.methods.contains(&row.test));
    if !defined {
        return Err("the group file defines no such method".to_string());
    }
    match row.owner {
        Owner::Phase10(reason) => check_phase_10(row, reason, context),
        Owner::Deviation(number) => {
            if context.deviations.contains(&number) {
                Ok(())
            } else {
                Err(format!(
                    "{DEVIATIONS} has no Deviation {number} that is not withdrawn"
                ))
            }
        }
        Owner::Environment => check_environment(row, context),
        Owner::Flaky => {
            let stem = row.group.rsplit('/').next().expect("a group name");
            let entry = row
                .reason
                .strip_prefix("common.txt: ")
                .ok_or("the reason is not `common.txt: ENTRY`")?
                .to_ascii_uppercase();
            if !context.flaky.contains(&entry) {
                Err(format!("{COMMON_FAILURES} has no entry {entry}"))
            } else if entry != format!("{stem}.TESTGROUP/{}", row.test).to_ascii_uppercase() {
                Err(format!("{entry} is not this row's test"))
            } else {
                Ok(())
            }
        }
        Owner::NotRun => check_not_run(row, &text, context),
    }
}

fn check_phase_10(row: &Row, reason: Deferral, context: &Context) -> Result<(), String> {
    if row.reason == "reaching_rxapi" {
        return if reason != Deferral::Rxapi {
            Err("a reaching_rxapi row is owned by Phase 10 Rxapi".to_string())
        } else if context
            .rxapi
            .contains(&(row.group.clone(), row.test.clone()))
        {
            Ok(())
        } else {
            Err("reaching_rxapi does not rename it out".to_string())
        };
    }
    if row.reason == "group" {
        return match phase_10_group(&row.group) {
            Some(found) if found == reason => Ok(()),
            Some(found) => Err(format!("the group is Phase 10's for {}", found.name())),
            None => Err("the roadmap does not make the group Phase 10's".to_string()),
        };
    }
    match row.refusal() {
        Some(text) if refusal_carries(text, reason) => Ok(()),
        Some(text) => Err(format!(
            "the refusal does not carry Phase 10 {}: {text}",
            reason.name()
        )),
        None => Err("the reason is not reaching_rxapi, group or `refusal: TEXT`".to_string()),
    }
}

fn check_environment(row: &Row, context: &Context) -> Result<(), String> {
    let record = row
        .reason
        .split("; ")
        .find_map(|part| part.strip_prefix("record: "))
        .ok_or("no `record: PATH`")?;
    if !record.starts_with(ORACLE_RUNS) {
        return Err(format!("the record is not under {ORACLE_RUNS}"));
    }
    let tracked = Command::new("git")
        .arg("-C")
        .arg(&context.worktree)
        .args(["ls-files", "--error-unmatch", record])
        .output()
        .expect("cannot run git");
    if !tracked.status.success() {
        return Err(format!("{record} is not committed"));
    }
    if !read_lossy(&context.worktree.join(record)).contains(&row.test) {
        return Err(format!("{record} does not name the test"));
    }
    let ruled = row
        .reason
        .split("; ")
        .find_map(|part| part.strip_prefix("ruled: Moritz "))
        .ok_or("no `ruled: Moritz DATE`")?;
    let date: Vec<&str> = ruled.split('-').collect();
    if date.len() == 3
        && [4, 2, 2]
            .iter()
            .zip(&date)
            .all(|(len, part)| part.len() == *len && part.bytes().all(|b| b.is_ascii_digit()))
    {
        Ok(())
    } else {
        Err(format!("{ruled:?} is not a date"))
    }
}

pub(super) fn check_not_run(row: &Row, text: &str, context: &Context) -> Result<(), String> {
    if let Some(line) = row.reason.strip_prefix("restrictOS line ") {
        let number: usize = line.parse().map_err(|_| "not a line number")?;
        let source = text
            .lines()
            .nth(number.wrapping_sub(1))
            .ok_or("no such line")?;
        let lower = source.to_ascii_lowercase();
        let at = lower
            .find("restrictos(")
            .ok_or("the line holds no restrictOS")?;
        let argument = source[at + "restrictos(".len()..]
            .trim_start()
            .trim_start_matches(['\'', '"']);
        let argument = argument
            .split(['\'', '"'])
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        let words: Vec<&str> = argument.split_whitespace().collect();
        return if words.is_empty() || words == ["UNIX"] || words.contains(&"LINUX") {
            Err(format!("restrictOS({argument:?}) admits LINUX"))
        } else {
            Ok(())
        };
    }
    if let Some(name) = row.reason.strip_prefix("test type ") {
        if !context.skipped_types.contains(name) {
            return Err(format!("{name} is a type the framework runs by default"));
        }
        let classes = source_classes(text);
        let tests = test_classes(&classes);
        let assignment = format!("~{}", name.to_ascii_lowercase());
        let typed = classes.iter().any(|class| {
            tests.contains(&class.name)
                && class.methods.contains(&row.test)
                && class.body.contains("ootesttype")
                && class.body.contains(&assignment)
        });
        return if typed {
            Ok(())
        } else {
            Err(format!("no test class defining it sets its type to {name}"))
        };
    }
    Err("the reason is not `restrictOS line N` or `test type NAME`".to_string())
}

