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

use rexx_core::NameMap;
use rustc_hash::FxHashMap;

use crate::options::PackageOptions;
use crate::plan::{Package, ProgramId};
use crate::{Interp, MergedRoutine};

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
    /// The security manager the package carries, which all three
    /// `setSecurityManager` setters write.
    pub(crate) security_manager: Option<ObjRef>,
    /// Every `::ROUTINE` the program installs and every routine `~addRoutine`
    /// files, by upcased name.
    pub(crate) routines: FxHashMap<Box<[u8]>, MergedRoutine>,
    /// Whether a program has written the package's `.ROUTINES` table, whose
    /// entries routine lookups read in place of [`PackageData::routines`]
    /// from then on (`PackageClass::findLocalRoutine`,
    /// `classes/PackageClass.cpp:822`).
    pub(crate) routines_written: bool,
    /// `publicRoutines`: the subset of [`PackageData::routines`] a
    /// `::ROUTINE ... PUBLIC` filed.
    pub(crate) public_routines: FxHashMap<Box<[u8]>, MergedRoutine>,
    /// `mergedPublicRoutines`: every `::REQUIRES ... LIBRARY`'s library
    /// routines, then each `::REQUIRES`'s public and imported routines, a name
    /// keeping its first entry (`PackageClass::mergeLibrary` and
    /// `::mergeRequired`, `classes/PackageClass.cpp:693-772`).
    pub(crate) merged_public_routines: FxHashMap<Box<[u8]>, MergedRoutine>,
    /// `installedClasses`: the classes the package's `::CLASS` directives and
    /// `~addClass` installed, by uppercased name.
    pub(crate) classes: NameMap<Box<[u8]>, ObjRef>,
    /// `installedPublicClasses`, a second table beside
    /// [`PackageData::classes`] and not a flag on its entries
    /// (`classes/PackageClass.cpp:1406`-`:1419`).
    pub(crate) public_classes: NameMap<Box<[u8]>, ObjRef>,
    /// `mergedPublicClasses`, merged alongside the routines above.
    pub(crate) merged_public_classes: NameMap<Box<[u8]>, ObjRef>,
    /// The packages `::REQUIRES ... NAMESPACE` directives registered, under
    /// the upcased qualifier (`PackageClass::addNamespace`,
    /// `classes/PackageClass.cpp:2152`).
    pub(crate) namespaces: FxHashMap<Box<[u8]>, ObjRef>,
    /// The `Directory` `Package~local` answers, built on first ask.
    pub(crate) local: Option<ObjRef>,
    /// `loadedPackages`, in the order they were added.
    pub(crate) imports: Vec<ObjRef>,
    /// Every package whose public routines and classes were merged into this
    /// one. A merged `::ROUTINE` names its program, not an object, so this is
    /// what keeps the program's package alive for it, as the oracle's merged
    /// `Routine` object does. Not `~importedPackages`: an external call
    /// merges without importing.
    pub(crate) merged_from: Vec<ObjRef>,
    /// `parentPackage`: the package a routine or class lookup that misses
    /// this one's own tables walks next.
    pub(crate) parent: Option<ObjRef>,
    /// The `Routine` object standing for the program's main section.
    pub(crate) main_routine: Option<ObjRef>,
    /// The `.METHODS`, `.ROUTINES` and `.RESOURCES` tables, by
    /// [`PackageTable::index`], each built on first ask.
    pub(crate) tables: [Option<ObjRef>; 3],
    /// The value each `::CONSTANT` accessor answers, by declaring directive.
    pub(crate) constants: FxHashMap<usize, ObjRef>,
}

impl PackageFields for PackageData {
    fn trace(&self, out: &mut Vec<ObjRef>) {
        let PackageData {
            program: _,
            options: _,
            initial_options: _,
            security_manager,
            // Each `Routine` object here is in the package's `.ROUTINES`
            // table too, which `~addRoutine` writes.
            routines: _,
            routines_written: _,
            public_routines,
            merged_public_routines,
            classes,
            public_classes,
            merged_public_classes,
            namespaces,
            local,
            imports,
            merged_from,
            parent,
            main_routine,
            tables,
            constants,
        } = self;
        out.extend(*security_manager);
        // The `Routine` objects `~addRoutine` filed as themselves, which the
        // table does not follow into these copies.
        for table in [public_routines, merged_public_routines] {
            out.extend(table.values().filter_map(|merged| match merged {
                MergedRoutine::LibraryObject(_, object)
                | MergedRoutine::InternalObject(_, object) => Some(*object),
                _ => None,
            }));
        }
        // A package pins the classes it declares: measured, a `::CLASS`
        // class survives a forced collection on the oracle.
        for table in [classes, public_classes, merged_public_classes] {
            out.extend(table.values().copied());
        }
        out.extend(namespaces.values().copied());
        out.extend(*local);
        out.extend(imports.iter().copied());
        out.extend(merged_from.iter().copied());
        out.extend(*parent);
        out.extend(*main_routine);
        out.extend(tables.iter().flatten().copied());
        out.extend(constants.values().copied());
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
    pub(crate) fn package_fields_mut(
        &mut self,
        package: Package,
    ) -> Result<&mut PackageData, crate::error::Failure> {
        let object = self.package_object(package)?;
        Ok(self
            .package_data_mut(object)
            .expect("package_object builds a package body"))
    }

    /// The fields of `program`'s package, built first where it does not
    /// exist yet. May allocate, and so collect.
    pub(crate) fn program_fields_mut(
        &mut self,
        program: ProgramId,
    ) -> Result<&mut PackageData, crate::error::Failure> {
        self.package_fields_mut(Package::Program(program))
    }

    /// `package` and its fields, then each package up its parent chain
    /// (`PackageClass::parentPackage`).
    pub(crate) fn package_chain(
        &self,
        package: Option<ObjRef>,
    ) -> impl Iterator<Item = (ObjRef, &PackageData)> + '_ {
        let first = package.and_then(|package| Some((package, self.package_data(package)?)));
        // A parent is a package that existed before its child's program was
        // compiled, so the programs along a chain strictly decrease and the
        // chain cannot close; the bound costs less than that argument.
        std::iter::successors(first, |(_, data)| {
            let parent = data.parent?;
            Some((parent, self.package_data(parent)?))
        })
        .take(self.programs.len() + 1)
    }

    /// Whether `package` is the REXX package's own object, which
    /// `checkRexxPackage` refuses writes to (`classes/PackageClass.cpp:470`).
    /// A copy of it is not: the oracle's test is `isOldSpace`, and a copy is
    /// new.
    pub(crate) fn is_the_rexx_package(&self, package: ObjRef) -> bool {
        self.rexx_package == Some(package)
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
