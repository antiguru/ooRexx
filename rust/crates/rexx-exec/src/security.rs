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

//! The security manager (D12): where a manager object is stored, the one
//! send every checkpoint makes, and the info directory it is handed.

use rexx_core::{BehaviourId, Body, ObjRef};

use crate::error::Raised;
use crate::plan::ProgramId;
use crate::{Failure, Interp};

/// The message one checkpoint sends, and the entries its info directory
/// carries in. `SecurityManager.cpp`'s own six checks, one per method there.
pub(crate) mod message {
    pub(crate) const COMMAND: &[u8] = b"COMMAND";
    pub(crate) const STREAM: &[u8] = b"STREAM";
    pub(crate) const CALL: &[u8] = b"CALL";
    pub(crate) const REQUIRES: &[u8] = b"REQUIRES";
    pub(crate) const LOCAL: &[u8] = b"LOCAL";
    pub(crate) const ENVIRONMENT: &[u8] = b"ENVIRONMENT";
    pub(crate) const METHOD: &[u8] = b"METHOD";
}

/// The entry names the checkpoints read and write. `GlobalNames`' own
/// spellings, which are the keys the manager sees and sets.
pub(crate) mod key {
    pub(crate) const ADDRESS: &[u8] = b"ADDRESS";
    pub(crate) const ARGUMENTS: &[u8] = b"ARGUMENTS";
    pub(crate) const COMMAND: &[u8] = b"COMMAND";
    pub(crate) const ERROR: &[u8] = b"ERROR";
    pub(crate) const FAILURE: &[u8] = b"FAILURE";
    pub(crate) const NAME: &[u8] = b"NAME";
    pub(crate) const OBJECT: &[u8] = b"OBJECT";
    pub(crate) const RC: &[u8] = b"RC";
    pub(crate) const RESULT: &[u8] = b"RESULT";
    pub(crate) const SECURITYMANAGER: &[u8] = b"SECURITYMANAGER";
    pub(crate) const STREAM: &[u8] = b"STREAM";
}

impl Interp {
    /// Installs `manager` on `program`'s package, or clears it when the
    /// setter was sent no argument.
    ///
    /// `MethodClass::setSecurityManager` and `RoutineClass::setSecurityManager`
    /// both reach `RexxCode::setSecurityManager` (`execution/RexxCode.cpp:245`),
    /// which sets it on the package rather than on the executable, so all
    /// three setters write the same place.
    pub(crate) fn install_security_manager(&mut self, program: ProgramId, manager: Option<ObjRef>) {
        match manager {
            Some(manager) => {
                self.security_managers.insert(program, manager);
            }
            None => {
                self.security_managers.remove(&program);
            }
        }
    }

    /// `RexxActivation::getEffectiveSecurityManager`
    /// (`execution/RexxActivation.cpp:4977`): the running code's own package's
    /// manager. There is no instance-level manager here, so that arm's
    /// fallback is `None`.
    pub(crate) fn effective_security_manager(&self) -> Option<ObjRef> {
        if self.security_managers.is_empty() {
            return None;
        }
        self.security_managers
            .get(&self.activation().program_id)
            .copied()
    }

    /// One checkpoint: `SecurityManager::callSecurityManager`
    /// (`execution/SecurityManager.cpp:147`-`158`).
    ///
    /// Answers the info directory when the manager handled the checkpoint,
    /// and `None` when it did not or when no manager is installed. A manager
    /// that returns nothing is 91.999 and one that returns anything but `0`
    /// or `1` is 34.903, both raised here rather than by the caller.
    pub(crate) fn security_check(
        &mut self,
        message: &[u8],
        entries: &[(&[u8], ObjRef)],
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(manager) = self.effective_security_manager() else {
            return Ok(None);
        };
        let frame = self.roots.push_frame();
        let outcome = self.security_send(manager, message, entries);
        self.roots.pop_frame(frame);
        outcome
    }

    /// [`Interp::security_check`]'s body, so that the root frame it runs
    /// under is popped on the raising paths too.
    fn security_send(
        &mut self,
        manager: ObjRef,
        message: &[u8],
        entries: &[(&[u8], ObjRef)],
    ) -> Result<Option<ObjRef>, Failure> {
        let directory = self.security_arguments(entries)?;
        self.roots.push_temp(directory);
        let caller = self.caller();
        let answer = self.send_message(manager, message, None, &[Some(directory)], caller)?;
        let Some(answer) = answer else {
            return Err(Raised::no_result(message).into());
        };
        self.roots.push_temp(answer);
        let text = self.required_string_value(answer)?;
        let text = self.to_text(text).into_owned();
        match crate::eval::logical_value(&text) {
            Some(true) => Ok(Some(directory)),
            Some(false) => Ok(None),
            None => Err(Raised::authorization_not_logical(&text).into()),
        }
    }

    /// The info directory one checkpoint hands its manager.
    fn security_arguments(&mut self, entries: &[(&[u8], ObjRef)]) -> Result<ObjRef, Failure> {
        let class = self
            .classes()
            .lookup("Directory")
            .expect("Directory is a native class");
        let directory = self.native_instance(class);
        self.roots.push_temp(directory);
        let frame = self.roots.push_frame();
        for (name, value) in entries {
            let index = self.text(name);
            self.roots.push_temp(index);
            let caller = self.caller();
            self.send_message(
                directory,
                b"PUT",
                None,
                &[Some(*value), Some(index)],
                caller,
            )?;
        }
        self.roots.pop_frame(frame);
        Ok(directory)
    }

    /// The `ARGUMENTS` array a `METHOD` or `CALL` checkpoint carries, holes
    /// and all -- `new_array(count, arguments)`, which copies the argument
    /// list as it stands.
    pub(crate) fn security_arguments_array(&mut self, args: &[Option<ObjRef>]) -> ObjRef {
        let array = self.alloc_with(BehaviourId::ARRAY, Body::array(args.to_vec()));
        self.roots.push_temp(array);
        array
    }

    /// One entry of an info directory the manager has answered, or `None`
    /// where it left the name unset. `DirectoryClass::get`, which is an
    /// exact-string lookup and not `ENTRY`'s upcasing one.
    pub(crate) fn security_entry(
        &mut self,
        directory: ObjRef,
        name: &[u8],
    ) -> Result<Option<ObjRef>, Failure> {
        let index = self.text(name);
        self.roots.push_temp(index);
        let caller = self.caller();
        let answer = self.send_message(directory, b"AT", None, &[Some(index)], caller)?;
        Ok(answer.filter(|value| *value != ObjRef::NIL))
    }
}
