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

//! The [`Surface`] an extension's callbacks reach this interpreter through.

use std::borrow::Cow;

use rexx_api::callbacks::Surface;
use rexx_core::{BehaviourId, Body, ObjRef};

use crate::error::{Raised, displayable};
use crate::{Failure, Interp};

impl Surface for Interp {
    fn clear_condition(&mut self) {
        let frame = self.native_frame_mut();
        frame.raised = None;
        frame.additional = None;
        frame.result = None;
        frame.condition = None;
    }

    fn new_array(&mut self, items: &[Option<ObjRef>]) -> ObjRef {
        self.alloc_with(
            BehaviourId::ARRAY,
            Body::Array {
                dimensions: None,
                slots: items.to_vec(),
            },
        )
    }

    fn raise_exception(&mut self, number: usize, substitutions: Option<ObjRef>) {
        // Each item's `stringValue()`, the default name where that raises,
        // and nothing for an empty slot, which is `messageSubstitution`'s
        // (`interpreter/concurrency/Activity.cpp:1246`).
        let items = substitutions
            .and_then(|array| self.array_slots_of(array))
            .unwrap_or_default();
        let texts: Vec<Vec<u8>> = items
            .into_iter()
            .map(|item| {
                item.map(|object| self.native_found(object))
                    .unwrap_or_default()
            })
            .collect();
        let major = u16::try_from(number / 1000).unwrap_or(u16::MAX);
        let minor = u16::try_from(number % 1000).unwrap_or(0);
        let frame = self.native_frame_mut();
        frame.raised = Some(Raised::syntax(major, minor, texts).into());
        frame.additional = substitutions;
        frame.result = None;
        frame.condition = None;
    }

    fn raise_condition(
        &mut self,
        name: &[u8],
        description: Option<ObjRef>,
        additional: Option<ObjRef>,
        result: Option<ObjRef>,
    ) {
        let description = description.map(|text| self.string_value_text(text));
        let raised = Raised {
            description,
            ..Raised::condition(Cow::Owned(String::from_utf8_lossy(name).into_owned()))
        };
        let frame = self.native_frame_mut();
        frame.raised = Some(raised.into());
        frame.additional = additional;
        frame.result = result;
        frame.condition = None;
    }

    fn has_condition(&mut self) -> bool {
        self.native_frame().raised.is_some()
    }

    fn condition_object(&mut self) -> Option<ObjRef> {
        let frame = self.native_frame();
        if let Some(object) = frame.condition {
            return Some(object);
        }
        let Some(Failure::Raised(raised)) = &frame.raised else {
            return None;
        };
        let (raised, additional, result) = (raised.clone(), frame.additional, frame.result);
        self.pending_additional = additional;
        self.pending_result = result;
        let object = self.build_native_condition_object(&raised).ok()?;
        self.native_frame_mut().condition = Some(object);
        Some(object)
    }

    fn display_condition(&mut self) -> isize {
        let Some(object) = self.condition_object() else {
            return 0;
        };
        let condition = self.condition_entry(object, b"CONDITION");
        if condition
            .map(|name| self.string_value_text(name))
            .as_deref()
            != Some(b"SYNTAX")
        {
            return 0;
        }
        let report = self.condition_report(object);
        self.write_trace_report(&report);
        // `Error_Interpretation / 1000` where `RC` is not a number.
        self.condition_entry(object, b"RC")
            .and_then(|rc| {
                std::str::from_utf8(&self.string_value_text(rc))
                    .ok()?
                    .trim()
                    .parse()
                    .ok()
            })
            .unwrap_or(49)
    }

    fn directory_entry(&mut self, directory: ObjRef, name: &[u8]) -> Option<ObjRef> {
        self.condition_entry(directory, name)
    }
}

impl Interp {
    /// `Activity::display` (`interpreter/concurrency/Activity.cpp:1414`):
    /// the condition object's traceback lines, then the error lines its
    /// entries spell.
    fn condition_report(&mut self, object: ObjRef) -> Vec<u8> {
        let mut report = Vec::new();
        if let Some(traceback) = self.condition_entry(object, b"TRACEBACK") {
            let caller = self.caller();
            let lines = self
                .send_message(traceback, b"MAKEARRAY", None, &[], caller)
                .ok()
                .flatten()
                .and_then(|array| self.array_slots_of(array))
                .unwrap_or_default();
            for line in lines.into_iter().flatten() {
                if line != ObjRef::NIL {
                    report.extend_from_slice(&self.string_value_text(line));
                    report.push(b'\n');
                }
            }
        }
        let entry = |interp: &mut Interp, name: &[u8]| {
            interp
                .condition_entry(object, name)
                .map(|value| interp.string_value_text(value))
        };
        let rc = entry(self, b"RC").unwrap_or_default();
        report.extend_from_slice(b"Error ");
        report.extend_from_slice(&rc);
        if let Some(program) = entry(self, b"PROGRAM").filter(|program| !program.is_empty()) {
            report.extend_from_slice(b" running ");
            report.extend_from_slice(&program);
            if let Some(position) = entry(self, b"POSITION") {
                report.extend_from_slice(b" line ");
                report.extend_from_slice(&position);
            }
        }
        report.extend_from_slice(b":  ");
        report.extend_from_slice(&entry(self, b"ERRORTEXT").unwrap_or_default());
        report.push(b'\n');
        if let Some(message) = entry(self, b"MESSAGE") {
            report.extend_from_slice(b"Error ");
            report.extend_from_slice(&entry(self, b"CODE").unwrap_or_default());
            report.extend_from_slice(b":  ");
            report.extend_from_slice(&message);
            report.push(b'\n');
        }
        displayable(&mut report);
        report
    }
}
