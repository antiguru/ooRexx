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

//! The interpreter's own object identities: the package, `Method` and
//! `Routine` objects and the annotation tables, each built on first ask and
//! kept, and the class and routine lookups a package object answers.

use super::{
    Annotated, BehaviourId, Body, ClassPackage, EnvScope, Failure, Interp, Loud, NativeObject,
    ObjRef, Package, ProgramId, default_object_name, env_seam, hash, package_root_key,
};

impl Interp {
    /// Records a class a `::CLASS` directive installed, under the running
    /// package's own id.
    pub(crate) fn record_package_class(
        &mut self,
        program: ProgramId,
        name: &[u8],
        class: ObjRef,
        public: bool,
    ) {
        self.invalidate_rexx_class_cache();
        self.package_classes
            .entry(program)
            .or_default()
            .insert(name.to_ascii_uppercase().into(), class);
        if public {
            self.package_public_classes
                .entry(program)
                .or_default()
                .insert(name.to_ascii_uppercase().into(), class);
        }
        // **A class the interpreter's own library installed belongs to the
        // `REXX` package, not to the program whose directives installed it.**
        // Leaving it out of this table is what `Interp::package_object_for`
        // reads as `Package::Rexx`, which is the oracle's answer: measured
        // on both engines and the oracle, `.Alarm~package~name` is `REXX`.
        if !self.library_programs.contains(&program) {
            self.class_packages
                .insert(class, ClassPackage::Program(program));
        }
    }

    /// Records a class `Class~subclass` or `Class~mixinClass` built, whose
    /// own `package` field the oracle leaves null
    /// (`classes/ClassClass.cpp:1546`, `:1496`, then `:1582`).
    pub(crate) fn record_packageless_class(&mut self, class: ObjRef) {
        self.class_packages.insert(class, ClassPackage::Null);
    }

    /// `Package~addClass` and `Package~addPublicClass`, which differ only in
    /// whether the public table gets the entry too --
    /// `PackageClass::addInstalledClass` (`classes/PackageClass.cpp:1401`),
    /// which both `addClassRexx` (`:1932`) and `addPublicClassRexx`
    /// (`:1950`) reach with the flag set differently.
    pub(crate) fn add_installed_class(
        &mut self,
        program: ProgramId,
        name: &[u8],
        class: ObjRef,
        public: bool,
    ) {
        // The reverse direction is deliberately not written here.
        // `~package` answers the package a class was *defined* in, which is
        // `RexxClass::package`, a field of the class object;
        // `addInstalledClass` writes the package's own two tables and
        // touches no field of the class it is handed, so adding a class to a
        // package's table does not move it.
        self.invalidate_rexx_class_cache();
        self.package_classes
            .entry(program)
            .or_default()
            .insert(name.to_ascii_uppercase().into(), class);
        if public {
            self.package_public_classes
                .entry(program)
                .or_default()
                .insert(name.to_ascii_uppercase().into(), class);
        }
    }

    /// The package object `class~package` answers, or `.nil` for a class that
    /// belongs to no package.
    pub(crate) fn package_object_for(&mut self, class: ObjRef) -> ObjRef {
        // A class this crate's own bootstrap registered is in no program's
        // table, and that absence is what `Package::Rexx` names -- the
        // distinction the enum exists to keep out of an `Option`.
        let package = match self.class_packages.get(&class).copied() {
            None => Package::Rexx,
            Some(ClassPackage::Null) => return ObjRef::NIL,
            Some(ClassPackage::Program(program)) => Package::Program(program),
        };
        self.package_object(package)
    }

    /// The one object standing for `package`, built on first use.
    pub(crate) fn package_object(&mut self, package: Package) -> ObjRef {
        if let Some(found) = self.package_objects.get(&package).copied() {
            return found;
        }
        let package_class = self.package_class();
        // The `REXX` package was given its name by the interpreter's own
        // startup rather than derived from a class id, exactly as `.environment`
        // and `.local` were, and a package a program installed into
        // carries no name of its own -- measured, `.Array~package` renders
        // `The REXX Package` and a `::CLASS`'s renders `a Package`.
        let rendered = match package {
            Package::Rexx => b"The REXX Package".to_vec(),
            Package::Program(_) => default_object_name(self.class_id_text(package_class))
                .as_bytes()
                .to_vec(),
        };
        let object = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Native(Box::new(NativeObject::new(package_class, &rendered))),
        );
        // A package object outlives every send that can reach it and is
        // reachable from no other object, so the root is a global rather than
        // a temp -- the position `.environment` and `.local` are in.
        self.roots.add_global(&package_root_key(package), object);
        self.package_objects.insert(package, object);
        self.attach_annotations(object, Annotated::Package(package));
        object
    }

    /// `Class~method`'s answer: the one `Method` object this class's instance
    /// dictionary entry named `name` has, built on first ask and kept.
    pub(crate) fn method_object(
        &mut self,
        class: ObjRef,
        name: &[u8],
        scope: ObjRef,
        record: crate::ExecutableRecord,
    ) -> ObjRef {
        let key = (class, Box::<[u8]>::from(name));
        if let Some(found) = self.method_objects.get(&key).copied() {
            return found;
        }
        let method_class = self.method_class();
        let object = self.native_instance(method_class);
        self.executable_sources.insert(object, record);
        let held = self.heap.get_mut(object).expect("just allocated");
        let Body::Native(native) = &mut held.body else {
            unreachable!("allocated as Body::Native by native_instance")
        };
        native.set_scope(scope);
        self.class_owns(class, object);
        self.method_objects.insert(key, object);
        // The dictionary entry the caller found, which is what its
        // annotations are keyed by: this class, the instance side, this name.
        self.attach_annotations(object, Annotated::Member(class, false, name.into()));
        object
    }

    /// The one `Method` object for the method `name` defined at `scope` --
    /// what `RexxContext~executable` answers from a `::METHOD` context.
    pub(crate) fn method_executable(
        &mut self,
        scope: ObjRef,
        name: &[u8],
    ) -> Result<ObjRef, crate::error::Failure> {
        let found = self
            .classes()
            .own_instance_slot(scope, &String::from_utf8_lossy(name));
        match found {
            Some(rexx_classes::MethodSlot::Defined { scope, method }) => {
                let record = crate::ExecutableRecord {
                    source: self.installed_executable_source(method),
                    installed: Some(method),
                    routine: None,
                };
                Ok(self.method_object(scope, name, scope, record))
            }
            _ => Err(crate::Loud::receiver_class(
                "a method context whose scope no longer defines it",
            )
            .into()),
        }
    }

    /// The one `Routine` object standing for `program`'s own main section --
    /// what `RexxContext~executable` answers from a `PROGRAM` or
    /// `INTERNALCALL` context. [`Interp::program_routine_objects`] carries
    /// why the identity is kept.
    pub(crate) fn program_routine_object(&mut self, program: ProgramId) -> ObjRef {
        if let Some(found) = self.program_routine_objects.get(&program).copied() {
            return found;
        }
        let class = self.routine_class();
        let object = self.native_instance(class);
        self.roots
            .add_global(&program_routine_root_key(program), object);
        self.program_routine_objects.insert(program, object);
        self.executable_sources.insert(
            object,
            crate::ExecutableRecord {
                source: crate::ExecutableSource::Main { program },
                installed: None,
                routine: None,
            },
        );
        object
    }

    /// `MethodClass::newScope` (`classes/MethodClass.cpp:183`): the same
    /// method object with `scope` filled in when it had none, and a copy
    /// carrying `scope` when it already had one.
    fn method_new_scope(&mut self, method: ObjRef, scope: ObjRef) -> Option<ObjRef> {
        let mut copy = match self.heap.get(method).map(|held| &held.body) {
            Some(Body::Native(native)) if native.scope().is_none() => {
                let object = self.heap.get_mut(method).expect("read just above");
                let Body::Native(native) = &mut object.body else {
                    unreachable!("matched as Body::Native just above")
                };
                native.set_scope(scope);
                return Some(method);
            }
            Some(Body::Native(native)) => native.clone(),
            _ => return None,
        };
        copy.set_scope(scope);
        // The clone is out of the collector's sight until `alloc_with`
        // returns, and `alloc_with` can collect. The only arena handle it
        // carries is its annotation table, which
        // [`Interp::annotation_table`] roots as a global for the whole run,
        // so nothing the clone reaches can be swept while it is detached.
        let object = self.alloc_with(BehaviourId::OBJECT, Body::Native(copy));
        self.roots.push_temp(object);
        // The copy reports on the same directive the original did, and
        // carries the flag writes the four setters made on it -- both are
        // `RexxObject::copy`'s doing, which duplicates the whole method
        // object including its flag word. Measured, oracle rc 0:
        // `.K2~define("X", .K~method("M"))` then `.K2~method("X")~source`
        // answers `M`'s own body lines, and the same copy taken after
        // `setPrivate` and `setUnguarded` answers `1 0` for `isPrivate` and
        // `isGuarded` where the directive alone answers `0 1`.
        if let Some(source) = self.executable_sources.get(&method).copied() {
            self.executable_sources.insert(object, source);
        }
        if let Some(writes) = self.method_flag_writes.get(&method).copied() {
            self.method_flag_writes.insert(object, writes);
        }
        Some(object)
    }

    /// `~define` with a method object: install it in `class`'s own instance
    /// dictionary under `name` and make [`Interp::method_object`] answer it.
    pub(crate) fn define_method_object(
        &mut self,
        class: ObjRef,
        name: &[u8],
        source: ObjRef,
    ) -> Option<()> {
        let object = self.method_new_scope(source, class)?;
        let method = self.classes().mint_method_id();
        self.bind_loaded_method(method, object);
        self.classes()
            .define_instance_method(class, &String::from_utf8_lossy(name), method);
        self.hold_method_object(class, name, object);
        Some(())
    }

    /// Makes `method` run the library procedure `object` is, where `object` is
    /// a `loadExternalMethod` answer, and do nothing otherwise.
    fn bind_loaded_method(&mut self, method: rexx_classes::MethodId, object: ObjRef) {
        let Some(crate::ExecutableRecord {
            source: crate::ExecutableSource::Loaded { code },
            ..
        }) = self.executable_sources.get(&object).copied()
        else {
            return;
        };
        let key = self.library_code_key(code).clone();
        if key.routine {
            return;
        }
        let Some(library) = self.libraries.get(&key.library).map(std::rc::Rc::clone) else {
            return;
        };
        self.library_externals.insert(
            method,
            crate::LibraryBinding {
                library,
                procedure: key.procedure,
            },
        );
        self.defined_library_codes.insert(method, code);
    }

    /// `defineClassMethod`: the same shape as [`Interp::define_method_object`]
    /// on the class side, plus the row that makes the installed method
    /// runnable.
    pub(crate) fn define_class_method_object(
        &mut self,
        class: ObjRef,
        name: &[u8],
        source: ObjRef,
    ) -> Option<()> {
        let body = self.table_method_bodies.get(&source).copied()?;
        let object = self.method_new_scope(source, class)?;
        let method = self.classes().mint_method_id();
        self.classes()
            .define_class_method(class, &String::from_utf8_lossy(name), method);
        self.method_bodies.insert(method, body);
        self.hold_method_object(class, name, object);
        Some(())
    }

    /// `~defineMethods`: one mutation for the whole table.
    pub(crate) fn define_method_table(
        &mut self,
        class: ObjRef,
        entries: &[(Box<[u8]>, Option<ObjRef>)],
    ) -> Option<()> {
        let mut installed: Vec<(String, Option<rexx_classes::MethodId>)> = Vec::new();
        let mut objects: Vec<(Box<[u8]>, ObjRef)> = Vec::new();
        let frame = self.roots.push_frame();
        for (name, source) in entries {
            let name_text = String::from_utf8_lossy(name).into_owned();
            let Some(source) = *source else {
                installed.push((name_text, None));
                continue;
            };
            let object = self.method_new_scope(source, class)?;
            self.roots.push_temp(object);
            let object = self.method_new_scope(object, class)?;
            self.roots.push_temp(object);
            let method = self.classes().mint_method_id();
            self.bind_loaded_method(method, object);
            installed.push((name_text, Some(method)));
            objects.push((name.clone(), object));
        }
        self.classes().define_instance_methods(class, &installed);
        for (name, object) in objects {
            self.hold_method_object(class, &name, object);
        }
        self.roots.pop_frame(frame);
        Some(())
    }

    /// Make [`Interp::method_object`] answer `object` for this dictionary
    /// entry, and root it the way one this crate built is rooted.
    fn hold_method_object(&mut self, class: ObjRef, name: &[u8], object: ObjRef) {
        self.class_owns(class, object);
        self.method_objects.insert((class, name.into()), object);
    }

    /// Forget the `Method` object this dictionary entry answered, for a
    /// `~delete` or a `~define` that took the entry away.
    pub(crate) fn drop_method_object(&mut self, class: ObjRef, name: &[u8]) {
        self.method_objects.remove(&(class, name.into()));
    }

    /// The `StringTable` `~annotations` answers for `site`, built empty on
    /// first ask and kept.
    pub(crate) fn annotation_table(&mut self, site: Annotated) -> ObjRef {
        if let Some(found) = self.annotations.get(&site).copied() {
            return found;
        }
        let string_table = self.environment_model().string_table;
        let table = self.native_instance(string_table);
        // A class-owned site is held by its class, so the table dies with it.
        // Every other site belongs to a package or a program, which outlive
        // the run, and those keep the global root.
        match site {
            Annotated::Class(owner) | Annotated::Member(owner, _, _) => {
                self.class_owns(owner, table);
            }
            _ => self.roots.add_global(&annotation_root_key(&site), table),
        }
        self.annotations.insert(site, table);
        table
    }

    /// Records what one `::ANNOTATE` directive named, under every key that
    /// reaches it.
    pub(crate) fn record_annotations(
        &mut self,
        sites: &[Annotated],
        pairs: &[(Box<[u8]>, Box<[u8]>)],
    ) {
        let Some((first, rest)) = sites.split_first() else {
            return;
        };
        let table = self.annotation_table(first.clone());
        // Each value is pushed as a temporary as it is built, because the
        // next value's own allocation may collect and the table does not hold
        // it until the line below.
        let frame = self.roots.push_frame();
        for (name, value) in pairs {
            let value = self.text(value);
            self.roots.push_temp(value);
            let object = self.heap.get_mut(table).expect("just built and rooted");
            let Body::Native(native) = &mut object.body else {
                unreachable!("allocated as Body::Native by native_instance")
            };
            native.set_entry(name, value);
        }
        self.roots.pop_frame(frame);
        for site in rest {
            self.annotations.insert(site.clone(), table);
        }
    }

    /// Gives `object` the annotation table `site` names, so that a send to it
    /// needs no way back to the directive that declared it.
    pub(crate) fn attach_annotations(&mut self, object: ObjRef, site: Annotated) {
        let table = self.annotation_table(site);
        let Some(held) = self.heap.get_mut(object) else {
            return;
        };
        if let Body::Native(native) = &mut held.body {
            native.set_annotations(table);
        }
    }

    /// The `StringTable` a receiver's `~annotations` answers, or the refusal
    /// for a receiver that carries none.
    pub(crate) fn annotations_of(&mut self, receiver: ObjRef) -> Result<ObjRef, Failure> {
        if self.is_class_object(receiver) {
            return Ok(self.annotation_table(Annotated::Class(receiver)));
        }
        match self.heap.get(receiver).map(|held| &held.body) {
            Some(Body::Native(native)) => native.annotations(),
            _ => None,
        }
        .ok_or_else(|| Loud::receiver_class("a value that carries no annotations").into())
    }

    /// `Method~scope`: the class the method object was defined at, and `.nil`
    /// for one that was defined at no class.
    pub(crate) fn method_scope(&self, receiver: ObjRef) -> Result<ObjRef, Failure> {
        match self.heap.get(receiver).map(|held| &held.body) {
            Some(Body::Native(native)) => Ok(native.scope().unwrap_or(ObjRef::NIL)),
            _ => Err(Loud::receiver_class("a value that carries no method scope").into()),
        }
    }

    /// `Package~name`'s answer for a package object this crate built, or
    /// `None` for a handle [`Interp::package_object_for`] did not produce.
    pub(crate) fn package_name(&self, package: ObjRef) -> Option<Vec<u8>> {
        Some(match self.which_package(package)? {
            Package::Rexx => crate::LIBRARY_PACKAGE_NAME.to_vec(),
            // A program's own package, answered as the file that program was
            // loaded from -- measured, oracle rc 0: a required file's
            // `.context~package~name` is that file's own path where the
            // requiring program's is its own.
            Package::Program(program) => self.program_display_name(program).to_vec(),
        })
    }

    /// The REXX package's own class table: this crate's native class registry
    /// -- `completeSystemClass` (`memory/Setup.cpp:199`-`:206`) files every
    /// `Setup.cpp` class there -- plus what the shipped `.orx` files' own
    /// `::CLASS` directives installed.
    pub(crate) fn rexx_package_class_table(
        &mut self,
        public_only: bool,
    ) -> Vec<(Box<[u8]>, ObjRef)> {
        let mut entries: Vec<(Box<[u8]>, ObjRef)> = self
            .classes()
            .registered()
            .map(|(name, class)| (name.as_bytes().into(), class))
            .collect();
        for program in self.library_programs.clone() {
            let held = if public_only {
                self.package_public_classes.get(&program)
            } else {
                self.package_classes.get(&program)
            };
            entries.extend(
                held.into_iter()
                    .flatten()
                    .map(|(name, class)| (name.clone(), *class)),
            );
        }
        entries
    }

    /// `PackageClass::findClass` (`classes/PackageClass.cpp:1085`): the whole
    /// search order a package resolves a class name over, from `package`.
    ///
    /// Refuses a name `.local` or `.environment` holds on the oracle and this
    /// crate does not build.
    pub(crate) fn package_find_class(
        &mut self,
        package: Option<ProgramId>,
        upper: &[u8],
    ) -> Result<ObjRef, Failure> {
        if let Some(found) = self.installed_class_of(package, upper) {
            return Ok(found);
        }
        if let Some(found) = self.package_public_class_of(package, upper) {
            return Ok(found);
        }
        let local = self.package_local(match package {
            Some(program) => Package::Program(program),
            None => Package::Rexx,
        });
        if let Some(found) = self.native_map_entry(local, upper) {
            return Ok(found);
        }
        match self.directory_lookup(
            &[EnvScope::Local, EnvScope::Environment],
            upper,
            &env_seam::Access::Resolve,
        )? {
            hash::DirectoryEntry::Found(found) => Ok(found),
            hash::DirectoryEntry::Owed(owner) => Err(Loud::environment_entry(upper, owner).into()),
            hash::DirectoryEntry::Absent => Ok(ObjRef::NIL),
        }
    }

    /// `PackageClass::findPublicClass` (`classes/PackageClass.cpp:1013`) at
    /// the `findPublicClassRexx` stub: this package's own public classes, the
    /// ones it imported, and then the REXX package's.
    pub(crate) fn package_find_public_class(
        &mut self,
        package: Option<ProgramId>,
        upper: &[u8],
    ) -> ObjRef {
        self.package_public_class_of(package, upper)
            .unwrap_or(ObjRef::NIL)
    }

    /// `installedClasses`: the classes `package`'s own directives and
    /// `~addClass` installed, which for the REXX package is the native class
    /// registry plus what the shipped `.orx` files declared.
    fn installed_class_of(&mut self, package: Option<ProgramId>, upper: &[u8]) -> Option<ObjRef> {
        let Some(program) = package else {
            for program in self.library_programs.clone() {
                if let Some(found) = self
                    .package_classes
                    .get(&program)
                    .and_then(|held| held.get(upper))
                {
                    return Some(*found);
                }
            }
            return self.classes().lookup_upper(upper);
        };
        self.package_classes.get(&program)?.get(upper).copied()
    }

    /// `PackageClass::findPublicClass`'s three steps, as an `Option`.
    fn package_public_class_of(
        &mut self,
        package: Option<ProgramId>,
        upper: &[u8],
    ) -> Option<ObjRef> {
        let Some(program) = package else {
            return self.rexx_package_class(upper);
        };
        if let Some(found) = self
            .package_public_classes
            .get(&program)
            .and_then(|held| held.get(upper))
        {
            return Some(*found);
        }
        if let Some(found) = self
            .merged_public_classes
            .get(&program)
            .and_then(|held| held.get(upper))
        {
            return Some(*found);
        }
        self.rexx_package_class(upper)
    }

    /// `PackageClass::findRoutine` (`classes/PackageClass.cpp:897`), as the
    /// `Routine` object or `.nil`.
    pub(crate) fn package_find_routine(
        &mut self,
        package: Option<ProgramId>,
        upper: &[u8],
    ) -> ObjRef {
        let Some(program) = package else {
            return ObjRef::NIL;
        };
        match self.find_routine(program, upper) {
            Some(merged) => self.merged_routine_object(merged).unwrap_or(ObjRef::NIL),
            None => ObjRef::NIL,
        }
    }

    /// `PackageClass::findRoutine` (`classes/PackageClass.cpp:822-911`) for
    /// the upcased `upper`: `findLocalRoutine`, every `routines` table up the
    /// parent chain, and only then `findPublicRoutine`, every merged table up
    /// the chain. A package's public routines are among its `routines`, so the
    /// second walk has only the merged tables left to read.
    pub(crate) fn find_routine(
        &self,
        program: ProgramId,
        upper: &[u8],
    ) -> Option<crate::MergedRoutine> {
        let chain = || {
            // Bounded rather than argued safe: a parent is always a program
            // that already existed when its child was built, so the chain
            // cannot close -- and the bound costs less than that sentence.
            std::iter::successors(Some(program), |child| {
                match self.package_parents.get(child) {
                    Some(crate::plan::Package::Program(parent)) => Some(*parent),
                    _ => None,
                }
            })
            .take(self.programs.len() + 1)
        };
        chain()
            .find_map(|package| self.routines.get(&package)?.get(upper).copied())
            .map(crate::MergedRoutine::Installed)
            .or_else(|| {
                chain().find_map(|package| {
                    self.merged_public_routines
                        .get(&package)?
                        .get(upper)
                        .copied()
                })
            })
    }

    /// The `Routine` object one imported routine is: the `::ROUTINE`'s own,
    /// or for a library routine, the one kept for its library code row,
    /// reporting that code's package.
    pub(crate) fn merged_routine_object(&mut self, merged: crate::MergedRoutine) -> Option<ObjRef> {
        match merged {
            crate::MergedRoutine::Installed(installed) => self.routine_object(installed),
            crate::MergedRoutine::Library(code) => Some(self.library_routine_object(code)),
        }
    }

    /// The one `Routine` object for the library routine at `code`'s row,
    /// which an imported-routine table and `loadExternalRoutine` both answer
    /// (`LibraryPackage::resolveRoutine`, `package/LibraryPackage.cpp:410-432`).
    pub(crate) fn library_routine_object(&mut self, code: usize) -> ObjRef {
        if let Some(found) = self.library_routine_objects.get(&code).copied() {
            return found;
        }
        let class = self.routine_class();
        let object = self.native_instance(class);
        self.roots
            .add_global(&library_routine_root_key(code), object);
        self.library_routine_objects.insert(code, object);
        self.record_loaded_executable(object, code);
        object
    }

    /// The file `name` resolves to from `package`'s own directory --
    /// `PackageClass::resolveProgramName`, which `~findProgram` and
    /// `~loadPackage` each reach with their own resolve type.
    /// **A package compiled from source text has no directory**, and searches
    /// the global context instead of borrowing one. `package_path` answers the
    /// running program's own path for a package with no file, which is right
    /// for `PARSE SOURCE` and a traceback and wrong here: measured with the
    /// same file name in both places, the oracle resolves an in-memory
    /// package's `findProgram` against the **current** directory where this
    /// crate resolved it against the directory of the program that built it.
    ///
    /// `compiled_method_names` is the test because it holds exactly the
    /// programs built from source -- an in-memory package, a `Method` and a
    /// `Routine` compiled from text -- while `new_file_executable`, which does
    /// come from a file, records a path instead.
    pub(crate) fn resolve_program_name(
        &self,
        package: Option<ProgramId>,
        name: &[u8],
        requires: bool,
    ) -> Option<String> {
        let program = package
            .filter(|program| !self.compiled_method_names.contains_key(program))
            .map(|program| self.package_path(program));
        self.resolve_search(program, name, requires)
    }

    /// Which package a package object stands for, or `None` for a handle
    /// [`Interp::package_object`] did not produce.
    pub(crate) fn which_package(&self, package: ObjRef) -> Option<Package> {
        self.package_objects
            .iter()
            .find(|(_, object)| **object == package)
            .map(|(which, _)| *which)
    }

    /// The package a `Method` or `Routine` object passed as a context argument
    /// hands on as the new code's parent, or `None` for an object this crate
    /// did not build.
    ///
    /// A `loadExternal*` answer no directive has bound hands on the `REXX`
    /// package. Its own package is `.nil`, which the oracle hands on as the
    /// parent (`execution/BaseExecutable.cpp:121-125`): measured, it answers
    /// until a name is resolved through that parent, and segfaults there
    /// (`corpus/oracle-crashes.txt` entry 15).
    pub(crate) fn executable_context_package(&self, object: ObjRef) -> Option<Package> {
        let source = self.executable_sources.get(&object)?.source;
        Some(self.source_package(source).unwrap_or(Package::Rexx))
    }
}

/// The [`rexx_core::RootSet::add_global`] key the `Routine` object for one
/// [`Interp::library_codes`] row is held under.
fn library_routine_root_key(code: usize) -> String {
    format!("the imported library routine of code row {code}")
}

/// The [`rexx_core::RootSet::add_global`] key one program's own main-section
/// `Routine` object is held under.
fn program_routine_root_key(program: ProgramId) -> String {
    format!("the main routine of the program {}", program.0)
}

/// The [`rexx_core::RootSet::add_global`] key one annotation table is held
/// under.
fn annotation_root_key(site: &Annotated) -> String {
    match site {
        Annotated::Package(package) => format!("annotations of {}", package_root_key(*package)),
        Annotated::Class(class) => format!("annotations of the class at {}", class.bits()),
        Annotated::Member(class, class_side, name) => format!(
            "annotations of the {} {} of the class at {}",
            if *class_side {
                "class method"
            } else {
                "instance method"
            },
            String::from_utf8_lossy(name),
            class.bits()
        ),
        Annotated::Unattached(ProgramId(program), name) => format!(
            "annotations of the unattached method {} of program {program}",
            String::from_utf8_lossy(name)
        ),
        Annotated::Routine(ProgramId(program), directive) => {
            format!("annotations of the routine at directive {directive} of program {program}")
        }
        Annotated::Compiled(count) => format!("annotations of compiled method {count}"),
    }
}
