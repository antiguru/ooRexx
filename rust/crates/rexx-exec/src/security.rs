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
use rexx_parse::Access;

use crate::dispatch::AccessScope;
use crate::error::Raised;
use crate::plan::{Package, ProgramId};
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
    /// Installs `manager` on `package`, or clears it when the setter was sent
    /// no argument.
    ///
    /// `MethodClass::setSecurityManager` and `RoutineClass::setSecurityManager`
    /// both reach `RexxCode::setSecurityManager` (`execution/RexxCode.cpp:245`),
    /// which sets it on the package rather than on the executable, so all
    /// three setters write the same place.
    ///
    /// Every activation already running `package`'s code keeps the manager it
    /// started under, as the oracle's copies it into its settings when it
    /// begins.
    pub(crate) fn install_security_manager(&mut self, package: ObjRef, manager: Option<ObjRef>) {
        if let Some(previous) = self.package_data(package).map(|data| data.security_manager) {
            self.keep_security_manager(package, previous);
        }
        self.security_armed |= manager.is_some();
        if let Some(data) = self.package_data_mut(package) {
            data.security_manager = manager;
        }
    }

    /// Gives every live activation of `package`'s code that does not yet
    /// keep a manager of its own `previous` to keep.
    /// A package installing before its own activation runs keeps it too, for
    /// that activation and the rest of the install.
    #[cold]
    fn keep_security_manager(&mut self, package: ObjRef, previous: Option<ObjRef>) {
        for row in 0..self.installing_managers.len() {
            let (program, kept) = self.installing_managers[row];
            if kept.is_none() && self.program_package(program) == Some(package) {
                self.installing_managers[row].1 = Some(previous);
            }
        }
        let mut programs = Vec::new();
        self.activity.running_programs(&mut programs);
        self.activities.running_programs(&mut programs);
        programs.retain(|&program| self.program_package(program) == Some(package));
        if programs.is_empty() {
            return;
        }
        let mut keep = |activation: &mut crate::activation::Activation| {
            if programs.contains(&activation.program_id)
                && activation
                    .cold
                    .as_deref()
                    .is_none_or(|cold| cold.security_manager.is_none())
            {
                activation.cold_mut().security_manager = Some(previous);
            }
        };
        self.activity.for_each_activation_mut(&mut keep);
        self.activities.for_each_activation_mut(&mut keep);
    }

    /// `RexxActivation::getEffectiveSecurityManager`
    /// (`execution/RexxActivation.cpp:4977`): the manager the running
    /// activation started under. There is no instance-level manager here, so
    /// that arm's fallback is `None`, as it is with no running activation.
    #[inline]
    pub(crate) fn effective_security_manager(&self) -> Option<ObjRef> {
        if !self.security_armed {
            return None;
        }
        self.armed_security_manager()
    }

    /// [`Interp::effective_security_manager`] once any manager has been
    /// installed. A started message's activity runs no activation, and the
    /// oracle finds no manager for it.
    #[inline(never)]
    fn armed_security_manager(&self) -> Option<ObjRef> {
        let activation = self.activity.running.as_deref()?;
        if let Some(kept) = activation
            .cold
            .as_deref()
            .and_then(|cold| cold.security_manager)
        {
            return kept;
        }
        self.program_data(activation.program_id)?.security_manager
    }

    /// The manager the running code's package has now, which its `LOCAL` and
    /// `ENVIRONMENT` checkpoints ask: `PackageClass::findClass` reads the
    /// package's own (`classes/PackageClass.cpp:1135`), not the activation's.
    #[inline]
    pub(crate) fn package_security_manager(&self) -> Option<ObjRef> {
        if !self.security_armed {
            return None;
        }
        self.armed_package_security_manager()
    }

    /// [`Interp::package_security_manager`] once any manager has been
    /// installed.
    #[inline(never)]
    fn armed_package_security_manager(&self) -> Option<ObjRef> {
        let program = self.activity.running.as_deref()?.program_id;
        self.program_data(program)?.security_manager
    }

    /// The manager `program`'s own installing activation runs under, which is
    /// the one its `::REQUIRES` directives consult
    /// (`RexxActivation::run` installs the package, `RexxActivation.cpp:508`):
    /// the package's when its install began.
    pub(crate) fn installing_security_manager(&self, program: ProgramId) -> Option<ObjRef> {
        if !self.security_armed {
            return None;
        }
        if let Some(kept) = self.kept_installing_manager(program) {
            return kept;
        }
        self.program_data(program)?.security_manager
    }

    /// The manager `program` had when its install, still running, began,
    /// where one has been set on it since.
    pub(crate) fn kept_installing_manager(&self, program: ProgramId) -> Option<Option<ObjRef>> {
        self.installing_managers
            .iter()
            .rev()
            .find(|(installing, _)| *installing == program)
            .and_then(|(_, kept)| *kept)
    }

    /// One checkpoint: `SecurityManager::callSecurityManager`
    /// (`execution/SecurityManager.cpp:147`-`158`).
    ///
    /// Answers the info directory, rooted as a temp of the caller's frame,
    /// when the manager handled the checkpoint, and `None` when it did not or
    /// when no manager is installed. A manager
    /// that returns nothing is 91.999 and one that returns anything but `0`
    /// or `1` is 34.903, both raised here rather than by the caller.
    pub(crate) fn security_check(
        &mut self,
        message: &[u8],
        entries: &[(&[u8], ObjRef)],
    ) -> Result<Option<ObjRef>, Failure> {
        let manager = self.effective_security_manager();
        self.security_check_by(manager, message, entries)
    }

    /// [`Interp::security_check`] against `manager` rather than the running
    /// activation's.
    pub(crate) fn security_check_by(
        &mut self,
        manager: Option<ObjRef>,
        message: &[u8],
        entries: &[(&[u8], ObjRef)],
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(manager) = manager else {
            return Ok(None);
        };
        let frame = self.roots.activity_mut().push_frame();
        let outcome = self.security_send(manager, message, entries);
        self.roots.activity_mut().pop_frame(frame);
        if let Ok(Some(directory)) = outcome {
            self.roots.activity_mut().push_temp(directory);
        }
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
        self.roots.activity_mut().push_temp(directory);
        let caller = self.caller();
        let answer = pinned!(
            self,
            crate::pinning::PinKind::SecurityManager,
            self.send_message(manager, message, None, &[Some(directory)], caller)
        )?;
        let Some(answer) = answer else {
            return Err(Raised::no_result(message).into());
        };
        self.roots.activity_mut().push_temp(answer);
        if self.truth(answer, Raised::authorization_not_logical)? {
            Ok(Some(directory))
        } else {
            Ok(None)
        }
    }

    /// The info directory one checkpoint hands its manager, filled by
    /// `DirectoryClass::put` itself rather than a sent `PUT`.
    fn security_arguments(&mut self, entries: &[(&[u8], ObjRef)]) -> Result<ObjRef, Failure> {
        let directory = crate::dispatch::hash::new_directory(self, 0)?;
        self.roots.activity_mut().push_temp(directory);
        let frame = self.roots.activity_mut().push_frame();
        for (name, value) in entries {
            crate::dispatch::hash::directory_put(self, directory, name, *value)?;
        }
        self.roots.activity_mut().pop_frame(frame);
        Ok(directory)
    }

    /// The `ARGUMENTS` array a `METHOD` or `CALL` checkpoint carries, holes
    /// and all -- `new_array(count, arguments)`, which copies the argument
    /// list as it stands.
    pub(crate) fn security_arguments_array(&mut self, args: &[Option<ObjRef>]) -> ObjRef {
        let array = self.alloc_charged(BehaviourId::ARRAY, Body::array(args.to_vec()));
        self.roots.activity_mut().push_temp(array);
        array
    }

    /// One entry of an info directory the manager has answered, or `None`
    /// where it left the name unset. `DirectoryClass::get`, which is an
    /// exact-string lookup and not `ENTRY`'s upcasing one. An entry the
    /// manager set to `.nil` is `Some(.nil)`: `get` answers the object stored.
    pub(crate) fn security_entry(
        &mut self,
        directory: ObjRef,
        name: &[u8],
    ) -> Result<Option<ObjRef>, Failure> {
        let index = self.text(name);
        self.roots.activity_mut().push_temp(index);
        let answer = crate::dispatch::hash::store_get(self, directory, index)?;
        if let Some(value) = answer {
            self.roots.activity_mut().push_temp(value);
        }
        Ok(answer)
    }

    /// `MethodClass::setProtected` on the method `object` stands for, which
    /// the dispatch seam then asks the manager about.
    pub(crate) fn make_method_protected(&mut self, object: ObjRef) {
        let Some(method) = self.unshared_identity(object) else {
            return;
        };
        let package = self
            .source_package(self.installed_executable_source(method))
            .unwrap_or(Package::Rexx);
        match self.special_methods.get_mut(method) {
            Some(existing) => existing.protected = true,
            None => self.special_methods.insert(
                method,
                AccessScope {
                    access: Access::Default,
                    protected: true,
                    package,
                },
            ),
        }
    }

    /// Whether the method `object` stands for is protected: a `Setup.cpp`
    /// protected primitive, or one `setProtected` was sent to.
    pub(crate) fn installed_is_protected(&self, object: ObjRef) -> bool {
        self.executable_sources
            .get(&object)
            .and_then(|record| record.installed)
            .and_then(|method| self.special_methods.get(method))
            .is_some_and(|scope| scope.protected)
    }
}
