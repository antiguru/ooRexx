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

//! The state a `Package` object holds in its own body: `PackageClass`'s
//! fields (`classes/PackageClass.hpp`).

use rexx_core::{ObjRef, PackageFields, PackageState};

use crate::Interp;
use crate::options::PackageOptions;
use crate::plan::{Package, ProgramId};

/// One package's fields. A copy is a field copy holding the same program.
#[derive(Clone, Debug, Default)]
pub(crate) struct PackageData {
    /// The program the package was compiled from, `None` for the REXX
    /// package. Every copy of a package holds the same one.
    pub(crate) program: Option<ProgramId>,
    /// What `::OPTIONS` directives and `Package~options` writes left on the
    /// package, `None` where neither has.
    pub(crate) options: Option<PackageOptions>,
    /// `PackageClass::initialPackageSettings`, entered at the first change.
    pub(crate) initial_options: Option<PackageOptions>,
}

impl PackageFields for PackageData {
    fn trace(&self, _out: &mut Vec<ObjRef>) {
        let PackageData {
            program: _,
            options: _,
            initial_options: _,
        } = self;
    }

    fn clone_fields(&self) -> Box<dyn PackageFields> {
        Box::new(self.clone())
    }
}

impl PackageData {
    /// A fresh package's fields, over `program`.
    pub(crate) fn state(program: Option<ProgramId>) -> PackageState {
        PackageState::new(Box::new(PackageData {
            program,
            ..PackageData::default()
        }))
    }
}

impl Interp {
    /// The object standing for `program`'s package, or `None` before one has
    /// been built.
    #[inline]
    pub(crate) fn program_package(&self, program: ProgramId) -> Option<ObjRef> {
        self.program_packages.get(program.0).copied().flatten()
    }

    /// `package`'s fields, or `None` for an object that is not a package.
    #[inline]
    pub(crate) fn package_data(&self, package: ObjRef) -> Option<&PackageData> {
        self.heap
            .get(package)?
            .body
            .package()?
            .downcast_ref::<PackageData>()
    }

    /// [`Interp::package_data`] for a caller that changes them.
    #[inline]
    pub(crate) fn package_data_mut(&mut self, package: ObjRef) -> Option<&mut PackageData> {
        self.heap
            .get_mut(package)?
            .body
            .package_mut()?
            .downcast_mut::<PackageData>()
    }

    /// The fields of `program`'s package, or `None` before it has been built.
    #[inline]
    pub(crate) fn program_data(&self, program: ProgramId) -> Option<&PackageData> {
        self.package_data(self.program_package(program)?)
    }

    /// The fields of `package`'s object, built first where it does not exist
    /// yet. May allocate, and so collect.
    pub(crate) fn package_fields_mut(&mut self, package: Package) -> &mut PackageData {
        let object = self.package_object(package);
        self.package_data_mut(object)
            .expect("package_object builds a package body")
    }

    /// Which package a package object's code belongs to, or `None` for an
    /// object that is not a package.
    pub(crate) fn which_package(&self, package: ObjRef) -> Option<Package> {
        Some(match self.package_data(package)?.program {
            Some(program) => Package::Program(program),
            None => Package::Rexx,
        })
    }
}
