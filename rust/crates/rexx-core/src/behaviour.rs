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

//! Behaviours: which methods an object responds to, and what it inherits.

use crate::body::BehaviourId;
use std::collections::HashMap;

/// Identifies a method body. The bodies themselves live elsewhere; this table
/// only answers "which method does this message resolve to".
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct MethodId(pub u32);

#[derive(Default)]
struct BehaviourEntry {
    superclass: Option<BehaviourId>,
    /// Keyed by the uppercased message name, because Rexx uppercases message
    /// names before dispatch.
    methods: HashMap<String, MethodId>,
}

pub struct BehaviourTable {
    entries: Vec<BehaviourEntry>,
}

impl BehaviourTable {
    pub fn new() -> Self {
        BehaviourTable {
            entries: Vec::new(),
        }
    }

    fn entry(&mut self, id: BehaviourId) -> &mut BehaviourEntry {
        let index = id.0 as usize;
        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, BehaviourEntry::default);
        }
        &mut self.entries[index]
    }

    pub fn define(&mut self, id: BehaviourId, name: &str, method: MethodId) {
        self.entry(id)
            .methods
            .insert(name.to_ascii_uppercase(), method);
    }

    pub fn set_superclass(&mut self, id: BehaviourId, superclass: BehaviourId) {
        self.entry(id).superclass = Some(superclass);
    }

    /// Resolves a message by walking the superclass chain.
    ///
    /// The visited set is not defensive programming: the bootstrap object
    /// graph is genuinely cyclic -- `.class` is an instance of itself -- so a
    /// chain walk that assumed acyclicity would hang during startup rather
    /// than in some exotic user program.
    pub fn lookup(&self, id: BehaviourId, name: &str) -> Option<MethodId> {
        let name = name.to_ascii_uppercase();
        let mut visited = Vec::new();
        let mut current = Some(id);
        while let Some(behaviour) = current {
            if visited.contains(&behaviour) {
                return None;
            }
            visited.push(behaviour);
            let entry = self.entries.get(behaviour.0 as usize)?;
            if let Some(method) = entry.methods.get(&name) {
                return Some(*method);
            }
            current = entry.superclass;
        }
        None
    }
}

impl Default for BehaviourTable {
    fn default() -> Self {
        Self::new()
    }
}
