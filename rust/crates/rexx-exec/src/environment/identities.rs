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
    ) -> Result<(), Failure> {
        self.invalidate_rexx_class_cache();
        let data = self.program_fields_mut(program)?;
        data.classes.insert(name.to_ascii_uppercase().into(), class);
        if public {
            data.public_classes
                .insert(name.to_ascii_uppercase().into(), class);
        }
        // **A class the interpreter's own library installed belongs to the
        // `REXX` package, not to the program whose directives installed it.**
        // Leaving it out of this table is what `Interp::package_object_for`
        // reads as `Package::Rexx`, which is the oracle's answer: measured
        // on both engines and the oracle, `.Alarm~package~name` is `REXX`.
        // The REXX package's own tables hold it too, the first library
        // program's declaration of a name standing.
        if !self.library_programs.contains(&program) {
            self.class_packages
                .insert(class, ClassPackage::Program(program));
            let package = self.package_object(Package::Program(program))?;
            self.class_holds_package(class, package);
            return Ok(());
        }
        self.roots.activity_mut().push_temp(class);
        let rexx = self.rexx_package_object();
        let Some(data) = self.package_data_mut(rexx) else {
            return Ok(());
        };
        data.classes
            .entry(name.to_ascii_uppercase().into())
            .or_insert(class);
        if public {
            data.public_classes
                .entry(name.to_ascii_uppercase().into())
                .or_insert(class);
        }
        Ok(())
    }

    /// Records a class `Class~subclass` or `Class~mixinClass` built, whose
    /// own `package` field the oracle leaves null
    /// (`classes/ClassClass.cpp:1546`, `:1496`, then `:1582`).
    pub(crate) fn record_packageless_class(&mut self, class: ObjRef) {
        self.class_packages.insert(class, ClassPackage::Null);
    }

    /// Gives `copy` the package `class` answers, which is what a clone of the
    /// class object carries.
    pub(crate) fn copy_class_package(&mut self, class: ObjRef, copy: ObjRef) {
        // The copy does not hold the package: `class`, which is never
        // collected (D59), already does.
        if let Some(package) = self.class_packages.get(&class).copied() {
            self.class_packages.insert(copy, package);
        }
    }

    /// Records that `class` keeps the package it was defined in alive, as
    /// `RexxClass::package` does. A class outside the arena is never
    /// collected and holds nothing.
    fn class_holds_package(&mut self, class: ObjRef, package: ObjRef) {
        if matches!(
            self.heap.get(class).map(|held| &held.body),
            Some(Body::Class { .. })
        ) {
            self.class_owns(class, package);
        }
    }

    /// Whether `class` is one the interpreter's own bootstrap built, which
    /// no program's install recorded -- the oracle's `isPrimitiveClass`.
    pub(crate) fn is_primitive_class(&self, class: ObjRef) -> bool {
        !self.class_packages.contains_key(&class)
    }

    /// `Package~addClass` and `Package~addPublicClass`, which differ only in
    /// whether the public table gets the entry too --
    /// `PackageClass::addInstalledClass` (`classes/PackageClass.cpp:1401`),
    /// which both `addClassRexx` (`:1932`) and `addPublicClassRexx`
    /// (`:1950`) reach with the flag set differently.
    pub(crate) fn add_installed_class(
        &mut self,
        package: ObjRef,
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
        let Some(data) = self.package_data_mut(package) else {
            return;
        };
        data.classes.insert(name.to_ascii_uppercase().into(), class);
        if public {
            data.public_classes
                .insert(name.to_ascii_uppercase().into(), class);
        }
    }

    /// The package object `class~package` answers, or `.nil` for a class that
    /// belongs to no package.
    pub(crate) fn package_object_for(&mut self, class: ObjRef) -> Result<ObjRef, Failure> {
        // A class this crate's own bootstrap registered is in no program's
        // table, and that absence is what `Package::Rexx` names -- the
        // distinction the enum exists to keep out of an `Option`.
        let package = match self.class_packages.get(&class).copied() {
            None => Package::Rexx,
            Some(ClassPackage::Null) => return Ok(ObjRef::NIL),
            Some(ClassPackage::Program(program)) => Package::Program(program),
        };
        self.package_object(package)
    }

    /// The REXX package's object, built on first use. It is a global root, so
    /// [`Interp::package_object`] never refuses it.
    pub(crate) fn rexx_package_object(&mut self) -> ObjRef {
        self.held_package_object(Package::Rexx)
    }

    /// [`Interp::package_object`] for a package that a running activation,
    /// an install in progress or a reachable package's table holds.
    ///
    /// # Panics
    ///
    /// If that package was collected, which is a rooting defect.
    pub(crate) fn held_package_object(&mut self, package: Package) -> ObjRef {
        match self.package_object(package) {
            Ok(object) => object,
            Err(_) => panic!("{package:?}'s package was collected while it was held"),
        }
    }

    /// The one object standing for `package`, built on first use. A freshly
    /// built program package is rooted as a temp of the caller's frame.
    ///
    /// Refuses a package that was built and has been collected: whatever
    /// asked for it is a holder the collector was not shown, and a rebuilt
    /// package would answer with a new identity and none of its state.
    pub(crate) fn package_object(&mut self, package: Package) -> Result<ObjRef, Failure> {
        let built = match package {
            Package::Rexx => self.rexx_package,
            Package::Program(program) => self.program_package(program),
        };
        if let Package::Program(program) = package {
            let loaded = self.programs.get(program).is_some();
            debug_assert!(
                loaded,
                "{package:?}'s package was asked for after its program was freed"
            );
            if !loaded {
                return Err(Loud::unknown_receiver("a package whose program was freed").into());
            }
        }
        if let Some(found) = built {
            let live = self.heap.get(found).is_some();
            debug_assert!(
                live,
                "{package:?}'s package was asked for after it was collected"
            );
            if !live {
                return Err(
                    Loud::unknown_receiver("a package whose object is no longer live").into(),
                );
            }
            return Ok(found);
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
        let mut native = NativeObject::new(package_class, &rendered);
        native.set_package(crate::package_state::PackageData::state(match package {
            Package::Rexx => None,
            Package::Program(program) => Some(program),
        }));
        let object = self.alloc_with(BehaviourId::OBJECT, Body::Native(Box::new(native)));
        // The REXX package lives for the whole run. A program's package lives
        // while something reaches it: an activation of its code
        // (`Interp::package_roots`), a class, `Routine` or `Method` object of
        // its code, or a holder of the object.
        match package {
            Package::Rexx => {
                self.roots.add_global(&package_root_key(package), object);
                self.rexx_package = Some(object);
            }
            Package::Program(program) => {
                self.roots.activity_mut().push_temp(object);
                self.charge_unheld(crate::PACKAGE_BYTES);
                let hold = self.programs.set_package(program, object);
                if let Some(data) = self.package_data_mut(object) {
                    data.hold = hold;
                }
            }
        }
        self.attach_annotations(object, Annotated::Package(package));
        Ok(object)
    }

    /// `Class~method`'s answer: the one `Method` object the dictionary entry
    /// named `name` on `class`'s instance side, or its class side where
    /// `class_side`, has, built on first ask and kept.
    pub(crate) fn method_object(
        &mut self,
        class: ObjRef,
        class_side: bool,
        name: &[u8],
        scope: ObjRef,
        record: crate::ExecutableRecord,
    ) -> Result<ObjRef, Failure> {
        let key = (class, class_side, Box::<[u8]>::from(name));
        if let Some(found) = self.method_objects.get(&key).copied() {
            return Ok(found);
        }
        let method_class = self.method_class();
        let object = self.native_instance(method_class);
        self.record_executable(object, record)?;
        if let Some(method) = record.installed {
            self.executable_objects.entry(method).or_insert(object);
        }
        let held = self.heap.get_mut(object).expect("just allocated");
        let Body::Native(native) = &mut held.body else {
            unreachable!("allocated as Body::Native by native_instance")
        };
        native.set_scope(scope);
        self.class_owns(class, object);
        self.method_objects.insert(key, object);
        // The dictionary entry the caller found, which is what its
        // annotations are keyed by.
        self.attach_annotations(object, Annotated::Member(class, class_side, name.into()));
        Ok(object)
    }

    /// The one `Method` object for the method `name` defined at `scope` --
    /// what `RexxContext~executable` answers from a `::METHOD` context.
    pub(crate) fn method_executable(
        &mut self,
        scope: ObjRef,
        name: &[u8],
    ) -> Result<ObjRef, crate::error::Failure> {
        // A method set on an object with a `.nil` scope has no class to ask.
        let found = if self.heap.is_class(scope) {
            self.classes()
                .own_instance_slot(scope, &String::from_utf8_lossy(name))
        } else {
            None
        };
        match found {
            Some(rexx_classes::MethodSlot::Defined { scope, method }) => {
                let record = crate::ExecutableRecord {
                    source: self.installed_executable_source(method),
                    installed: Some(method),
                    routine: None,
                    owns_installed: false,
                };
                self.method_object(scope, false, name, scope, record)
            }
            _ => Err(crate::Loud::unknown_receiver(
                "a method context whose scope no longer defines it",
            )
            .into()),
        }
    }

    /// What `RexxContext~executable` answers from a method activation
    /// running `body` for a send to `receiver` under `name` at `scope`: the
    /// `Method` object the send ran, which is not always one a class
    /// dictionary reaches by that name. Measured, oracle rc 0, each answers
    /// `Method`: a body `setMethod` or `enhanced` installed, one `run` ran,
    /// and one whose class `define` took the name back from while it ran.
    pub(crate) fn running_method_executable(
        &mut self,
        scope: ObjRef,
        name: &[u8],
        receiver: ObjRef,
        body: crate::InstalledMethodBody,
    ) -> Result<ObjRef, Failure> {
        let runs = |interp: &Self, method: rexx_classes::MethodId| {
            interp.method_bodies.get(&method).is_some_and(|found| {
                found.program == body.program && found.directive == body.directive
            })
        };
        if let Some(Some(entry)) = self.own_method_entry(receiver, name)
            && entry.scope == scope
            && runs(self, entry.method)
        {
            return Ok(entry.executable);
        }
        if self.heap.is_class(scope)
            && let Some(rexx_classes::MethodSlot::Defined { scope, method }) = self
                .classes()
                .own_instance_slot(scope, &String::from_utf8_lossy(name))
            && runs(self, method)
        {
            let record = crate::ExecutableRecord {
                source: self.installed_executable_source(method),
                installed: Some(method),
                routine: None,
                owns_installed: false,
            };
            return self.method_object(scope, false, name, scope, record);
        }
        // A `run`'s activation carries its own object, so what is left is a
        // directive's body a class no longer reaches by this name: the
        // object standing for its identity, the first minted where a copy
        // shares the body.
        let standing = self
            .executable_objects
            .iter()
            .filter(|(method, object)| runs(self, **method) && self.heap.get(**object).is_some())
            .min_by_key(|(method, _)| method.0)
            .map(|(_, object)| *object);
        if let Some(object) = standing {
            return Ok(object);
        }
        let method = self
            .method_bodies
            .keys()
            .copied()
            .filter(|method| runs(self, *method))
            .min_by_key(|method| method.0);
        let method_class = self.method_class();
        let object = self.native_instance(method_class);
        // A body compiled from source text is a synthetic directive, whose
        // object reports on the whole of its program.
        let synthetic = self
            .programs
            .get(body.program)
            .and_then(|program| program.directives.get(body.directive))
            .is_some_and(|directive| directive.clause_span.is_empty());
        let source = match method {
            _ if synthetic => crate::ExecutableSource::Main {
                program: body.program,
            },
            Some(method) => self.installed_executable_source(method),
            None => crate::ExecutableSource::Directive {
                program: body.program,
                directive: body.directive,
            },
        };
        self.record_executable(
            object,
            crate::ExecutableRecord {
                source,
                installed: method,
                routine: None,
                owns_installed: false,
            },
        )?;
        if let Some(method) = method {
            self.executable_objects.insert(method, object);
        }
        let held = self.heap.get_mut(object).expect("just allocated");
        let Body::Native(native) = &mut held.body else {
            unreachable!("allocated as Body::Native by native_instance")
        };
        native.set_scope(scope);
        // A class's own member, or else a table of its own: a `.nil` scope
        // names no class to key one by.
        let site = if self.heap.is_class(scope) {
            let class_side = match self
                .programs
                .get(body.program)
                .and_then(|program| program.directives.get(body.directive))
                .map(|directive| &directive.kind)
            {
                Some(rexx_parse::DirectiveKind::Method(method)) => method.class_method,
                Some(rexx_parse::DirectiveKind::Attribute(attribute)) => attribute.class_method,
                _ => false,
            };
            Annotated::Member(scope, class_side, name.into())
        } else {
            let site = Annotated::Compiled(self.compiled_methods);
            self.compiled_methods += 1;
            site
        };
        self.attach_annotations(object, site);
        #[cfg(test)]
        {
            self.executables_rebuilt += 1;
        }
        Ok(object)
    }

    /// The one `Routine` object standing for `program`'s own main section --
    /// what `RexxContext~executable` answers from a `PROGRAM` or
    /// `INTERNALCALL` context, held by the program's package.
    pub(crate) fn program_routine_object(&mut self, program: ProgramId) -> Result<ObjRef, Failure> {
        if let Some(found) = self
            .program_data(program)
            .and_then(|data| data.main_routine)
        {
            return Ok(found);
        }
        let package = self.package_object(Package::Program(program))?;
        let class = self.routine_class();
        let object = self.native_instance(class);
        if let Some(data) = self.package_data_mut(package) {
            data.main_routine = Some(object);
        }
        self.record_executable(
            object,
            crate::ExecutableRecord {
                source: crate::ExecutableSource::Main { program },
                installed: None,
                routine: None,
                owns_installed: false,
            },
        )?;
        Ok(object)
    }

    /// Files `record` as what the `Method` or `Routine` object `object`
    /// reports on and runs, and makes `object` keep the package its code
    /// belongs to alive, as `BaseExecutable::package` does. May allocate, so
    /// `object` must be rooted.
    pub(crate) fn record_executable(
        &mut self,
        object: ObjRef,
        record: crate::ExecutableRecord,
    ) -> Result<(), Failure> {
        self.executable_sources.insert(object, record);
        let program = match record.source {
            crate::ExecutableSource::Directive { program, .. }
            | crate::ExecutableSource::Main { program }
            | crate::ExecutableSource::External { program } => program,
            crate::ExecutableSource::Native | crate::ExecutableSource::Loaded { .. } => {
                return Ok(());
            }
        };
        let package = self.package_object(Package::Program(program))?;
        match self.heap.get_mut(object).map(|held| &mut held.body) {
            Some(Body::Native(native)) => native.set_executable_package(package),
            Some(Body::Instance { native, .. }) => {
                *native = Some(Box::new(rexx_core::NativeState::Executable(package)));
            }
            _ => {}
        }
        Ok(())
    }

    /// `MethodClass::newScope` (`classes/MethodClass.cpp:183`): the same
    /// method object with `scope` filled in when it had none, and a copy
    /// carrying `scope` when it already had one.
    fn method_new_scope(&mut self, method: ObjRef, scope: ObjRef) -> Option<ObjRef> {
        match self.heap.get(method).map(|held| &held.body) {
            Some(Body::Native(native)) if native.scope().is_none() => {
                let object = self.heap.get_mut(method).expect("read just above");
                let Body::Native(native) = &mut object.body else {
                    unreachable!("matched as Body::Native just above")
                };
                native.set_scope(scope);
                Some(method)
            }
            Some(Body::Native(_)) => self.clone_executable(method, Some(scope)),
            _ => None,
        }
    }

    /// `Method~copy`: `BaseExecutable::copy`
    /// (`execution/BaseExecutable.cpp:58`), a new object reporting on and
    /// running what `original` does, with a copy of its annotations, under a
    /// method identity of its own, so a flag written on one leaves the other
    /// alone. `None` for an object this crate did not build.
    pub(crate) fn copy_method_object(&mut self, original: ObjRef) -> Option<ObjRef> {
        let object = self.clone_executable(original, None)?;
        if let Some(template) = self
            .executable_sources
            .get(&object)
            .and_then(|record| record.installed)
        {
            let copy = self.copy_method_identity(template);
            if let Some(record) = self.executable_sources.get_mut(&object) {
                record.installed = Some(copy);
                record.owns_installed = true;
            }
            self.executable_objects.insert(copy, object);
        }
        let annotations = match self.heap.get(object).map(|held| &held.body) {
            Some(Body::Native(native)) => native.annotations(),
            _ => None,
        };
        if let Some(table) = annotations {
            let caller = self.caller();
            let fresh = self
                .send_message(table, b"COPY", None, &[], caller)
                .ok()
                .flatten()?;
            if let Some(Body::Native(native)) = self.heap.get_mut(object).map(|held| &mut held.body)
            {
                native.set_annotations(fresh);
            }
        }
        Some(object)
    }

    /// The executable records [`Interp::copy_method_object`] carries over,
    /// for `object`, an instance copy of `original`, a `Method` or `Routine`
    /// subclass instance; nothing for any other `original`.
    pub(crate) fn copy_executable_records(&mut self, original: ObjRef, object: ObjRef) {
        let Some(source) = self.executable_sources.get(&original).copied() else {
            return;
        };
        self.executable_sources.insert(object, source);
        if let Some(writes) = self.method_flag_writes.get(&original).copied() {
            self.method_flag_writes.insert(object, writes);
        }
        if let Some(template) = source.installed {
            let copy = self.copy_method_identity(template);
            if let Some(record) = self.executable_sources.get_mut(&object) {
                record.installed = Some(copy);
                record.owns_installed = true;
            }
            self.executable_objects.insert(copy, object);
        }
    }

    /// A new object over a clone of `original`'s native body, carrying
    /// `scope` where one is given, and reporting on and flagged as
    /// `original` is.
    fn clone_executable(&mut self, original: ObjRef, scope: Option<ObjRef>) -> Option<ObjRef> {
        let mut copy = match self.heap.get(original).map(|held| &held.body) {
            Some(Body::Native(native)) => native.clone(),
            _ => return None,
        };
        if let Some(scope) = scope {
            copy.set_scope(scope);
        }
        // The clone is out of the collector's sight until `alloc_with`
        // returns, and `alloc_with` can collect. Every arena handle it
        // carries is one `original` holds too, and the caller keeps
        // `original` live, so nothing the clone reaches can be swept while
        // it is detached.
        let object = self.alloc_with(BehaviourId::OBJECT, Body::Native(copy));
        self.roots.activity_mut().push_temp(object);
        // The copy reports on the same directive the original did, and
        // carries the flag writes the four setters made on it -- both are
        // `RexxObject::copy`'s doing, which duplicates the whole method
        // object including its flag word. Measured, oracle rc 0:
        // `.K2~define("X", .K~method("M"))` then `.K2~method("X")~source`
        // answers `M`'s own body lines, and the same copy taken after
        // `setPrivate` and `setUnguarded` answers `1 0` for `isPrivate` and
        // `isGuarded` where the directive alone answers `0 1`.
        if let Some(source) = self.executable_sources.get(&original).copied() {
            self.executable_sources.insert(object, source);
        }
        if let Some(writes) = self.method_flag_writes.get(&original).copied() {
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
        let (object, method) = self.scoped_method(source, class)?;
        self.classes()
            .define_instance_method(class, &String::from_utf8_lossy(name), method);
        self.hold_method_object(class, false, name, object);
        Some(())
    }

    /// [`Interp::method_new_scope`] with the method identity its answer runs
    /// under, which is what `setMethod`, `run`, `enhanced` and `define`
    /// install: the object's own where `newScope` answers the object itself,
    /// and one [`Interp::copy_method_identity`] mints for a copy. `None`
    /// for an object this crate did not build, since every one it builds
    /// carries an identity.
    pub(crate) fn scoped_method(
        &mut self,
        method: ObjRef,
        scope: ObjRef,
    ) -> Option<(ObjRef, rexx_classes::MethodId)> {
        let template = self.executable_sources.get(&method)?.installed?;
        let object = self.method_new_scope(method, scope)?;
        if object == method {
            return Some((object, template));
        }
        let copy = self.copy_method_identity(template);
        if let Some(record) = self.executable_sources.get_mut(&object) {
            record.installed = Some(copy);
            record.owns_installed = true;
        }
        self.executable_objects.insert(copy, object);
        Some((object, copy))
    }

    /// Mints the identity a `loadExternalMethod` answer over a library
    /// procedure runs under, binding it to that procedure.
    pub(crate) fn bind_loaded_method(
        &mut self,
        object: ObjRef,
        code: usize,
    ) -> rexx_classes::MethodId {
        let method = self.classes().mint_method_id();
        let key = self.library_code_key(code).clone();
        if let Some(library) = self.libraries.get(&key.library).map(std::rc::Rc::clone) {
            self.library_externals.insert(
                method,
                crate::LibraryBinding {
                    method: library.method_index(&key.procedure),
                    library,
                },
            );
        }
        self.defined_library_codes.insert(method, code);
        self.executable_objects.insert(method, object);
        method
    }

    /// `defineClassMethod`: the same shape as [`Interp::define_method_object`]
    /// on the class side.
    pub(crate) fn define_class_method_object(
        &mut self,
        class: ObjRef,
        name: &[u8],
        source: ObjRef,
    ) -> Option<()> {
        let (object, method) = self.scoped_method(source, class)?;
        self.classes()
            .define_class_method(class, &String::from_utf8_lossy(name), method);
        self.hold_method_object(class, true, name, object);
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
        let frame = self.roots.activity_mut().push_frame();
        for (name, source) in entries {
            let name_text = String::from_utf8_lossy(name).into_owned();
            let Some(source) = *source else {
                installed.push((name_text, None));
                continue;
            };
            let (object, _) = self.scoped_method(source, class)?;
            self.roots.activity_mut().push_temp(object);
            let (object, method) = self.scoped_method(object, class)?;
            self.roots.activity_mut().push_temp(object);
            installed.push((name_text, Some(method)));
            objects.push((name.clone(), object));
        }
        self.classes().define_instance_methods(class, &installed);
        for (name, object) in objects {
            self.hold_method_object(class, false, &name, object);
        }
        self.roots.activity_mut().pop_frame(frame);
        Some(())
    }

    /// Make [`Interp::method_object`] answer `object` for this dictionary
    /// entry, and root it the way one this crate built is rooted.
    pub(crate) fn hold_method_object(
        &mut self,
        class: ObjRef,
        class_side: bool,
        name: &[u8],
        object: ObjRef,
    ) {
        self.class_owns(class, object);
        self.method_objects
            .insert((class, class_side, name.into()), object);
    }

    /// Forget the `Method` object this instance dictionary entry answered,
    /// for a `~delete` or a `~define` that took the entry away.
    pub(crate) fn drop_method_object(&mut self, class: ObjRef, name: &[u8]) {
        self.method_objects.remove(&(class, false, name.into()));
    }

    /// The `StringTable` `~annotations` answers for `site`, built empty on
    /// first ask and kept.
    pub(crate) fn annotation_table(&mut self, site: Annotated) -> ObjRef {
        if let Some(found) = self.annotations.get(&site).copied() {
            return found;
        }
        let table = crate::dispatch::hash::new_string_table(self)
            .expect("StringTable is not an abstract class");
        // A class-owned site is held by its class, and a program package's
        // and a compiled method's by the object `attach_annotations` hands it
        // to before anything allocates, so the table dies with its holder.
        // Every other site keeps the global root.
        match site {
            Annotated::Class(owner) | Annotated::Member(owner, _, _) => {
                self.class_owns(owner, table);
            }
            Annotated::Package(Package::Program(_)) => {}
            // Counted afresh for each object, so no later ask names it.
            Annotated::Compiled(_) => return table,
            _ => self.roots.add_global(&annotation_root_key(&site), table),
        }
        self.annotations.insert(site, table);
        table
    }

    /// Drops the global root [`Interp::annotation_table`] gave `site`'s table.
    pub(crate) fn unroot_annotation_table(&mut self, site: &Annotated) {
        self.roots.remove_global(&annotation_root_key(site));
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
        let frame = self.roots.activity_mut().push_frame();
        for (name, value) in pairs {
            let value = self.text(value);
            self.roots.activity_mut().push_temp(value);
            crate::dispatch::hash::directory_put(self, table, name, value)
                .expect("a StringTable accepts a string index");
        }
        self.roots.activity_mut().pop_frame(frame);
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
            Some(Body::Instance { pools, .. }) => self.subclass_annotations(pools),
            _ => None,
        }
        .ok_or_else(|| Loud::receiver_class("a value that carries no annotations").into())
    }

    /// `Method~scope`: the class the method object was defined at, and `.nil`
    /// for one that was defined at no class.
    pub(crate) fn method_scope(&self, receiver: ObjRef) -> Result<ObjRef, Failure> {
        match self.heap.get(receiver).map(|held| &held.body) {
            Some(Body::Native(native)) => Ok(native.scope().unwrap_or(ObjRef::NIL)),
            // A subclass instance is never installed at a class: `define` and
            // `setMethod` refuse it (93.974).
            Some(Body::Instance { pools, .. })
                if self.executable_subclass_annotations(pools).is_some() =>
            {
                Ok(ObjRef::NIL)
            }
            _ => Err(Loud::receiver_class("a value that carries no method scope").into()),
        }
    }

    /// `Package~name`'s answer, or `None` for a value that is not a package.
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

    /// The classes `Package~classes`, or `~publicClasses` where
    /// `public_only`, answers for `package`: its own table, after the native
    /// class registry for the REXX package and a copy of it.
    pub(crate) fn package_class_table(
        &mut self,
        package: ObjRef,
        public_only: bool,
    ) -> Vec<(Box<[u8]>, ObjRef)> {
        let Some(data) = self.package_data(package) else {
            return Vec::new();
        };
        let own: Vec<(Box<[u8]>, ObjRef)> = if public_only {
            &data.public_classes
        } else {
            &data.classes
        }
        .iter()
        .map(|(name, class)| (name.clone(), *class))
        .collect();
        if data.program.is_some() {
            return own;
        }
        // One table, as the oracle's is: a name the package's own fields hold
        // replaces the registry's entry rather than following it.
        let mut entries: Vec<(Box<[u8]>, ObjRef)> = self
            .registered_classes()
            .into_iter()
            .filter(|(name, _)| !own.iter().any(|(held, _)| held == name))
            .collect();
        entries.extend(own);
        entries
    }

    /// This crate's native class registry, which `completeSystemClass`
    /// (`memory/Setup.cpp:199`-`:206`) files every `Setup.cpp` class of the
    /// REXX package's table in. The shipped `.orx` files' own `::CLASS`
    /// directives are in the package's own fields.
    fn registered_classes(&mut self) -> Vec<(Box<[u8]>, ObjRef)> {
        self.classes()
            .registered()
            .map(|(name, class)| (name.as_bytes().into(), class))
            .collect()
    }

    /// `PackageClass::findClass` (`classes/PackageClass.cpp:1085`): the whole
    /// search order a package resolves a class name over, from `package`.
    ///
    /// Refuses a name `.local` or `.environment` holds on the oracle and this
    /// crate does not build.
    pub(crate) fn package_find_class(
        &mut self,
        package: ObjRef,
        upper: &[u8],
    ) -> Result<ObjRef, Failure> {
        if let Some(found) = self.installed_class_of(package, upper)? {
            return Ok(found);
        }
        if let Some(found) = self.package_public_class_of(package, upper)? {
            return Ok(found);
        }
        if let Some(local) = self.package_data(package).and_then(|data| data.local) {
            match self.package_local_entry(local, upper)? {
                hash::DirectoryEntry::Found(found) => return Ok(found),
                hash::DirectoryEntry::Owed(owner) => {
                    return Err(Loud::environment_entry(upper, owner).into());
                }
                hash::DirectoryEntry::Absent => {}
            }
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
        package: ObjRef,
        upper: &[u8],
    ) -> Result<ObjRef, Failure> {
        Ok(self
            .package_public_class_of(package, upper)?
            .unwrap_or(ObjRef::NIL))
    }

    /// Whether `package` is the REXX package or a copy of it, which no
    /// program compiled, and the refusal for a value that is not a package.
    fn is_rexx_package(&self, package: ObjRef) -> Result<bool, Failure> {
        match self.package_data(package) {
            Some(data) => Ok(data.program.is_none()),
            None => Err(Loud::receiver_class("a value that is not a package").into()),
        }
    }

    /// `installedClasses`: the classes `package`'s own directives and
    /// `~addClass` installed, which for the REXX package is the native class
    /// registry plus what the shipped `.orx` files declared.
    fn installed_class_of(
        &mut self,
        package: ObjRef,
        upper: &[u8],
    ) -> Result<Option<ObjRef>, Failure> {
        if self.is_rexx_package(package)? {
            if let Some(found) = self
                .package_data(package)
                .and_then(|data| data.classes.get(upper))
            {
                return Ok(Some(*found));
            }
            return Ok(self.classes().lookup_upper(upper));
        }
        Ok(self.chain_installed_class(Some(package), upper))
    }

    /// `PackageClass::findPublicClass`'s steps: the package's installed public
    /// classes, then its merged ones, then its parent's, then the REXX
    /// package's.
    fn package_public_class_of(
        &mut self,
        package: ObjRef,
        upper: &[u8],
    ) -> Result<Option<ObjRef>, Failure> {
        if self.is_rexx_package(package)? {
            // Its installed public classes are its own fields and the native
            // registry, ahead of what it merged.
            let data = self.package_data(package);
            if let Some(found) = data.and_then(|data| data.public_classes.get(upper)) {
                return Ok(Some(*found));
            }
            let merged = data.and_then(|data| data.merged_public_classes.get(upper).copied());
            return Ok(self.classes().lookup_upper(upper).or(merged));
        }
        if let Some(found) = self.chain_public_class(Some(package), upper) {
            return Ok(Some(found));
        }
        Ok(self.rexx_package_class(upper))
    }

    /// `PackageClass::findRoutine` (`classes/PackageClass.cpp:897`), as the
    /// `Routine` object or `.nil`.
    pub(crate) fn package_find_routine(&mut self, package: ObjRef, upper: &[u8]) -> ObjRef {
        if self.routines_written {
            return match self.find_written_routine(Some(package), upper) {
                Some(crate::FoundRoutine::Merged(merged)) => {
                    self.merged_routine_object(merged).unwrap_or(ObjRef::NIL)
                }
                Some(crate::FoundRoutine::Object(object)) => object,
                None => ObjRef::NIL,
            };
        }
        match self.find_routine(Some(package), upper) {
            Some(merged) => self.merged_routine_object(merged).unwrap_or(ObjRef::NIL),
            None => ObjRef::NIL,
        }
    }

    /// Records `parent` as `program`'s parent package. May allocate `program`'s
    /// package, and so collect.
    pub(crate) fn set_package_parent(
        &mut self,
        program: ProgramId,
        parent: ObjRef,
    ) -> Result<(), Failure> {
        self.roots.activity_mut().push_temp(parent);
        self.program_fields_mut(program)?.parent = Some(parent);
        Ok(())
    }

    /// A class installed in `package` or up its parent chain --
    /// `findInstalledClass` and its recursion to `parentPackage`
    /// (`classes/PackageClass.cpp:982-1003`).
    pub(crate) fn chain_installed_class(
        &self,
        package: Option<ObjRef>,
        upper: &[u8],
    ) -> Option<ObjRef> {
        self.package_chain(package)
            .find_map(|(_, data)| data.classes.get(upper).copied())
    }

    /// A public class of `package` or of one up its parent chain: each
    /// package's installed public classes, then its merged ones
    /// (`classes/PackageClass.cpp:1014-1049`).
    pub(crate) fn chain_public_class(
        &self,
        package: Option<ObjRef>,
        upper: &[u8],
    ) -> Option<ObjRef> {
        self.package_chain(package).find_map(|(_, data)| {
            data.public_classes
                .get(upper)
                .or_else(|| data.merged_public_classes.get(upper))
                .copied()
        })
    }

    /// `PackageClass::findRoutine` (`classes/PackageClass.cpp:822-911`) for
    /// the upcased `upper`: `findLocalRoutine`, every `routines` table up the
    /// parent chain, and only then `findPublicRoutine`, every merged table up
    /// the chain. A package's public routines are among its `routines`, so the
    /// second walk has only the merged tables left to read.
    pub(crate) fn find_routine(
        &self,
        package: Option<ObjRef>,
        upper: &[u8],
    ) -> Option<crate::MergedRoutine> {
        self.package_chain(package)
            .find_map(|(_, data)| data.routines.get(upper).copied())
            .or_else(|| {
                self.package_chain(package)
                    .find_map(|(_, data)| data.merged_public_routines.get(upper).copied())
            })
    }

    /// [`Interp::find_routine`] where a package in the chain is one whose
    /// `.ROUTINES` table a program wrote: that table is read in place of the
    /// package's own routines.
    pub(crate) fn find_written_routine(
        &mut self,
        package: Option<ObjRef>,
        upper: &[u8],
    ) -> Option<crate::FoundRoutine> {
        let chain: Vec<(ObjRef, bool)> = self
            .package_chain(package)
            .map(|(object, data)| (object, data.routines_written))
            .collect();
        for (package, written) in chain.iter().copied() {
            if written {
                let table =
                    self.package_string_table(package, crate::environment::PackageTable::Routines)?;
                if let Some(object) = self.native_entry(table, upper) {
                    return Some(crate::FoundRoutine::Object(object));
                }
            } else if let Some(merged) = self
                .package_data(package)
                .and_then(|data| data.routines.get(upper))
            {
                return Some(crate::FoundRoutine::Merged(*merged));
            }
        }
        chain
            .into_iter()
            .find_map(|(package, _)| {
                self.package_data(package)?
                    .merged_public_routines
                    .get(upper)
                    .copied()
            })
            .map(crate::FoundRoutine::Merged)
    }

    /// What a call runs for a `Routine` object a written `.ROUTINES` table
    /// holds. The oracle casts the entry unchecked
    /// (`classes/PackageClass.cpp:827`), so anything else is refused.
    pub(crate) fn routine_object_resolution(
        &self,
        object: ObjRef,
    ) -> Result<crate::run::Resolved, crate::error::Failure> {
        let record = self.executable_sources.get(&object).copied();
        match record {
            Some(crate::ExecutableRecord {
                routine: Some((program, directive)),
                ..
            }) => Ok(crate::run::Resolved::Routine(crate::InstalledRoutine {
                program,
                directive,
            })),
            Some(crate::ExecutableRecord {
                source: crate::ExecutableSource::Loaded { code },
                installed: None,
                ..
            }) => Ok(crate::run::Resolved::MergedLibraryRoutine(code)),
            _ if let Some(row) = self.rexx_routine_object(object) => {
                Ok(crate::run::Resolved::MergedInternal(row))
            }
            _ => Err(crate::Loud::receiver_class(
                "a .ROUTINES entry that is not a routine this crate built",
            )
            .into()),
        }
    }

    /// The entry `~addRoutine` files `object` under: the `::ROUTINE` it
    /// reports on, the library or internal row it runs where it is the object
    /// that row answers for, and itself over that row otherwise.
    pub(crate) fn added_routine(&self, object: ObjRef) -> Option<crate::MergedRoutine> {
        Some(match self.routine_object_resolution(object).ok()? {
            crate::run::Resolved::Routine(installed) => {
                crate::MergedRoutine::Installed(installed, self.program_package(installed.program)?)
            }
            crate::run::Resolved::MergedLibraryRoutine(code) => {
                if self.library_routine_objects.get(&code) == Some(&object) {
                    crate::MergedRoutine::Library(code)
                } else {
                    crate::MergedRoutine::LibraryObject(code, object)
                }
            }
            crate::run::Resolved::MergedInternal(row) => {
                let index = crate::internal_routines::index_of(row);
                if self.internal_routine_objects[index] == Some(object) {
                    crate::MergedRoutine::Internal(index)
                } else {
                    crate::MergedRoutine::InternalObject(index, object)
                }
            }
            _ => return None,
        })
    }

    /// The `Routine` object one imported routine is: the `::ROUTINE`'s own,
    /// or for a library routine, the one kept for its library code row,
    /// reporting that code's package.
    pub(crate) fn merged_routine_object(&mut self, merged: crate::MergedRoutine) -> Option<ObjRef> {
        match merged {
            crate::MergedRoutine::Installed(installed, _) => self.routine_object(installed),
            crate::MergedRoutine::Library(code) => Some(self.library_routine_object(code)),
            crate::MergedRoutine::Internal(index) => Some(self.internal_routine_object(index)),
            crate::MergedRoutine::LibraryObject(_, object)
            | crate::MergedRoutine::InternalObject(_, object) => Some(object),
        }
    }

    /// The one `Routine` object for the internal package routine at `index`.
    pub(crate) fn internal_routine_object(&mut self, index: usize) -> ObjRef {
        if let Some(found) = self.internal_routine_objects[index] {
            return found;
        }
        let class = self.routine_class();
        let object = self.native_instance(class);
        self.roots
            .add_global(&internal_routine_root_key(index), object);
        self.internal_routine_objects[index] = Some(object);
        self.attach_annotations(object, Annotated::InternalRoutine(index));
        self.record_rexx_routine_object(object, index);
        object
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
        self.attach_annotations(object, Annotated::LibraryRoutine(code));
        object
    }

    /// Builds what a copy of `package` shares with it rather than holding
    /// alone: its program's main `Routine` object and its `.METHODS`,
    /// `.ROUTINES` and `.RESOURCES` tables, each where it has one.
    pub(crate) fn build_package_shares(&mut self, package: ObjRef) -> Result<(), Failure> {
        use crate::environment::PackageTable;
        if let Some(program) = self.package_data(package).and_then(|data| data.program) {
            self.program_routine_object(program)?;
        }
        for kind in [
            PackageTable::UnattachedMethods,
            PackageTable::Routines,
            PackageTable::Resources,
        ] {
            self.package_string_table(package, kind);
        }
        Ok(())
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

    /// The package a `Method` or `Routine` object passed as a context argument
    /// hands on as the new code's parent, or `None` for an object this crate
    /// did not build.
    ///
    /// A `loadExternal*` answer no directive has bound hands on the `REXX`
    /// package. Its own package is `.nil`, which the oracle hands on as the
    /// parent (`execution/BaseExecutable.cpp:121-125`): measured, it answers
    /// until a name is resolved through that parent, and segfaults there
    /// (`corpus/oracle-crashes.txt` entry 15).
    pub(crate) fn executable_context_package(
        &mut self,
        object: ObjRef,
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(record) = self.executable_sources.get(&object) else {
            return Ok(None);
        };
        let package = self.source_package(record.source).unwrap_or(Package::Rexx);
        self.package_object(package).map(Some)
    }
}

/// The [`rexx_core::RootSet::add_global`] key the `Routine` object for one
/// [`Interp::library_codes`] row is held under.
fn library_routine_root_key(code: usize) -> String {
    format!("the imported library routine of code row {code}")
}

/// The [`rexx_core::RootSet::add_global`] key the internal package routine at
/// `index` is held under.
fn internal_routine_root_key(index: usize) -> String {
    format!("the internal package routine of index {index}")
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
        Annotated::Unattached(program, name) => format!(
            "annotations of the unattached method {} of program {program}",
            String::from_utf8_lossy(name)
        ),
        Annotated::Routine(program, directive) => {
            format!("annotations of the routine at directive {directive} of program {program}")
        }
        Annotated::LibraryRoutine(code) => {
            format!("annotations of the library routine at code row {code}")
        }
        Annotated::InternalRoutine(index) => {
            format!("annotations of the internal package routine of index {index}")
        }
        Annotated::Compiled(count) => format!("annotations of compiled method {count}"),
    }
}
