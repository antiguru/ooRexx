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

//! Installing a program's directives: requires and namespaces, classes, methods,
//! attributes and constants, libraries, and the executable records.

use crate::activation::{Activation, CallType, MethodIdentity};
use crate::clause::SavedClauseState;
use crate::directives::{
    AnnotatedSite, FileClasses, annotation_target, attribute_dictionary_keys, class_install_order,
    class_members, directive_clause, directive_gap, member_dictionary_keys, method_dictionary_keys,
    unresolved_external,
};
use crate::error::{Failure, FailureSite, Raised};
use crate::options::PackageOptions;
use crate::plan::{Package, Plan, ProgramId};
use crate::{
    CallContext, Code, ExecutableRecord, ExecutableSource, GeneratedKind, GeneratedMethod,
    InstallBody, InstalledMethodBody, InstalledRoutine, Interp, LIBRARY_PACKAGE_NAME,
    LibraryBinding, LibraryCodeKey, LibraryLoad, Loud, MergedRoutine, Namespace, dispatch,
    environment, internal_routines, library_search_of, plan, require, run,
};
use rexx_classes::{ClassKind, InheritRefusal, MethodId};
use rexx_core::{ObjRef, SlotFrame};
use rexx_parse::{
    Access, AttributeDirective, ClassDirective, ClassRef, CodeBody, ConstantDirective,
    ConstantValue, Directive, DirectiveKind, Expr, GuardOption, MethodDirective, Program,
    Protection,
};
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::rc::Rc;

impl Interp {
    /// Resolves every `::` directive of `program`, filling its package's
    /// routine and class tables and refusing the ones this crate cannot
    /// resolve.
    /// ```text
    /// ::class foo
    /// ::class foo + ::method bar
    /// ::class foo + ::attribute baz
    /// ::constant kk 5
    /// ::resource foo ... ::END
    /// ::annotate package author 'me'
    /// ::annotate <target> <name>, with the target declared above it
    /// a loose ::method with no ::class
    /// ```
    /// ```text
    /// ::class foo subclass zzznotaclass     98.909 rc 158
    /// ::class foo metaclass zzznotaclass    98.908 rc 158
    /// ::class bar inherit zzznotaclass      98.909 rc 158
    /// ::requires 'no_such_file_zz.rex'      43.901 rc 213
    /// ::routine z external "LIBRARY nosuchlib nosuchfn"   98.903 rc 158
    /// ::method m external "LIBRARY nosuchlib nosuchfn"    98.903 rc 158
    /// ::method m external "LIBRARY REXX nosuchentry"      90.998 rc 166
    /// ```
    pub(super) fn install_directives(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
    ) -> Result<(), Failure> {
        // **The oracle's first walk, in source order**: the `::ANNOTATE`
        // targets, the `::OPTIONS` settings and the `EXTERNAL` libraries. The
        // parser has already refused what the declarations alone decide
        // (duplicate names, a misplaced `::CONSTANT` expression, a missing
        // `::ANNOTATE` target). See the table above `directive_gap` for the
        // order this walk is the first of, and for every probe placing a form
        // in it.
        // The class a member directive's keys are claimed against, and the
        // keys claimed so far. `None` is `LanguageParser`'s `unattachedMethods`
        // table, which is one table for the whole file rather than one per
        // class (`parser/DirectiveParser.cpp:518`).
        let mut current_class: Option<usize> = None;
        // Which directive claimed each key, and not merely that one did:
        // `::ANNOTATE ATTRIBUTE` and `::ANNOTATE CONSTANT` accept only a key
        // whose claimant is of the matching kind, which is `isAttribute()`
        // and `isConstant()` on the method object the C++ finds.
        let mut claimed: FxHashMap<(Option<usize>, bool, Vec<u8>), usize> = FxHashMap::default();
        // The declaring directive of each upcased class and routine name,
        // which an `::ANNOTATE CLASS` or `::ANNOTATE ROUTINE` resolves its
        // target against -- `classDependencies` and `routines`, the tables
        // `findClassDirective` and `findRoutine` read
        // (`parser/DirectiveParser.cpp:230`, `:258`).
        let mut declared_classes: FxHashMap<Vec<u8>, usize> = FxHashMap::default();
        let mut declared_routines: FxHashMap<Vec<u8>, usize> = FxHashMap::default();
        // What each `::ANNOTATE` recorded, keyed by what its target names.
        // Filled in this walk and converted below, because a target's own
        // object does not exist yet: a `::CLASS` has no class object until
        // the install pass creates one.
        let mut staged: BTreeMap<AnnotatedSite, Vec<(Box<[u8]>, Box<[u8]>)>> = BTreeMap::new();
        // Every write to a `Directive` target, in `::ANNOTATE` order, which
        // is the order a shared library `Routine` object's table takes them in.
        let mut directive_writes: Vec<(usize, Box<[u8]>, Box<[u8]>)> = Vec::new();
        // The package's routine records, handed over where this walk finishes
        // as `resolveDependencies` hands its tables to the package
        // (`parser/LanguageParser.cpp:1893-1900`), so a translation that raises
        // leaves none.
        let mut routines: FxHashMap<Box<[u8]>, InstalledRoutine> = FxHashMap::default();
        let mut public_routines: FxHashMap<Box<[u8]>, InstalledRoutine> = FxHashMap::default();
        let mut routine_codes: Vec<(InstalledRoutine, usize)> = Vec::new();
        let mut rexx_routines: Vec<(
            InstalledRoutine,
            &'static internal_routines::InternalRoutine,
        )> = Vec::new();
        self.untranslated.insert(id);
        // Built before anything is installed, so that no write below
        // allocates the object its fields live in. A program with no
        // directives writes none, and builds no package until asked.
        if !program.directives.is_empty() {
            self.package_object(Package::Program(id))?;
        }
        for (index, directive) in program.directives.iter().enumerate() {
            // **A synthetic directive installs nothing**, which is what lets
            // `Interp::new_file_executable` file a loaded file's main section
            // as a directive of its own without also declaring it under a
            // name a program could call. The whole set of them is the ones
            // this crate builds, and each carries an empty clause span where
            // a written directive's spans at least `::method x` --
            // `no_written_directive_has_an_empty_clause_span` asserts that
            // over every corpus program, so this is a narrow rule rather than
            // a trap that silently drops a real directive.
            if directive.clause_span.is_empty() {
                continue;
            }
            claim_member_keys(directive, current_class, &mut claimed, index);
            match &directive.kind {
                DirectiveKind::Class(class) => {
                    declared_classes.insert(class.name.to_ascii_uppercase(), index);
                    current_class = Some(index);
                }
                DirectiveKind::Routine(routine) => {
                    // Resolves nothing outside this file and runs nothing:
                    // the body is already assembled in the AST, so installing
                    // it is recording a name.
                    let name: Box<[u8]> = routine.name.to_ascii_uppercase().into();
                    declared_routines.insert(name.to_vec(), index);
                    let installed = InstalledRoutine {
                        program: id,
                        directive: index,
                    };
                    if routine.access == Access::Public {
                        public_routines.insert(name.clone(), installed);
                    }
                    routines.insert(name, installed);
                }
                DirectiveKind::Annotate(annotate) => {
                    let target = match annotation_target(
                        program,
                        &annotate.target,
                        current_class,
                        &claimed,
                        &declared_classes,
                        &declared_routines,
                    ) {
                        Ok(target) => target,
                        Err(missing) => {
                            self.blame_translation(id, program, directive);
                            return Err(Raised::missing_annotation_target(
                                missing.kind,
                                missing.name,
                            )
                            .into());
                        }
                    };
                    // **Accumulative, and the last write to a name wins.**
                    // Each arm of `annotateDirective` reaches for its
                    // target's own table and `processAnnotation` puts into
                    // it (`parser/DirectiveParser.cpp:2259`), so a second
                    // `::ANNOTATE` of one target adds to the first's pairs.
                    // Measured, oracle rc 0: `::annotate class K a 1` beside
                    // `::annotate class K b 2` leaves `~annotations~items` 2,
                    // and `::annotate class K a 1 a 2` leaves it 1 with `A`
                    // answering `2`.
                    for site in target {
                        let pairs = staged.entry(site.clone()).or_default();
                        for annotation in &annotate.annotations {
                            let name = program.symbols.name(annotation.name).as_bytes();
                            pairs.retain(|(held, _)| **held != *name);
                            pairs.push((name.into(), annotation.value.clone()));
                            if let AnnotatedSite::Directive(directive) = site {
                                directive_writes.push((
                                    directive,
                                    name.into(),
                                    annotation.value.clone(),
                                ));
                            }
                        }
                    }
                }
                // **Applied in this walk, so its own refusal is in source
                // order with the rest.** Measured: the 33.1 below wins over a
                // duplicate `::ROUTINE` pair standing after it and loses to
                // one standing before it, and it wins over a `::CLASS` that
                // cannot resolve on either side of it -- the class pass is
                // the second walk. `::OPTIONS` itself never resolves a name
                // and never runs code, so this is the whole of installing it.
                DirectiveKind::Options(options) => {
                    self.options_armed = true;
                    let outcome = {
                        let package = self
                            .package_fields_mut(Package::Program(id))?
                            .options
                            .get_or_insert_default();
                        options.iter().try_for_each(|option| package.apply(option))
                    };
                    if let Err(error) = outcome {
                        self.blame_translation(id, program, directive);
                        return Err(run::raised_from_settings(error).into());
                    }
                    // An arming route of the required-string protocol:
                    // `::OPTIONS NOSTRING SYNTAX` turns a rendering into a
                    // raise exactly as a `NOSTRING` trap does. See
                    // [`Interp::reqstr_armed`] for why this only ever sets.
                    if self
                        .options_of(id)
                        .is_some_and(PackageOptions::escalates_nostring)
                    {
                        self.reqstr_armed = true;
                    }
                    // `Interp::lostdigits_armed`, under the same set-once
                    // rule: the arithmetic path's gate.
                    if self
                        .options_of(id)
                        .is_some_and(PackageOptions::escalates_lostdigits)
                    {
                        self.lostdigits_armed = true;
                    }
                }
                _ => {}
            }

            // **The eager bind** (D37), in this walk because the oracle does
            // it while the directive is being translated:
            // `createNativeMethod` raises from inside `methodDirective`
            // (`parser/DirectiveParser.cpp:1385`), so the file is refused
            // before its own first clause runs. Measured, oracle: a file
            // opening `say "prolog ran"` and carrying one `::METHOD
            // EXTERNAL` on a missing entry point is rc 166 with stdout empty,
            // and the same file naming `file_separator` is rc 0 printing the
            // prologue. Each procedure's shared code is bound to this package
            // as its directive resolves, where nothing bound it first, so a
            // later directive's refusal leaves the binding in place
            // (`createNativeMethod`, `parser/DirectiveParser.cpp:1381-1388`).
            let (keys, internal) = match self.resolve_external(&directive.kind) {
                Ok(resolved) => resolved,
                Err(refusal) => {
                    if refusal.echoes {
                        self.blame_translation(id, program, directive);
                    }
                    return Err(refusal.failure);
                }
            };
            let installed = InstalledRoutine {
                program: id,
                directive: index,
            };
            for key in keys {
                let routine = key.routine;
                let row = self.library_code(key);
                self.library_codes[row].get_or_insert(id);
                if routine {
                    routine_codes.push((installed, row));
                }
            }
            if let Some(row) = internal {
                rexx_routines.push((installed, row));
            }
        }
        if !routines.is_empty() || !public_routines.is_empty() {
            let package = self.package_object(Package::Program(id))?;
            let data = self.program_fields_mut(id)?;
            data.routines.extend(
                routines
                    .into_iter()
                    .map(|(name, installed)| (name, MergedRoutine::Installed(installed, package))),
            );
            data.public_routines.extend(
                public_routines
                    .into_iter()
                    .map(|(name, installed)| (name, MergedRoutine::Installed(installed, package))),
            );
        }
        self.library_routine_codes.extend(routine_codes);
        self.rexx_routine_rows.extend(rexx_routines);
        self.untranslated.remove(&id);
        // Where the translation ends and before any `::REQUIRES` loads, which
        // is where `generateProgram` and `installPackage` run it.
        self.override_package_settings(id)?;
        // `rexxc` stops here. Measured, oracle `rexxc -s`: a `::REQUIRES` of a
        // missing file and a `::CLASS` subclassing an unknown name are rc 0.
        if self.translate_only {
            return Ok(());
        }

        // **A second pass, because the oracle's own translation-time
        // refusals above happen before every install-time one below**
        // (98.9xx/43.901/the `::CONSTANT` expression evaluation), so a
        // program with both gets the translation error -- which is what
        // running the whole first pass before any of this reproduces.
        let mut declared: FxHashMap<Box<[u8]>, usize> = FxHashMap::default();
        for (index, directive) in program.directives.iter().enumerate() {
            if let DirectiveKind::Class(class) = &directive.kind {
                declared
                    .entry(class.name.to_ascii_uppercase().into())
                    .or_insert(index);
            }
        }

        // **The order the file's classes are installed in**, which is not
        // source order once a `SUBCLASS` names a class declared later. See
        // `class_install_order`; a cycle is 98.911 and never reaches the
        // installs below.
        let order = match class_install_order(program, &declared) {
            Ok(order) => order,
            Err(blame) => {
                self.blame_translation(id, program, &program.directives[blame]);
                let path = self.program_display_name(id).to_vec();
                return Err(Raised::cyclic_inheritance(&path).into());
            }
        };

        // A `::REQUIRES` file is opened after the cycle check and before any
        // class is created -- measured, 43.901 against a file whose first
        // directive is a `::CLASS` that fails to resolve, and 98.911 against
        // one whose classes form a cycle. See `directive_gap`.
        self.load_required_packages(id, program)?;

        // **The failing-`::CONSTANT` blame target is the class the oracle
        // installed LAST, not the last one in the file and not the nearest
        // preceding one, and every part of that is measured.** `::class A` /
        // `::constant x (1/0)` / `::class B` blames `B`, and a third
        // `::class C` after it blames `C`, so the blame does not depend on
        // which class the constant is lexically under. `::class b subclass a`
        // / `::constant c (1/0)` / `::class a` blames **`b`**, which source
        // order reaches first, and `::class c subclass b` / `::class a` /
        // `::constant x (1/0)` / `::class b subclass a` blames `c`, first in
        // the file. Which positional rules the corpus excludes, and which
        // witness excludes which, is in `class_install_order`'s own doc.
        // Tracked separately from `class_members` below, which is R9's
        // registry attachment and a genuinely different rule: a `::METHOD` or
        // `::ATTRIBUTE` attaches to the class positionally nearest above it.
        let last_class_directive = order.last().map(|index| &program.directives[*index]);

        // **Which class each `::METHOD`, `::ATTRIBUTE` and `::CONSTANT`
        // attaches to**, taken positionally (R9), so that the pass below can
        // install a class's own members while it is constructing that class.
        let members = class_members(program);

        // **Install walks the class list once per pass, and each pass
        // finishes before the next begins** (`PackageClass::processInstall`):
        // create every class (`classes/PackageClass.cpp:1281`), then resolve
        // every `::CONSTANT` expression (`:1290`), then send `ACTIVATE` to
        // every class (`:1299`). Each pass walks `order` rather than the
        // file, because `processInstall`'s own list is the class list.
        let mut classes: FxHashMap<usize, ObjRef> = FxHashMap::default();
        for index in &order {
            let attached = members.get(index).map_or(&[][..], Vec::as_slice);
            let class =
                self.install_class_at(id, program, *index, &declared, &classes, attached)?;
            classes.insert(*index, class);
            // **After the install and not inside it**, which is where
            // `ClassDirective::install` puts `setAnnotations`
            // (`instructions/ClassDirective.cpp:243`): the class is built and
            // has been sent `INIT` by then. Measured, oracle rc 0: a
            // class-side `init` saying `self~annotation("A")` prints `The NIL
            // object` under an `::ANNOTATE CLASS` that the main body reads
            // back as the annotation's value.
            self.attach_directive_annotations(program, &mut staged, *index, class, attached);
        }

        // What is left names no class object: the package, the file's
        // `::ROUTINE`s, and the method-shaped directives ahead of its first
        // `::CLASS`. `annotation_target` produces a `Directive` key for a
        // `::CLASS` and a `::ROUTINE` alone, and the loop above removed every
        // `::CLASS`'s, because `order` holds every `::CLASS` in the file.
        for (target, pairs) in staged {
            let site = match target {
                AnnotatedSite::Package => environment::Annotated::Package(Package::Program(id)),
                AnnotatedSite::Directive(directive) => {
                    environment::Annotated::Routine(id, directive)
                }
                AnnotatedSite::Member(_, name) => environment::Annotated::Unattached(id, name),
            };
            self.record_annotations(&[site], &pairs);
        }
        // `::ANNOTATE ROUTINE` reaches the one object a library procedure
        // has, so every directive bound to it writes into one table
        // (`parser/DirectiveParser.cpp:2003-2014`).
        for (directive, name, value) in directive_writes {
            let installed = InstalledRoutine {
                program: id,
                directive,
            };
            if let Some(code) = self.library_routine_codes.get(&installed).copied() {
                self.record_annotations(
                    &[environment::Annotated::LibraryRoutine(code)],
                    &[(name, value)],
                );
            } else if let Some(row) = self.rexx_routine_rows.get(&installed).copied() {
                let index = internal_routines::index_of(row);
                self.record_annotations(
                    &[environment::Annotated::InternalRoutine(index)],
                    &[(name, value)],
                );
            }
        }

        // The gap forms whose stage is after the classes are created; see
        // the table above `directive_gap` for the probe behind each. Measured, `::options digits
        // 12` beside a failing `::CLASS` is the oracle's `::CLASS` line, so
        // this walk cannot move ahead of the pass above.
        for directive in &program.directives {
            if let Some(loud) = directive_gap(&directive.kind) {
                return Err(loud.into());
            }
        }

        for index in &order {
            let attached = members.get(index).map_or(&[][..], Vec::as_slice);
            self.resolve_constants(id, program, classes[index], attached, last_class_directive)?;
        }

        for index in &order {
            // `Some` inside this loop by construction: `last_class_directive`
            // is `order.last()` and the loop body runs only for a non-empty
            // `order`.
            let blame = last_class_directive.expect("a non-empty install order has a last class");
            self.send_directive_message(id, program, classes[index], dispatch::ACTIVATE, blame)?;
        }
        Ok(())
    }

    /// The file a package was loaded from: the program's own path, or the
    /// resolved name a `::REQUIRES` found it under.
    pub(crate) fn package_path(&self, id: ProgramId) -> &[u8] {
        match self.required_paths.get(&id) {
            Some(path) => path,
            None => &self.program_path,
        }
    }

    /// The file of the package `method`'s native code reports: the
    /// `EXTERNAL` directive's, or for a method `~define` installed from a
    /// `loadExternalMethod` answer, the package of the first directive that
    /// bound the same code. `None` where there is neither.
    pub(crate) fn external_package_path(&self, method: MethodId) -> Option<Vec<u8>> {
        let program = self.external_package_program(method)?;
        if self.library_programs.contains(&program) {
            return Some(LIBRARY_PACKAGE_NAME.to_vec());
        }
        Some(self.package_path(program).to_vec())
    }

    /// The package [`Interp::external_package_path`] names.
    pub(crate) fn external_package_program(&self, method: MethodId) -> Option<ProgramId> {
        if let Some(code) = self.defined_library_codes.get(&method) {
            return self.library_code_program(*code);
        }
        self.external_packages.get(&method).copied()
    }

    /// Loads every library and package this program's `::REQUIRES` directives
    /// name and merges what each makes public.
    fn load_required_packages(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
    ) -> Result<(), Failure> {
        if !program
            .directives
            .iter()
            .any(|directive| matches!(directive.kind, DirectiveKind::Requires(_)))
        {
            return Ok(());
        }
        self.activity
            .requires_installing
            .push(self.package_path(id).into());
        let outcome = self.install_requires(id, program);
        self.activity.requires_installing.pop();
        outcome
    }

    /// Pushes the level installing `id`, stopped on its directive
    /// `directive`, where `id` has no running activation: the oracle installs
    /// a package in an activation of its own (`RexxActivation::run`,
    /// `RexxActivation.cpp:508`), which the files it requires run above, and
    /// keeps the security manager that activation began under. `None` outside
    /// an install, and where `id` already has a running activation.
    fn push_installing_level(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        directive: usize,
    ) -> Option<(SlotFrame, SavedClauseState)> {
        let call_type = self.install_call?;
        if self.frames().any(|activation| activation.program_id == id) {
            return None;
        }
        let saved_clause = self.save_clause_state();
        let frame = self.roots.push_slots(0);
        let level_id = self.next_activation_id();
        let mut installing = Activation::new(
            level_id,
            Rc::clone(program),
            id,
            Rc::new(Plan::default()),
            frame,
        );
        installing.call_type = call_type;
        installing.body = None;
        installing.cold_mut().directive_clause = Some(directive);
        if let Some(kept) = self.kept_installing_manager(id) {
            installing.cold_mut().security_manager = Some(kept);
        }
        self.push_activation(installing);
        self.record_call_convention();
        self.enter_directive_clause(directive_clause(program, &program.directives[directive]).0);
        Some((frame, saved_clause))
    }

    /// Pops what [`Interp::push_installing_level`] pushed.
    fn pop_installing_level(&mut self, (frame, saved_clause): (SlotFrame, SavedClauseState)) {
        self.pop_activation();
        self.roots.activity_mut().pop_slots(frame);
        self.restore_clause_state(saved_clause);
    }

    /// [`Interp::load_required_packages`]' walk: every `::REQUIRES ... LIBRARY`
    /// in source order, then every other `::REQUIRES` in source order, as
    /// `PackageClass::processInstall` installs them
    /// (`classes/PackageClass.cpp:1227-1260`), stopping at the first that
    /// fails.
    fn install_requires(&mut self, id: ProgramId, program: &Rc<Program>) -> Result<(), Failure> {
        for library in [true, false] {
            for (index, directive) in program.directives.iter().enumerate() {
                let DirectiveKind::Requires(requires) = &directive.kind else {
                    continue;
                };
                if requires.library != library {
                    continue;
                }
                if library {
                    if let Some(package) = internal_routines::Package::named(&requires.name) {
                        self.merge_internal_package(id, package)?;
                        continue;
                    }
                    match self.require_library(&requires.name) {
                        Ok(loaded) => self.merge_library(id, &requires.name, &loaded)?,
                        Err(failure) => {
                            self.seal_site_level();
                            self.blame_directive(id, program, directive);
                            return Err(failure);
                        }
                    }
                    continue;
                }
                let level = self.push_installing_level(id, program, index);
                let loaded = self.load_requires(Some(id), &requires.name, true);
                if let Some(level) = level {
                    self.pop_installing_level(level);
                }
                match loaded {
                    Ok(required) => {
                        let into = self.package_object(Package::Program(id))?;
                        let from = self.package_object(Package::Program(required))?;
                        self.add_imported_package(into, from);
                        self.merge_package(into, from);
                        // `RequiresDirective::install`
                        // (`instructions/RequiresDirective.cpp:137`): the
                        // registration is what the directive does *after* the
                        // load and the merge, so a namespace neither narrows
                        // the merge nor replaces it.
                        if let Some(namespace) = requires.namespace {
                            let name = program.symbols.name(namespace).as_bytes().into();
                            self.program_fields_mut(id)?.namespaces.insert(name, from);
                        }
                    }
                    Err(failure) => {
                        self.seal_site_level();
                        self.blame_directive(id, program, directive);
                        return Err(failure);
                    }
                }
            }
        }
        Ok(())
    }

    /// The package `name` names, loaded and its prologue run if this is the
    /// first `::REQUIRES` to reach it.
    ///
    /// `from` is the package whose directory and extension the search starts
    /// with, and `None` searches the **global** context instead -- no parent
    /// directory, no parent extension. That is what `Package~new(name)` uses:
    /// measured, it does not look beside its caller, so a file the caller sits
    /// next to is 43.901 there where a `::REQUIRES` of the same name finds it.
    ///
    /// The cache is consulted either way and keyed by the name as written, so
    /// a global lookup still hits what a `::REQUIRES` loaded earlier --
    /// measured, `Package~new` after a `loadPackage` of the same short name
    /// answers the package already loaded.
    ///
    /// The package is rooted as a temp of the caller's frame: on a cache hit
    /// the weak cache may be all that reaches it.
    ///
    /// `directive` is a `::REQUIRES` of `from` being installed, which asks
    /// `from`'s own manager rather than the running activation's.
    fn load_requires(
        &mut self,
        from: Option<ProgramId>,
        name: &[u8],
        directive: bool,
    ) -> Result<ProgramId, Failure> {
        let loaded = self.find_requires(from, name, directive)?;
        if let Some(package) = self.program_package(loaded) {
            self.roots.activity_mut().push_temp(package);
        }
        Ok(loaded)
    }

    /// [`Interp::load_requires`]'s lookup and load.
    fn find_requires(
        &mut self,
        from: Option<ProgramId>,
        name: &[u8],
        directive: bool,
    ) -> Result<ProgramId, Failure> {
        // The name as written, and the file it resolves to, are looked up
        // before the manager is asked, and a hit asks it nothing:
        // `InterpreterInstance::loadRequires`
        // (`runtime/InterpreterInstance.cpp:1021`-`:1045`), whose callers
        // resolve first (`classes/PackageClass.cpp:1319`). Then the manager
        // sees the short name, then the resolved one
        // (`package/PackageManager.cpp:718`-`:766`). Its short-name answer is
        // only looked up, and the search, a miss and the cache entries use
        // the original.
        let original = name;
        if let Some(&loaded) = self.required_packages.get(original) {
            self.check_not_installing(loaded)?;
            return Ok(loaded);
        }
        let found = match from {
            Some(id) => self.resolve_requires(id, original),
            None => self.resolve_search(None, original, true),
        };
        if let Some(found) = &found
            && let Some(&loaded) = self.required_packages.get(&found[..])
        {
            self.check_not_installing(loaded)?;
            self.required_packages.insert(original.into(), loaded);
            return Ok(loaded);
        }
        let manager = match from {
            Some(id) if directive => self.installing_security_manager(id),
            _ => self.effective_security_manager(),
        };
        let mut inherited = None;
        let Some(name) = self.check_requires_access(manager, name, &mut inherited)? else {
            return Err(Raised::requires_file_not_found(original).into());
        };
        let name = &name[..];
        if let Some(&loaded) = self.required_packages.get(name) {
            self.check_not_installing(loaded)?;
            return Ok(loaded);
        }
        let resolved = match found.clone() {
            Some(resolved) => {
                let Some(checked) =
                    self.check_requires_access(manager, &resolved, &mut inherited)?
                else {
                    return Err(Raised::requires_file_not_found(original).into());
                };
                Some(checked.to_vec())
            }
            None => None,
        };
        if let Some(resolved) = &resolved
            && let Some(&loaded) = self.required_packages.get(&resolved[..])
        {
            self.check_not_installing(loaded)?;
            self.required_packages.insert(original.into(), loaded);
            return Ok(loaded);
        }
        let Some(resolved) = resolved else {
            return Err(Raised::requires_file_not_found(original).into());
        };
        // `getRequiresFile` translates the name it was given, the manager's
        // replacement included, and an unreadable one is 3.1 naming it
        // (`LanguageParser::createPackage`).
        let Ok(text) = std::fs::read(OsStr::from_bytes(&resolved)) else {
            return Err(Raised::executable_file_unreadable(&resolved).into());
        };
        let parsed = Rc::new(self.parse_file(text, &resolved)?);
        let required = self.load_program(Rc::clone(&parsed));
        self.required_paths
            .insert(required, resolved.as_slice().into());
        // Built before it is cached: the cache holds the package object
        // weakly (`PackageManager::addToRequiresCache`,
        // `package/PackageManager.cpp:921`), so an entry lives exactly as long
        // as its object.
        let package = self.package_object(Package::Program(required))?;
        // Installed before the prologue runs, which is where
        // `getRequiresFile` puts it (`package/PackageManager.cpp:830`-`:833`).
        if let Some(manager) = inherited {
            self.install_security_manager(package, Some(manager));
        }
        // **Cached before the prologue runs, not after**, which is
        // `addRequiresFile` standing ahead of `runProlog` at
        // `InterpreterInstance.cpp:1060`: a name reached again from inside
        // that prologue must find this entry, or the circularity check has
        // nothing to fire on.
        self.required_packages.insert(original.into(), required);
        self.required_packages
            .insert(resolved.as_slice().into(), required);
        if let Some(found) = &found {
            self.required_packages
                .insert(found.as_slice().into(), required);
        }
        // A `::REQUIRES` prologue runs in the requiring package's installing
        // activation, which starts from that package's `::OPTIONS` and, under
        // `NUMERIC INHERIT`, from the caller of its install. A run-time load
        // (`.Package~new(name)`, `loadPackage`) runs in the running
        // activation. Measured: a `DIGITS 4` package requiring an inheriting
        // file reports `digits()` 4 in that file's prologue, and so do both
        // run-time loads under `NUMERIC DIGITS 4`.
        let requirer = match (from, &self.install_caller) {
            (Some(from), Some(caller)) => Some(self.start_settings(from, caller.as_ref())),
            _ => self
                .running_activation()
                .map(|activation| activation.settings.clone()),
        };
        if let Err(failure) = self.run_loaded(parsed, required, CallType::Requires, None, requirer)
        {
            // `getRequiresFile` caches a package only once it has translated
            // (`package/PackageManager.cpp:828-836`), so a later ask for one
            // whose translation raised translates it again; a package that
            // failed to install or in its prologue stays cached.
            if self.untranslated.contains(&required) {
                self.required_packages.remove(original);
                self.required_packages.remove(&resolved[..]);
                if let Some(found) = &found {
                    self.required_packages.remove(&found[..]);
                }
            }
            return Err(failure);
        }
        Ok(required)
    }

    /// The security manager's `REQUIRES` checkpoint
    /// (`execution/SecurityManager.cpp:318`-`:346`).
    ///
    /// Answers the name to load under -- `manager`'s own replacement where
    /// it made one -- or `None` where it removed the entry, which forbids the
    /// load. `inherited` collects the `SECURITYMANAGER` the new package is to
    /// carry, across both of a load's two checks.
    fn check_requires_access(
        &mut self,
        manager: Option<ObjRef>,
        name: &[u8],
        inherited: &mut Option<ObjRef>,
    ) -> Result<Option<Vec<u8>>, Failure> {
        if manager.is_none() {
            return Ok(Some(name.to_vec()));
        }
        let requested = self.text(name);
        self.roots.activity_mut().push_temp(requested);
        let entries = [(crate::security::key::NAME, requested)];
        let Some(info) =
            self.security_check_by(manager, crate::security::message::REQUIRES, &entries)?
        else {
            return Ok(Some(name.to_vec()));
        };
        // `secObject != TheNilObject` (`SecurityManager.cpp:339`).
        if let Some(manager) = self.security_entry(info, crate::security::key::SECURITYMANAGER)?
            && manager != ObjRef::NIL
        {
            *inherited = Some(manager);
        }
        let Some(replaced) = self.security_entry(info, crate::security::key::NAME)? else {
            return Ok(None);
        };
        self.roots.activity_mut().push_temp(replaced);
        let text = self.required_string_value(replaced)?;
        Ok(Some(self.to_text(text).into_owned()))
    }

    /// 98.952 when `loaded`'s own `::REQUIRES` directives are still
    /// installing, and `Ok` otherwise -- `Activity::checkRequires`
    /// (`concurrency/Activity.cpp:3702`).
    fn check_not_installing(&self, loaded: ProgramId) -> Result<(), Failure> {
        let path = self.package_path(loaded);
        if self
            .activity
            .requires_installing
            .iter()
            .any(|open| **open == *path)
        {
            return Err(Raised::circular_requires(path).into());
        }
        Ok(())
    }

    /// `PackageClass::addPackage` (`classes/PackageClass.cpp:1367`): records
    /// `from` as one of `into`'s imports, once.
    pub(super) fn add_imported_package(&mut self, into: ObjRef, from: ObjRef) -> bool {
        let Some(data) = self.package_data_mut(into) else {
            return false;
        };
        if data.imports.contains(&from) {
            return false;
        }
        data.imports.push(from);
        true
    }

    /// The public routines and classes `from` contributes to `into`: its own
    /// first, then the ones it imported. The REXX package's own public
    /// classes are its whole class table's.
    pub(super) fn merge_package(&mut self, into: ObjRef, from: ObjRef) {
        let Some(data) = self.package_data(from) else {
            return;
        };
        let routines: Vec<(Box<[u8]>, MergedRoutine)> = data
            .public_routines
            .iter()
            .chain(&data.merged_public_routines)
            .map(|(name, merged)| (name.clone(), *merged))
            .collect();
        let imported: Vec<(Box<[u8]>, ObjRef)> = data
            .merged_public_classes
            .iter()
            .map(|(name, class)| (name.clone(), *class))
            .collect();
        let mut classes = self.package_class_table(from, true);
        classes.extend(imported);
        self.merge_routines(into, routines);
        self.invalidate_rexx_class_cache();
        let Some(data) = self.package_data_mut(into) else {
            return;
        };
        for (name, class) in classes {
            data.merged_public_classes.entry(name).or_insert(class);
        }
    }

    /// The package a namespace qualifier written in `package` names, or `None`
    /// when nothing registered it.
    fn find_namespace(&self, package: ProgramId, upper: &[u8]) -> Option<Namespace> {
        if upper == LIBRARY_PACKAGE_NAME {
            return Some(Namespace::Rexx);
        }
        let found = *self.program_data(package)?.namespaces.get(upper)?;
        Some(match self.package_data(found)?.program {
            None => Namespace::Rexx,
            Some(_) => Namespace::Package(found),
        })
    }

    /// The class `namespace:name` names from `package`, or the oracle's own
    /// refusal for either half missing.
    pub(super) fn namespace_class(
        &mut self,
        package: ProgramId,
        namespace: &[u8],
        name: &[u8],
    ) -> Result<ObjRef, Failure> {
        let Some(target) = self.find_namespace(package, namespace) else {
            let path = self.package_path(package).to_vec();
            return Err(Raised::namespace_not_found(namespace, &path).into());
        };
        let found = match target {
            Namespace::Rexx => self.rexx_package_class(name),
            Namespace::Package(found) => self.public_class_of(found, name),
        };
        found.ok_or_else(|| Raised::namespace_class_not_found(name, namespace).into())
    }

    /// `findPublicClass` for one package: its own `::CLASS ... PUBLIC`
    /// declarations, then the ones it imported (`classes/PackageClass.cpp:760`
    /// region). Measured, oracle rc 0: a class a *required* file of the
    /// namespace package declares public is reachable through the qualifier.
    fn public_class_of(&self, package: ObjRef, upper: &[u8]) -> Option<ObjRef> {
        let data = self.package_data(package)?;
        data.public_classes
            .get(upper)
            .or_else(|| data.merged_public_classes.get(upper))
            .copied()
    }

    /// The routine `namespace:name` names from `package`, or the oracle's own
    /// refusal for either half missing.
    pub(super) fn namespace_routine(
        &self,
        package: ProgramId,
        namespace: &[u8],
        name: &[u8],
    ) -> Result<run::Resolved, Failure> {
        let Some(target) = self.find_namespace(package, namespace) else {
            let path = self.package_path(package).to_vec();
            return Err(Raised::namespace_not_found(namespace, &path).into());
        };
        let found = match target {
            Namespace::Rexx => None,
            Namespace::Package(found) => self.package_data(found).and_then(|data| {
                data.public_routines
                    .get(name)
                    .or_else(|| data.merged_public_routines.get(name))
                    .map(|merged| merged.resolved())
            }),
        };
        found.ok_or_else(|| Raised::namespace_routine_not_found(name, namespace).into())
    }

    /// The file a `::REQUIRES` of `name` in package `id` resolves to, or
    /// `None` when no route holds one.
    fn resolve_requires(&self, id: ProgramId, name: &[u8]) -> Option<Vec<u8>> {
        self.resolve_in_package(id, name, true)
    }

    /// The file `name` resolves to from package `id`, or `None` when no route
    /// holds one: `PackageClass::resolveProgramName`
    /// (`classes/PackageClass.cpp:926`), searched from the directory and
    /// extension of the name each package was given, its own first and then
    /// each parent's. The REXX package has no name and searches the global
    /// context.
    pub(crate) fn resolve_in_package(
        &self,
        id: ProgramId,
        name: &[u8],
        requires: bool,
    ) -> Option<Vec<u8>> {
        let Some(package) = self.program_package(id) else {
            return self.resolve_search(Some(self.program_display_name(id)), name, requires);
        };
        self.package_chain(Some(package)).find_map(|(_, data)| {
            let program = data
                .program
                .map(|program| self.program_display_name(program));
            self.resolve_search(program, name, requires)
        })
    }

    /// [`Interp::resolve_requires`] for a caller that names the searching
    /// package's path itself and chooses the resolve type.
    pub(crate) fn resolve_search(
        &self,
        program: Option<&[u8]>,
        name: &[u8],
        requires: bool,
    ) -> Option<Vec<u8>> {
        // The interpreter's own environment and directory, never the process's
        // (`Interp::env`, `Interp::cwd`): the harnesses run interpreters on
        // threads, and `ootest/testOORexx.rex` sets `PATH` through `VALUE` and
        // changes directory before calling the program it then has to find.
        let variable = |wanted: &[u8]| {
            self.env
                .iter()
                .find(|(held, _)| held == wanted)
                .map(|(_, value)| value.as_slice())
        };
        require::search(
            program,
            name,
            variable(b"REXX_PATH"),
            variable(b"PATH"),
            self.cwd_bytes(),
            requires,
        )
    }

    /// `PackageClass::loadPackageRexx`'s load: the same
    /// [`Interp::load_requires`] a `::REQUIRES` performs, which is what the
    /// C++ calls too (`classes/PackageClass.cpp:1842`).
    pub(crate) fn load_package(
        &mut self,
        program: Option<ProgramId>,
        name: &[u8],
    ) -> Result<ProgramId, Failure> {
        self.load_requires(program, name, false)
    }

    /// `PackageClass::newRexx`'s load: the same file load, searched from the
    /// **global** context rather than from any package's directory.
    pub(crate) fn load_package_global(&mut self, name: &[u8]) -> Result<ProgramId, Failure> {
        self.load_requires(None, name, false)
    }

    /// `PackageClass::newRexx`'s in-memory form: a package compiled from
    /// source lines, its directives installed and its prologue run.
    ///
    /// `parent` is the package its names resolve through after its own, and
    /// `None` leaves it none at all -- measured, a source package built with
    /// no context raises 43.1 for a routine its caller defines.
    pub(crate) fn package_from_source(
        &mut self,
        name: &[u8],
        lines: &[Vec<u8>],
        parent: Option<ObjRef>,
    ) -> Result<ProgramId, Failure> {
        let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
        let source = match crate::compiled::lines_source(&borrowed, name) {
            Some(restored) => restored?,
            None => rexx_parse::ProgramSource::from_lines(&borrowed),
        };
        // A restored package inherits no context: measured, its routine
        // calling one the caller defines raises 43.1.
        let parent = parent.filter(|_| source.is_traceable());
        let parsed = match rexx_parse::program_from(source) {
            Ok(parsed) => Rc::new(parsed),
            Err(rejected) => return Err(self.raise_parse_failure(&rejected, Some(name))),
        };
        self.package_from_program(name, parsed, parent)
    }

    /// [`Interp::package_from_source`] past its parse: `parsed` loaded as a
    /// package, its directives installed and its prologue run.
    pub(crate) fn package_from_program(
        &mut self,
        name: &[u8],
        parsed: Rc<Program>,
        parent: Option<ObjRef>,
    ) -> Result<ProgramId, Failure> {
        let id = self.load_program(Rc::clone(&parsed));
        self.compiled_method_names.insert(id, name.into());
        // Recorded ahead of the prologue for the reason
        // `new_file_executable` records its own parent early: the prologue
        // resolves routines, and must already reach the context.
        // Built before the prologue, rooted as a temp of this frame: a package
        // the prologue builds would otherwise go with the prologue's frame.
        self.package_object(Package::Program(id))?;
        if let Some(parent) = parent {
            self.set_package_parent(id, parent)?;
        }
        // Its prologue runs in the running activation, which `NUMERIC
        // INHERIT` inherits from. Measured: `.Package~new(name, source)` of an
        // inheriting source under `NUMERIC DIGITS 4` reports `digits()` 4.
        let running = self
            .running_activation()
            .map(|activation| activation.settings.clone());
        self.run_loaded(parsed, id, CallType::Requires, None, running)?;
        Ok(id)
    }

    /// Moves what the first walk recorded for one `::CLASS` and for the
    /// members attached to it onto the class object that has just been built.
    fn attach_directive_annotations(
        &mut self,
        program: &Rc<Program>,
        staged: &mut BTreeMap<AnnotatedSite, Vec<(Box<[u8]>, Box<[u8]>)>>,
        index: usize,
        class: ObjRef,
        attached: &[usize],
    ) {
        if let Some(pairs) = staged.remove(&AnnotatedSite::Directive(index)) {
            self.record_annotations(&[environment::Annotated::Class(class)], &pairs);
        }
        for &member in attached {
            // The sides one name is filed under, which is more than one for a
            // `::CONSTANT` and is why the keys are collected before the table
            // is built: both sides must answer one table.
            // Ordered for [`AnnotatedSite`]'s reason: the two halves of an
            // accessor pair are two names of one directive, and which of
            // them gets its table first must not depend on a hash seed.
            let mut sides: BTreeMap<Vec<u8>, Vec<bool>> = BTreeMap::new();
            for (name, class_side) in member_dictionary_keys(&program.directives[member].kind) {
                sides.entry(name).or_default().push(class_side);
            }
            for (name, sides) in sides {
                let key = AnnotatedSite::Member(member, name.clone().into());
                let Some(pairs) = staged.remove(&key) else {
                    continue;
                };
                let keys: Vec<environment::Annotated> = sides
                    .into_iter()
                    .map(|class_side| {
                        environment::Annotated::Member(class, class_side, name.clone().into())
                    })
                    .collect();
                self.record_annotations(&keys, &pairs);
            }
        }
    }

    /// Records the value of every literal `::CONSTANT` among `attached`.
    fn record_literal_constants(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        attached: &[usize],
    ) {
        for &index in attached {
            let DirectiveKind::Constant(constant) = &program.directives[index].kind else {
                continue;
            };
            let value = match &constant.value {
                ConstantValue::Name => self.literal(&constant.name),
                ConstantValue::Text(text) => self.literal(text),
                ConstantValue::Expression(_) => continue,
            };
            self.record_constant_value(id, index, value);
        }
    }

    /// Evaluates the `::CONSTANT` expressions among `attached`, in source
    /// order, and records what each answered.
    fn resolve_constants(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        class: ObjRef,
        attached: &[usize],
        last_class: Option<&Directive>,
    ) -> Result<(), Failure> {
        for &index in attached {
            let directive = &program.directives[index];
            let DirectiveKind::Constant(constant) = &directive.kind else {
                continue;
            };
            let ConstantValue::Expression(expr) = &constant.value else {
                continue;
            };
            let level = last_class.and_then(|class| {
                program
                    .directives
                    .iter()
                    .position(|declared| std::ptr::eq(declared, class))
            });
            let run = ConstantRun {
                name: b"::CONSTANT",
                receiver: class,
                scope: class,
                executable: None,
                arguments: &[],
            };
            match self.eval_constant_expression(id, program, run, index, expr, level) {
                Ok(value) => self.record_constant_value(id, index, value),
                Err(failure) => {
                    // Two clause echoes, innermost first, matching the
                    // oracle's own report exactly (measured, `::class K` /
                    // `::constant c (1/0)`):
                    // ```text
                    //      4 *-* ::constant c (1/0)
                    //      3 *-* ::class K
                    // ```
                    // ```text
                    //      5 *-* return 1/0
                    //      6 *-* ::constant c (self~m)
                    //      3 *-* ::class K
                    // ```
                    // `ConstantDirective`'s method runs as `::CONSTANT`.
                    if !self.seal_constant_level(&failure, id, program, index, b"::CONSTANT", &[]) {
                        return Err(failure);
                    }
                    self.blame_directive(
                        id,
                        program,
                        last_class.expect(
                            "the first pass already refused an expression with no \
                             preceding ::CLASS",
                        ),
                    );
                    return Err(failure);
                }
            }
        }
        Ok(())
    }

    /// Records what a `::CONSTANT` accessor answers, which its package holds.
    fn record_constant_value(&mut self, program: ProgramId, directive: usize, value: ObjRef) {
        if self.program_package(program).is_none() {
            self.roots.activity_mut().push_temp(value);
        }
        let package = self.held_package_object(Package::Program(program));
        if let Some(data) = self.package_data_mut(package) {
            data.constants.insert(directive, value);
        }
    }

    /// The value a `::CONSTANT` accessor answers, or `None` for an expression
    /// form whose pass has not run yet.
    pub(crate) fn constant_value(&self, generated: GeneratedMethod) -> Option<ObjRef> {
        self.program_data(generated.program)?
            .constants
            .get(&generated.directive)
            .copied()
    }

    /// The constant's name as its accessor was installed under, which is what
    /// a 97.4 report names.
    pub(crate) fn constant_name(&self, generated: GeneratedMethod) -> Result<Vec<u8>, Failure> {
        // `get` rather than an index, and a refusal rather than a panic, for
        // the reason `Interp::begin_method`'s own reads carry.
        let Some(directive) = self
            .programs
            .get(generated.program)
            .and_then(|program| program.directives.get(generated.directive))
        else {
            return Err(Loud::missing_body().into());
        };
        let DirectiveKind::Constant(constant) = &directive.kind else {
            return Err(Loud::missing_body().into());
        };
        Ok(constant.name.to_ascii_uppercase())
    }

    /// One message the install machinery sends a class object, run in a
    /// throwaway activation carrying the installing program.
    fn send_directive_message(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        class: ObjRef,
        name: &[u8],
        blame: &Directive,
    ) -> Result<(), Failure> {
        let frame = self.push_directive_activation(id, program);
        let caller = self.caller();
        let sent = self.send_message(class, name, None, &[], caller);
        self.pop_directive_activation(frame);
        let Err(failure) = sent else {
            return Ok(());
        };
        self.seal_site_level();
        self.blame_directive(id, program, blame);
        Err(failure)
    }

    /// Pushes the activation an install-time evaluation or send runs in, and
    /// answers the frame [`Interp::pop_directive_activation`] takes back.
    fn push_directive_activation(&mut self, id: ProgramId, program: &Rc<Program>) -> SlotFrame {
        let frame = self.roots.push_slots(0);
        let activation_id = self.next_activation_id();
        self.push_activation(Activation::new(
            activation_id,
            Rc::clone(program),
            id,
            Rc::new(Plan::default()),
            frame,
        ));
        frame
    }

    /// Tears down what [`Interp::push_directive_activation`] pushed.
    fn pop_directive_activation(&mut self, frame: SlotFrame) {
        self.pop_activation();
        self.roots.activity_mut().pop_slots(frame);
    }

    /// `::CLASS`'s own R9 install: a class object in [`Interp::classes`],
    /// carrying the name as written, and an entry in the running package's own
    /// class table under the uppercased one.
    fn install_class(
        &mut self,
        program: ProgramId,
        class: &ClassDirective,
        superclass: ObjRef,
        metaclass: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let name = String::from_utf8_lossy(&class.name).into_owned();
        // `define_unregistered_class`, not `define_class`: an installed class
        // does not go into `.environment`, and registering it there would let
        // `::class array` displace the environment's own `Array` for every
        // later lookup rather than only for this package's.
        let kind = if class.mixin {
            ClassKind::Mixin
        } else {
            ClassKind::Regular
        };
        let id = dispatch::new_subclass(self, superclass, name.as_bytes(), metaclass, kind)?;
        // **A class the interpreter's own library declares is a class in the
        // image**, and `RexxClass::liveGeneral` sets `REXX_DEFINED` on every
        // class in the image under `PREPARINGIMAGE` (`ClassClass.cpp:136`-
        // `:142`). Measured: the oracle refuses `.Alarm~inherit(.Comparable)`
        // with 98.985, the same refusal it gives `.Array~inherit()`.
        if self.library_bootstrap {
            self.classes().set_rexx_defined(id);
        }
        // The package's own installed-class table, which is what `.NAME`
        // resolution reads first -- see `environment/identities.rs`'s
        // `record_package_class` for why the registry's flat table is not
        // that.
        // `ClassDirective::install` passes the directive's own `isPublic()`
        // to `addInstalledClass` (`instructions/ClassDirective.cpp:209`),
        // which files a public class in both of the package's tables and
        // every other class in one (`classes/PackageClass.cpp:1410`-`:1419`).
        // That is the difference `~publicClasses` reads back.
        self.record_package_class(program, &class.name, id, class.access == Access::Public)?;
        Ok(id)
    }

    /// Installs the `::CLASS` at `index`, whose declared targets
    /// [`class_install_order`] has already put before it.
    fn install_class_at(
        &mut self,
        program_id: ProgramId,
        program: &Rc<Program>,
        index: usize,
        declared: &FxHashMap<Box<[u8]>, usize>,
        installed: &FxHashMap<usize, ObjRef>,
        attached: &[usize],
    ) -> Result<ObjRef, Failure> {
        let directive = &program.directives[index];
        if let Some(loud) = directive_gap(&directive.kind) {
            return Err(loud.into());
        }
        let DirectiveKind::Class(class) = &directive.kind else {
            return Err(Loud::missing_body().into());
        };
        let file_classes = FileClasses {
            declared,
            installed,
        };
        let named_metaclass = match &class.metaclass {
            None => None,
            Some(target) => Some(self.resolve_class_target(
                program_id,
                program,
                directive,
                target,
                &file_classes,
                Raised::metaclass_not_found,
            )?),
        };
        let superclass = match &class.subclass {
            None => self.root_and_metaclass().0,
            Some(target) => self.resolve_class_target(
                program_id,
                program,
                directive,
                target,
                &file_classes,
                Raised::class_not_found,
            )?,
        };
        // `RexxClass::subclass`'s own opening (`ClassClass.cpp:1566`-
        // `:1575`): a directive naming no `METACLASS` derives from the
        // superclass's own, and either way the value has to be a metaclass
        // before anything is built from it. Measured, `::CLASS S MIXINCLASS
        // Class METACLASS Object` raises this even though deriving from
        // `.Class` then discards the named metaclass -- the test is on what
        // the directive named, not on what the class ends up with.
        let metaclass = named_metaclass.unwrap_or_else(|| self.classes().metaclass(superclass));
        if !self.classes().is_metaclass(metaclass) {
            let name = self.class_default_name(metaclass).to_vec();
            self.blame_directive(program_id, program, directive);
            return Err(Raised::bad_metaclass(&name).into());
        }
        // A metaclass `NEW` that runs is a send from the directive.
        let sends = !dispatch::builds_in_place(self, metaclass);
        let frame = sends.then(|| self.push_directive_activation(program_id, program));
        let id = self.install_class(program_id, class, superclass, metaclass);
        if let Some(frame) = frame {
            self.pop_directive_activation(frame);
        }
        let id = match id {
            Ok(id) => id,
            Err(failure) => {
                self.seal_site_level();
                self.blame_directive(program_id, program, directive);
                return Err(failure);
            }
        };
        self.record_literal_constants(program_id, program, attached);
        // **The class-side members are the enhancing methods the class is
        // built with**: `ClassDirective::install` hands `classMethods` to
        // `mixinClass`/`subclass` (`ClassDirective.cpp:200`, `:205`), which
        // merges them into the class method dictionary before it builds
        // either behaviour and therefore before it sends `INIT`
        // (`ClassClass.cpp:1602`-`:1607`, then `:1613` builds the behaviour and
        // `:1631` sends the message).
        self.install_class_members(program_id, program, id, attached, true);
        // `RexxClass::subclass`'s own tail, in its order: `checkUninit`
        // (`ClassClass.cpp:1628`), the `INIT` send (`:1631`), then the
        // parent's `UNINIT` propagation (`:1634`-`:1637`). Reading a finished
        // parent is what `order` buys: a class is constructed after every
        // class it names.
        self.classes().check_uninit(id);
        self.send_directive_message(program_id, program, id, dispatch::INIT, directive)?;
        self.classes().refresh_parent_has_uninit(id);
        for target in &class.inherit {
            let mixin = self.resolve_class_target(
                program_id,
                program,
                directive,
                target,
                &file_classes,
                Raised::class_not_found,
            )?;
            self.inherit_mixin(program_id, program, directive, id, mixin)?;
        }
        self.install_class_members(program_id, program, id, attached, false);
        // Charged once its own methods count toward its behaviours' weight.
        if !self.library_bootstrap {
            dispatch::charge_class(self, id);
        }
        // `RexxClass::makeAbstract` (`ClassClass.cpp:1754`-`:1761`): a
        // metaclass cannot be made abstract, and any other class takes the
        // keyword by setting the flag `~new`'s `checkAbstract` reads.
        if class.abstract_ {
            if self.classes().is_metaclass(id) {
                let class_id = self.class_id_text(id).as_bytes().to_vec();
                self.blame_directive(program_id, program, directive);
                return Err(Raised::abstract_metaclass(&class_id).into());
            }
            self.classes().make_abstract(id);
        }
        Ok(id)
    }

    /// One side of a class's own members: the `::METHOD` and `::ATTRIBUTE`
    /// directives whose `CLASS` keyword matches `class_side`, and every
    /// `::CONSTANT`, which installs on both.
    fn install_class_members(
        &mut self,
        program_id: ProgramId,
        program: &Rc<Program>,
        class: ObjRef,
        attached: &[usize],
        class_side: bool,
    ) {
        for &index in attached {
            match &program.directives[index].kind {
                DirectiveKind::Method(method) if method.class_method == class_side => {
                    self.install_method(program_id, index, class, method);
                }
                DirectiveKind::Attribute(attribute) if attribute.class_method == class_side => {
                    self.install_attribute(program_id, index, class, attribute);
                }
                DirectiveKind::Constant(constant) => {
                    self.install_constant(program_id, index, class, constant, class_side);
                }
                _ => {}
            }
        }
    }

    /// `::CONSTANT`'s own R9 install: the upcased name lands in one of
    /// `class`'s dictionaries as a [`GeneratedKind::Constant`] accessor.
    fn install_constant(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        constant: &ConstantDirective,
        class_method: bool,
    ) {
        let upper = constant.name.to_ascii_uppercase();
        self.install_one_method(
            program,
            directive,
            class,
            &upper,
            InstallBody::Generated(GeneratedKind::Constant),
            class_method,
            Access::Default,
            Protection::Default,
        );
    }

    /// One class reference on a `::CLASS`, resolved against the file's own
    /// `::CLASS` names and then the registry.
    fn resolve_class_target(
        &mut self,
        installing: ProgramId,
        program: &Rc<Program>,
        directive: &Directive,
        target: &ClassRef,
        classes: &FileClasses<'_>,
        not_found: fn(&[u8]) -> Raised,
    ) -> Result<ObjRef, Failure> {
        if let Some(namespace) = target.namespace {
            let namespace = program.symbols.name(namespace).as_bytes().to_vec();
            let found = self.namespace_class(installing, &namespace, &target.name);
            if found.is_err() {
                self.blame_directive(installing, program, directive);
            }
            return found;
        }
        match classes.declared.get(target.name.as_ref()) {
            // Already installed, because `class_install_order` put it ahead
            // of this one; a `None` here would be that ordering and its
            // caller's loop disagreeing, which is an internal inconsistency
            // and gets this crate's loud refusal rather than a panic.
            Some(other) => match classes.installed.get(other) {
                Some(id) => Ok(*id),
                None => Err(Loud::missing_body().into()),
            },
            None => match self.directive_class(installing, &target.name) {
                Ok(Some(id)) => Ok(id),
                Ok(None) => {
                    self.blame_directive(installing, program, directive);
                    Err(not_found(&target.name).into())
                }
                Err(failure) => {
                    self.blame_directive(installing, program, directive);
                    Err(failure)
                }
            },
        }
    }

    /// One `INHERIT` entry, as the send the oracle makes for it.
    fn inherit_mixin(
        &mut self,
        program_id: ProgramId,
        program: &Rc<Program>,
        directive: &Directive,
        class: ObjRef,
        mixin: ObjRef,
    ) -> Result<(), Failure> {
        let Err(refusal) = self.classes().inherit(class, mixin) else {
            return Ok(());
        };
        // `Interp::class_default_name` borrows out of the registry, so the
        // substitutions are taken before the blaming below reborrows it.
        let mixin_name = self.class_default_name(mixin).to_vec();
        let raised = match refusal {
            InheritRefusal::NotAMixin => Raised::inherit_needs_a_mixinclass(&mixin_name),
            InheritRefusal::Recursive => {
                let class_name = self.class_default_name(class).to_vec();
                Raised::recursive_inherit(&class_name, &mixin_name)
            }
            InheritRefusal::BaseClass(base) => {
                let class_name = self.class_default_name(class).to_vec();
                let base_name = self.class_default_name(base).to_vec();
                Raised::inherit_base_class(&class_name, &mixin_name, &base_name)
            }
            // The `INHERIT` keyword carries no position, and this refusal is
            // the position's alone (`ClassClass.cpp:1350`), so a directive
            // cannot reach it. Answered rather than unreachable-panicked,
            // this crate's rule for a case the type admits and the caller
            // does not produce.
            InheritRefusal::NotInherited(other) => {
                let class_name = self.class_default_name(class).to_vec();
                let other_name = self.class_default_name(other).to_vec();
                Raised::not_inherited(&class_name, &other_name)
            }
        };
        let scope = self.root_and_metaclass().1;
        let scope = self.classes().id_string(scope).to_string();
        self.blame_native_method(b"INHERIT", &scope, class, &[Some(mixin)]);
        self.blame_directive(program_id, program, directive);
        Err(raised.into())
    }

    /// `::METHOD`'s own R9 install: the name lands in `class`'s instance
    /// dictionary, or its class dictionary for `::METHOD ... CLASS`, and
    /// [`Interp::record_method_body`] records which directive to read its
    /// body from later (Task 7's, not entered here).
    fn install_method(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        method: &MethodDirective,
    ) {
        // The bind `Interp::install_directives`' walk already resolved, asked
        // again rather than staged: `method_external` is a pure function of
        // the directive, and a staging map keyed by directive index would be
        // a second place for the answer to live. Every entry point it names
        // resolved, because that walk returned 90.998 for the first that did
        // not and this install never ran.
        let external = dispatch::native::method_external(method);
        for (name, generated) in method_dictionary_keys(method) {
            let native = dispatch::native::bound_entry(external.as_ref(), &name);
            debug_assert!(
                native.is_none() || generated.is_none(),
                "a ::METHOD bound to a LIBRARY REXX entry point also generated a method \
                 for {}, so one of them is lost",
                String::from_utf8_lossy(&name)
            );
            let body = self.install_body(external.as_ref(), &name, generated);
            self.install_one_method(
                program,
                directive,
                class,
                &name,
                body,
                method.class_method,
                method.access,
                method.protection,
            );
        }
    }

    /// `::ATTRIBUTE`'s own R9 install: one or two accessor names, per
    /// [`AttributeStyle`](rexx_parse::AttributeStyle) -- the plain name for a getter, the name with `=`
    /// appended for a setter, both for the default (neither `GET` nor `SET`)
    /// style -- landing in `class`'s instance or class dictionary the same
    /// way [`Interp::install_method`] does.
    fn install_attribute(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        attribute: &AttributeDirective,
    ) {
        // Read for the reason `Interp::install_method`'s is, and asked per
        // dictionary key because the two accessors of an `EXTERNAL` attribute
        // name two different procedures.
        let external = dispatch::native::attribute_external(attribute);
        for (name, generated) in attribute_dictionary_keys(attribute) {
            let body = self.install_body(external.as_ref(), &name, generated);
            // Both accessors of a `Both`-style attribute carry the
            // directive's own access scope, which is the oracle's own shape:
            // `attributeDirective` builds the getter and the setter and calls
            // `setAttributes(accessFlag, protectedFlag, guardFlag)` on each
            // (`parser/DirectiveParser.cpp:1683`, `:1690`).
            self.install_one_method(
                program,
                directive,
                class,
                &name,
                body,
                attribute.class_method,
                attribute.access,
                attribute.protection,
            );
        }
    }

    /// Adds one name to `class`'s instance or class dictionary and records
    /// what the name resolves to -- the tail every `::METHOD` and
    /// `::ATTRIBUTE` install shares, once per dictionary key rather than once
    /// per directive.
    #[allow(clippy::too_many_arguments)]
    fn install_one_method(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        name: &[u8],
        body: InstallBody,
        class_method: bool,
        access: Access,
        protection: Protection,
    ) {
        self.arm_reqstr_for(name);
        let name = String::from_utf8_lossy(name).into_owned();
        let method_id = if class_method {
            self.classes().add_class_method(class, &name)
        } else {
            self.classes().add_instance_method(class, &name)
        };
        self.record_method_body(method_id, program, directive, body);
        self.record_access_scope(method_id, program, access, protection);
    }

    /// Arms `Interp::reqstr_armed` for a method name the required-string
    /// protocol would send, called from every directive install that adds a
    /// name to a class's dictionary.
    fn arm_reqstr_for(&mut self, installed: &[u8]) {
        if installed == dispatch::MAKESTRING || installed == dispatch::DEFAULTNAME {
            self.reqstr_armed = true;
        }
    }

    /// Records which `(program, directive)` a just-minted
    /// [`rexx_classes::MethodId`] names, immediately after the call that
    /// minted it -- see [`Interp::method_bodies`]'s own doc for what the key
    /// is.
    fn record_method_body(
        &mut self,
        method: MethodId,
        program: ProgramId,
        directive: usize,
        body: InstallBody,
    ) {
        let previous = match body {
            InstallBody::Written => self
                .method_bodies
                .insert(method, InstalledMethodBody { program, directive })
                .is_some(),
            InstallBody::Generated(kind) => self
                .generated_methods
                .insert(
                    method,
                    GeneratedMethod {
                        program,
                        directive,
                        kind,
                    },
                )
                .is_some(),
            InstallBody::Native(_) | InstallBody::Library(_)
                if self.programs.get(program).is_some_and(|installed| {
                    installed
                        .directives
                        .get(directive)
                        .is_some_and(|declared| crate::guards::directive_guarded(&declared.kind))
                }) =>
            {
                self.guarded_externals.insert(method);
                self.record_external(method, program, body)
            }
            InstallBody::Native(_) | InstallBody::Library(_) => {
                self.record_external(method, program, body)
            }
        };
        debug_assert!(
            !previous,
            "a MethodId was recorded twice, so one of the two bodies is lost"
        );
    }

    /// [`Interp::record_method_body`] for an `EXTERNAL` method, answering
    /// whether `method` was recorded before.
    fn record_external(&mut self, method: MethodId, program: ProgramId, body: InstallBody) -> bool {
        match body {
            InstallBody::Native(entry) => {
                self.external_packages.insert(method, program);
                self.native_externals.insert(method, entry).is_some()
            }
            InstallBody::Library(binding) => {
                self.external_packages.insert(method, program);
                self.library_externals.insert(method, binding).is_some()
            }
            InstallBody::Written | InstallBody::Generated(_) => false,
        }
    }

    /// Files a body compiled from method source text as a program of its own
    /// and hangs it on the `Method` object, so a send can enter it.
    pub(super) fn record_compiled_body(
        &mut self,
        object: ObjRef,
        name: &[u8],
        parsed: Program,
        parent: Option<ObjRef>,
    ) -> Result<(), Failure> {
        let Program {
            source,
            main,
            symbols,
            ..
        } = parsed;
        let program = Rc::new(Program {
            source,
            main: CodeBody::default(),
            directives: vec![Directive {
                kind: DirectiveKind::Method(Box::new(MethodDirective {
                    name: name.into(),
                    class_method: false,
                    attribute: false,
                    abstract_: false,
                    access: Access::default(),
                    protection: Protection::default(),
                    guard: GuardOption::default(),
                    external: None,
                    delegate: None,
                    body: Some(main),
                })),
                clause_span: 0..0,
            }],
            symbols,
        });
        let program_id = self.load_program(program);
        self.compiled_method_names.insert(program_id, name.into());
        self.override_package_settings(program_id)?;
        // `generateMethod` and `generateRoutine` give the executable's own
        // package `parent` as its parent (`LanguageParser.cpp:603`, `:637`),
        // which its routine and class lookups walk.
        if let Some(parent) = parent {
            self.set_package_parent(program_id, parent)?;
        }
        let method = self.mint_unattached_body(
            object,
            InstalledMethodBody {
                program: program_id,
                directive: 0,
            },
        );
        self.record_executable(
            object,
            ExecutableRecord {
                source: ExecutableSource::Main {
                    program: program_id,
                },
                installed: Some(method),
                routine: None,
                owns_installed: false,
            },
        )
    }

    /// Mints the identity a `Method` object no class dictionary holds runs
    /// under, with `body` as what it runs, and files `object` as the one
    /// standing for it.
    fn mint_unattached_body(&mut self, object: ObjRef, body: InstalledMethodBody) -> MethodId {
        let method = self.classes().mint_method_id();
        self.method_bodies.insert(method, body);
        self.executable_objects.insert(method, object);
        method
    }

    /// The identity one `.METHODS` entry runs under, minted with its body
    /// recorded as [`Interp::install_method`], [`Interp::install_attribute`]
    /// and [`Interp::install_constant`] record a class's: `name` is the
    /// entry's key, which picks the half of an accessor pair. `source` is
    /// `program`'s own.
    pub(crate) fn unattached_method_id(
        &mut self,
        program: ProgramId,
        source: &Rc<Program>,
        directive: usize,
        name: &[u8],
    ) -> MethodId {
        let (body, access, protection) = match &source.directives[directive].kind {
            DirectiveKind::Method(method) => {
                let external = dispatch::native::method_external(method);
                let generated = method_dictionary_keys(method)
                    .into_iter()
                    .find(|(key, _)| key.as_slice() == name)
                    .and_then(|(_, generated)| generated);
                let body = self.install_body(external.as_ref(), name, generated);
                (body, method.access, method.protection)
            }
            DirectiveKind::Attribute(attribute) => {
                let external = dispatch::native::attribute_external(attribute);
                let generated = attribute_dictionary_keys(attribute)
                    .into_iter()
                    .find(|(key, _)| key.as_slice() == name)
                    .and_then(|(_, generated)| generated);
                let body = self.install_body(external.as_ref(), name, generated);
                (body, attribute.access, attribute.protection)
            }
            // Only a literal `::CONSTANT` can precede the first `::CLASS`.
            _ => {
                if !self
                    .program_data(program)
                    .is_some_and(|data| data.constants.contains_key(&directive))
                {
                    self.record_literal_constants(program, source, &[directive]);
                }
                (
                    InstallBody::Generated(GeneratedKind::Constant),
                    Access::Default,
                    Protection::default(),
                )
            }
        };
        let method = self.classes().mint_method_id();
        self.record_method_body(method, program, directive, body);
        self.record_access_scope(method, program, access, protection);
        method
    }

    /// What one dictionary key of a `::METHOD` or `::ATTRIBUTE` runs: the
    /// accessor the directive generates, the entry point or library procedure
    /// its `EXTERNAL` binds, or else its written body.
    fn install_body(
        &mut self,
        external: Option<&dispatch::native::MethodExternal>,
        name: &[u8],
        generated: Option<GeneratedKind>,
    ) -> InstallBody {
        let native = dispatch::native::bound_entry(external, name);
        let library = self.library_binding(external, name);
        match (generated, native, library) {
            (Some(kind), _, _) => InstallBody::Generated(kind),
            (None, Some(entry), _) => InstallBody::Native(entry),
            (None, None, Some(binding)) => InstallBody::Library(binding),
            (None, None, None) => InstallBody::Written,
        }
    }

    /// [`Interp::record_compiled_body`] for a `Routine`: the same program of
    /// its own, carrying a `::ROUTINE` rather than a `::METHOD`.
    pub(super) fn record_compiled_routine(
        &mut self,
        object: ObjRef,
        name: &[u8],
        parsed: Program,
        parent: Option<ObjRef>,
    ) -> Result<(), Failure> {
        let Program {
            source,
            main,
            symbols,
            ..
        } = parsed;
        let routine_name = source.image_name().unwrap_or(name).into();
        let program = Rc::new(Program {
            source,
            main: CodeBody::default(),
            directives: vec![Directive {
                kind: DirectiveKind::Routine(Box::new(rexx_parse::RoutineDirective {
                    name: routine_name,
                    access: Access::default(),
                    external: None,
                    body: Some(main),
                })),
                clause_span: 0..0,
            }],
            symbols,
        });
        let program_id = self.load_program(program);
        self.compiled_method_names.insert(program_id, name.into());
        self.override_package_settings(program_id)?;
        // `generateMethod` and `generateRoutine` give the executable's own
        // package `parent` as its parent (`LanguageParser.cpp:603`, `:637`),
        // which its routine and class lookups walk.
        if let Some(parent) = parent {
            self.set_package_parent(program_id, parent)?;
        }
        self.record_executable(
            object,
            ExecutableRecord {
                source: ExecutableSource::Main {
                    program: program_id,
                },
                installed: None,
                routine: Some((program_id, 0)),
                owns_installed: false,
            },
        )
    }

    /// `MethodClass::newFileRexx` and `RoutineClass::newFileRexx`
    /// (`classes/MethodClass.cpp:521`, `classes/RoutineClass.cpp:341`): the
    /// executable a file's own text becomes.
    ///
    /// `parent` is the package the loaded file resolves names through, and
    /// `None` is the caller's own.
    pub(crate) fn new_file_executable(
        &mut self,
        name: &[u8],
        routine: bool,
        parent: Option<ObjRef>,
    ) -> Result<ObjRef, Failure> {
        // **Read through the interpreter's own directory, recorded under the
        // name as given.** `Invocation::with_directory` is what a caller
        // running several interpreters on threads sets, so a relative name
        // must resolve there rather than against the process; but the package
        // keeps the spelling it was asked for -- measured, `~package~name`
        // for `newFile('rel.rex')` answers `rel.rex`.
        let resolved = crate::paths::normalize_bytes(name, self.cwd_bytes());
        let Ok(text) = std::fs::read(crate::paths::os_path(&resolved)) else {
            return Err(Raised::executable_file_unreadable(name).into());
        };
        let parsed = self.parse_file(text, name)?;
        let parent = match parent {
            // A restored executable inherits no context, the one given
            // included (`LanguageParser::createRoutine(name, context)`).
            _ if !parsed.source.is_traceable() => None,
            Some(parent) => Some(parent),
            None => self
                .running_program()
                .map(|program| self.package_object(Package::Program(program)))
                .transpose()?,
        };
        self.install_executable(parsed, name, routine, parent, |interp, id| {
            interp.required_paths.insert(id, name.into());
        })
    }

    /// `LanguageParser::generateRoutine` and `generateMethod`
    /// (`parser/LanguageParser.cpp:624`) over a parsed source: its main body
    /// becomes the executable, its directives install into a package of its
    /// own whose parent is `parent`, and `record` names that package. A
    /// routine restored from an image keeps the name the image gives it.
    pub(crate) fn install_executable(
        &mut self,
        parsed: Program,
        name: &[u8],
        routine: bool,
        parent: Option<ObjRef>,
        record: impl FnOnce(&mut Self, ProgramId),
    ) -> Result<ObjRef, Failure> {
        let Program {
            source,
            main,
            mut directives,
            symbols,
        } = parsed;
        let body = directives.len();
        directives.push(Directive {
            kind: if routine {
                DirectiveKind::Routine(Box::new(rexx_parse::RoutineDirective {
                    name: source.image_name().unwrap_or(name).into(),
                    access: Access::default(),
                    external: None,
                    body: Some(main),
                }))
            } else {
                DirectiveKind::Method(Box::new(MethodDirective {
                    name: name.into(),
                    class_method: false,
                    attribute: false,
                    abstract_: false,
                    access: Access::default(),
                    protection: Protection::default(),
                    guard: GuardOption::default(),
                    external: None,
                    delegate: None,
                    body: Some(main),
                }))
            },
            clause_span: 0..0,
        });
        let program = Rc::new(Program {
            source,
            main: CodeBody::default(),
            directives,
            symbols,
        });
        let id = self.load_program(Rc::clone(&program));
        record(self, id);
        // **Recorded before the directives install, not after.** Installing
        // runs a `::REQUIRES` prologue, and code running there resolves
        // routines -- which must already reach the package that built this
        // executable. `load_requires` caches ahead of its own prologue for
        // the same reason.
        if let Some(parent) = parent {
            self.set_package_parent(id, parent)?;
        }
        // The executable's package installs in the running activation, which
        // `NUMERIC INHERIT` inherits from. Measured: `.Routine~newFile` of a
        // file with no `::OPTIONS` requiring an inheriting file under
        // `NUMERIC DIGITS 4` reports 9 in that file's prologue, and with
        // `::OPTIONS NUMERIC INHERIT` reports 4.
        let running = self
            .running_activation()
            .map(|activation| activation.settings.clone());
        let outer = self.install_caller.replace(running.clone());
        let outer_call = self.install_call.replace(CallType::Requires);
        let installed = self.install_directives(id, &program);
        self.install_caller = outer;
        self.install_call = outer_call;
        if let Err(failure) = installed {
            self.seal_package_level(&failure, id);
            return Err(failure);
        }
        // `PackageClass::install` installs a package with classes or
        // requires by calling an empty routine of the package, which traces
        // its `>I>` and `<I<` under the package's `::OPTIONS TRACE`. The main
        // body built here is that empty routine.
        if program.directives.iter().any(|directive| {
            matches!(
                directive.kind,
                DirectiveKind::Class(_) | DirectiveKind::Requires(_)
            )
        }) {
            self.run_install_stub(Rc::clone(&program), id, running)?;
        }
        let class = if routine {
            self.routine_class()
        } else {
            self.method_class()
        };
        let object = self.native_instance(class);
        let site = environment::Annotated::Compiled(self.compiled_methods);
        self.compiled_methods += 1;
        self.attach_annotations(object, site);
        let installed = (!routine).then(|| {
            self.mint_unattached_body(
                object,
                InstalledMethodBody {
                    program: id,
                    directive: body,
                },
            )
        });
        self.record_executable(
            object,
            ExecutableRecord {
                source: ExecutableSource::Main { program: id },
                installed,
                routine: routine.then_some((id, body)),
                owns_installed: false,
            },
        )?;
        Ok(object)
    }

    /// Records a `Method` or `Routine` object whose body is a primitive, so
    /// that its readers answer `BaseCode`'s: an empty `~source`, the `REXX`
    /// package, and `0` from `~setSecurityManager`.
    pub(crate) fn record_native_executable(&mut self, object: ObjRef) {
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Native,
                installed: None,
                routine: None,
                owns_installed: false,
            },
        );
    }

    /// Records a `Method` or `Routine` object a `loadExternal*` send answered
    /// over a library procedure, whose package is `code`'s.
    pub(crate) fn record_loaded_executable(&mut self, object: ObjRef, code: usize) {
        let installed =
            (!self.library_code_key(code).routine).then(|| self.bind_loaded_method(object, code));
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Loaded { code },
                installed,
                routine: None,
                owns_installed: installed.is_some(),
            },
        );
    }

    /// [`Interp::record_native_executable`] for a `loadExternalMethod`
    /// answer over a `LIBRARY REXX` entry point, which runs that entry.
    pub(crate) fn record_rexx_external_method(
        &mut self,
        object: ObjRef,
        entry: &'static dispatch::native::NativeExternal,
    ) {
        self.record_native_executable(object);
        let method = self.classes().mint_method_id();
        self.native_externals.insert(method, entry);
        self.executable_objects.insert(method, object);
        if let Some(record) = self.executable_sources.get_mut(&object) {
            record.installed = Some(method);
            record.owns_installed = true;
        }
    }

    /// What the `Method` object for one installed method reports on: the
    /// directive that declared it, [`ExecutableSource::External`] for an
    /// `EXTERNAL` binding, or [`ExecutableSource::Native`] for a primitive.
    pub(crate) fn installed_executable_source(&self, method: MethodId) -> ExecutableSource {
        if let Some(installed) = self.method_bodies.get(&method) {
            return ExecutableSource::Directive {
                program: installed.program,
                directive: installed.directive,
            };
        }
        if let Some(generated) = self.generated_methods.get(&method) {
            if generated.kind == GeneratedKind::ConstantExpression {
                return ExecutableSource::ConstantExpression {
                    program: generated.program,
                    directive: generated.directive,
                };
            }
            return ExecutableSource::Directive {
                program: generated.program,
                directive: generated.directive,
            };
        }
        if let Some(program) = self.external_packages.get(&method) {
            return ExecutableSource::External { program: *program };
        }
        ExecutableSource::Native
    }

    /// The [`Interp::library_codes`] row for `key`, made unbound where there
    /// is none yet.
    pub(crate) fn library_code(&mut self, key: LibraryCodeKey) -> usize {
        if let Some(row) = self.library_code_rows.get(&key) {
            return *row;
        }
        let row = self.library_codes.len();
        self.library_codes.push(None);
        self.library_code_keys.push(key.clone());
        self.library_code_routines.push(None);
        self.library_code_rows.insert(key, row);
        row
    }

    /// Which package `source` reports, or `None` for a loaded procedure no
    /// directive has bound yet, whose package the oracle answers as `.nil`.
    pub(crate) fn source_package(&self, source: ExecutableSource) -> Option<Package> {
        match source {
            ExecutableSource::Directive { program, directive }
                if self
                    .rexx_routine_rows
                    .contains_key(&InstalledRoutine { program, directive }) =>
            {
                // Measured, oracle: `findRoutine` of a `::ROUTINE` bound to
                // `LIBRARY REXX Filespec` answers a routine whose package is
                // `The REXX Package`.
                Some(Package::Rexx)
            }
            ExecutableSource::Directive { program, directive } => Some(Package::Program(
                self.library_routine_codes
                    .get(&InstalledRoutine { program, directive })
                    .and_then(|row| self.library_codes[*row])
                    .unwrap_or(program),
            )),
            ExecutableSource::ConstantExpression { program, .. }
            | ExecutableSource::Main { program }
            | ExecutableSource::External { program } => Some(Package::Program(program)),
            ExecutableSource::Native => Some(Package::Rexx),
            ExecutableSource::Loaded { code } => self
                .library_codes
                .get(code)
                .copied()
                .flatten()
                .map(Package::Program),
        }
    }

    /// Replaces the environment this interpreter starts with, and the library
    /// search taken from it.
    pub(crate) fn adopt_environment(&mut self, environment: Vec<(Vec<u8>, Vec<u8>)>) {
        self.library_search = library_search_of(&environment);
        self.env = environment;
    }

    /// `PackageManager::loadLibrary` (`package/PackageManager.cpp:229`): the
    /// library `name` names, loaded once however many directives, methods and
    /// `~loadLibrary` sends ask for it.
    ///
    /// **The name is matched byte for byte**, which is measured rather than
    /// assumed: oracle, `::method m external "LIBRARY RXREGEXP RegExp_Init"`
    /// is `98.903 Unable to load library "RXREGEXP"` where the same directive
    /// spelling the name in lower case loads.
    pub(crate) fn resolve_library(&mut self, name: &[u8]) -> LibraryLoad {
        if let Some(held) = self.libraries.get(name) {
            return LibraryLoad::Loaded(Rc::clone(held));
        }
        #[cfg(test)]
        {
            self.library_open_attempts += 1;
            if let Some(library) = offered_library(name) {
                return self.settle_library(name, Ok(Some(library)));
            }
        }
        let spelling = String::from_utf8_lossy(name).into_owned();
        let opened = rexx_api::load::open(&spelling, &self.library_search);
        self.settle_library(name, opened)
    }

    /// What opening `name` answered, held where the oracle's package table
    /// keeps it.
    pub(super) fn settle_library(
        &mut self,
        name: &[u8],
        opened: Result<Option<rexx_api::load::Library>, rexx_api::load::Refused>,
    ) -> LibraryLoad {
        match opened {
            Ok(Some(mut library)) => {
                library.keep_thread_context(&self.thread_context());
                let held = self.libraries.hold(name, Rc::new(library));
                self.register_package_routines(name, &held);
                // Measured, oracle: a loader that raises leaves the library
                // held with its routines callable, and the ask raises.
                match self.run_package_hook(&held, rexx_api::load::Hook::Loader) {
                    Ok(()) => LibraryLoad::Loaded(held),
                    Err(failure) => LibraryLoad::Raised(failure),
                }
            }
            Ok(None) => {
                self.libraries.missed(name);
                LibraryLoad::Missing
            }
            Err(mut refused) => {
                refused.library.keep_thread_context(&self.thread_context());
                self.libraries.hold(name, Rc::from(refused.library));
                LibraryLoad::Version
            }
        }
    }

    /// `LibraryPackage::loadRoutines` (`package/LibraryPackage.cpp:270-299`):
    /// every routine `library` exports becomes callable by name from any
    /// package, whatever loaded it.
    fn register_package_routines(&mut self, name: &[u8], library: &rexx_api::load::Library) {
        for (upper, spelling) in library.package_routines() {
            self.routines_changed();
            let code = self.library_code(LibraryCodeKey {
                library: name.to_vec(),
                procedure: spelling.to_vec(),
                routine: true,
            });
            match self.package_routines.get(&upper) {
                Some(slot) => self.package_routine_codes[*slot] = code,
                None => {
                    self.package_routines
                        .insert(upper, self.package_routine_codes.len());
                    self.package_routine_codes.push(code);
                }
            }
        }
    }

    /// `PackageClass::mergeLibrary`: `library`'s routines join `id`'s own
    /// routine lookup where no earlier merge put the name.
    fn merge_library(
        &mut self,
        id: ProgramId,
        name: &[u8],
        library: &rexx_api::load::Library,
    ) -> Result<(), Failure> {
        let routines: Vec<(Box<[u8]>, MergedRoutine)> = library
            .package_routines()
            .into_iter()
            .map(|(upper, spelling)| {
                let code = self.library_code(LibraryCodeKey {
                    library: name.to_vec(),
                    procedure: spelling.to_vec(),
                    routine: true,
                });
                (upper.into_boxed_slice(), MergedRoutine::Library(code))
            })
            .collect();
        let into = self.package_object(Package::Program(id))?;
        self.merge_routines(into, routines);
        Ok(())
    }

    /// `PackageClass::mergeLibrary` over an internal package: its routines
    /// join `id`'s own routine lookup where no earlier merge put the name.
    fn merge_internal_package(
        &mut self,
        id: ProgramId,
        package: internal_routines::Package,
    ) -> Result<(), Failure> {
        let routines: Vec<(Box<[u8]>, MergedRoutine)> = internal_routines::package_indexes(package)
            .map(|index| {
                let name = internal_routines::INTERNAL_ROUTINES[index].name;
                let upper = name.to_ascii_uppercase().into_bytes();
                (upper.into_boxed_slice(), MergedRoutine::Internal(index))
            })
            .collect();
        let into = self.package_object(Package::Program(id))?;
        self.merge_routines(into, routines);
        Ok(())
    }

    /// Adds each of `routines` to `into`'s imported routines where the name
    /// is not there yet (`HashContents::mergePut`), moving
    /// [`Interp::routine_generation`] when one is added.
    fn merge_routines(&mut self, into: ObjRef, routines: Vec<(Box<[u8]>, MergedRoutine)>) {
        let Some(data) = self.package_data_mut(into) else {
            return;
        };
        let target = &mut data.merged_public_routines;
        let mut added = false;
        for (name, merged) in routines {
            if let std::collections::hash_map::Entry::Vacant(vacant) = target.entry(name) {
                vacant.insert(merged);
                added = true;
            }
        }
        if added {
            self.routines_changed();
        }
    }

    /// Moves [`Interp::routine_generation`], for a write to a routine table.
    pub(crate) fn routines_changed(&mut self) {
        self.routine_generation = self.routine_generation.wrapping_add(1);
    }

    /// [`Interp::resolve_library`] for a caller whose failure is a condition:
    /// 98.903 for a name that resolved to nothing and 98.982 for a package
    /// entry asking for a newer interpreter.
    fn require_library(&mut self, name: &[u8]) -> Result<Rc<rexx_api::load::Library>, Failure> {
        match self.resolve_library(name) {
            LibraryLoad::Loaded(library) => Ok(library),
            LibraryLoad::Missing => Err(Raised::library_not_loaded(name).into()),
            LibraryLoad::Version => Err(Raised::library_version(name).into()),
            LibraryLoad::Raised(failure) => Err(failure),
        }
    }

    /// The oracle's translation-time resolution of one directive's
    /// `EXTERNAL`: the library procedures it binds and the internal package
    /// routine it names, or the refusal. Measured, oracle: a file opening
    /// `say "prolog ran"` and carrying `::method x external "LIBRARY zorkolib
    /// z"` is 98.903 rc 158 with stdout empty, and `"LIBRARY REXX nosuch"` and
    /// `"LIBRARY rexxutil SysNoSuch"` on a `::ROUTINE` are 90.999 rc 166 on the
    /// directive's line.
    pub(crate) fn resolve_external(
        &mut self,
        kind: &DirectiveKind,
    ) -> Result<
        (
            Vec<LibraryCodeKey>,
            Option<&'static internal_routines::InternalRoutine>,
        ),
        ExternalRefusal,
    > {
        let external = match kind {
            DirectiveKind::Method(method) => method.external.is_some(),
            DirectiveKind::Attribute(attribute) => attribute.external.is_some(),
            DirectiveKind::Routine(routine) => routine.external.is_some(),
            _ => false,
        };
        if !external {
            return Ok((Vec::new(), None));
        }
        let refuse = |failure: Failure, echoes| ExternalRefusal { failure, echoes };
        if let Some(loud) = directive_gap(kind) {
            return Err(refuse(loud.into(), false));
        }
        if let Some(missing) = unresolved_external(kind) {
            let raised = Raised::external_method_not_found(&missing);
            return Err(refuse(raised.into(), true));
        }
        let keys = self
            .resolve_directive_library(kind)
            .map_err(|failure| refuse(failure, true))?;
        let mut internal = None;
        if let DirectiveKind::Routine(routine) = kind
            && let Some((package, entry)) = dispatch::native::internal_routine_entry(routine)
        {
            let Some(row) = internal_routines::package_routine(package, &entry) else {
                let raised = Raised::external_routine_not_found(&entry);
                return Err(refuse(raised.into(), true));
            };
            internal = Some(row);
        }
        Ok((keys, internal))
    }

    /// The library and the procedures one directive's `EXTERNAL` names,
    /// resolved so that a program naming a library that is not there is
    /// refused before its own first clause.
    ///
    /// Answers the procedures it bound, and none for a directive whose
    /// `EXTERNAL` names the `REXX` package or none at all; those are
    /// [`unresolved_external`]'s and [`directive_gap`]'s.
    fn resolve_directive_library(
        &mut self,
        kind: &DirectiveKind,
    ) -> Result<Vec<LibraryCodeKey>, Failure> {
        let external = match kind {
            DirectiveKind::Method(method) => dispatch::native::method_external(method),
            DirectiveKind::Attribute(attribute) => dispatch::native::attribute_external(attribute),
            DirectiveKind::Routine(routine) => dispatch::native::routine_external(routine),
            _ => return Ok(Vec::new()),
        };
        let Some(dispatch::native::MethodExternal::OtherLibrary { library, binds }) = external
        else {
            return Ok(Vec::new());
        };
        let routine = matches!(kind, DirectiveKind::Routine(_));
        let loaded = self.require_library(&library)?;
        // A routine's procedure is in the routine table and a method's in the
        // method table, and the two are separate exports: measured, oracle,
        // `::routine zzz external "LIBRARY rxmath"` is `90.999 Unable to find
        // external routine "ZZZ"`.
        let mut keys = Vec::with_capacity(binds.len());
        for bind in &binds {
            let procedure = if routine {
                loaded.routine(&bind.procedure).map(|row| row.name.clone())
            } else {
                loaded
                    .method(&bind.procedure)
                    .map(|_| bind.procedure.clone())
            };
            let Some(procedure) = procedure else {
                return Err(if routine {
                    Raised::external_routine_not_found(&bind.procedure).into()
                } else {
                    Raised::external_method_not_found(&bind.procedure).into()
                });
            };
            keys.push(LibraryCodeKey {
                library: library.clone(),
                procedure,
                routine,
            });
        }
        Ok(keys)
    }

    /// The [`Interp::package_routines`] slot `name` resolves to, or `None`
    /// for a name no loaded library exports as a routine.
    pub(crate) fn package_routine(&self, name: &[u8]) -> Option<usize> {
        // Ahead of the upcase, which allocates: a program that loads no
        // library reaches this on every name that resolves nowhere else.
        if self.package_routines.is_empty() {
            return None;
        }
        self.package_routines
            .get(&name.to_ascii_uppercase())
            .copied()
    }

    /// The [`Interp::library_codes`] row a [`Interp::package_routine`] slot
    /// calls.
    pub(crate) fn package_routine_code(&self, slot: usize) -> usize {
        self.package_routine_codes[slot]
    }

    /// The [`Interp::library_codes`] row a library-backed `::ROUTINE`
    /// directive bound, or `None` for any other routine.
    pub(crate) fn library_routine_code(&self, installed: InstalledRoutine) -> Option<usize> {
        self.library_routine_codes.get(&installed).copied()
    }

    /// The internal package routine a `::ROUTINE` directive bound, or `None`
    /// for any other routine.
    pub(crate) fn rexx_routine_row(
        &self,
        installed: InstalledRoutine,
    ) -> Option<&'static internal_routines::InternalRoutine> {
        self.rexx_routine_rows.get(&installed).copied()
    }

    /// The internal package routine `object` runs, or `None` for any other
    /// object.
    pub(crate) fn rexx_routine_object(
        &self,
        object: ObjRef,
    ) -> Option<&'static internal_routines::InternalRoutine> {
        match &self.heap.get(object)?.body {
            rexx_core::Body::Native(native) => native
                .routine_row()
                .map(|index| &internal_routines::INTERNAL_ROUTINES[index]),
            _ => None,
        }
    }

    /// Records `object`, a `Body::Native` `Routine`, as one that runs the
    /// internal package routine at `index`.
    pub(crate) fn record_rexx_routine_object(&mut self, object: ObjRef, index: usize) {
        self.record_native_executable(object);
        if let Some(rexx_core::Body::Native(native)) =
            self.heap.get_mut(object).map(|held| &mut held.body)
        {
            native.set_routine_row(index);
        }
    }

    /// The library and the routine row [`Interp::library_codes`] row `code`
    /// is, resolved by name on the first call, or `None` where its library
    /// is not held or exports no such routine.
    #[inline]
    pub(crate) fn library_code_routine(
        &mut self,
        code: usize,
    ) -> Option<(&rexx_api::load::Library, usize)> {
        let key = &self.library_code_keys[code];
        let library = self.libraries.get(&key.library)?;
        let index = match self.library_code_routines[code] {
            Some(index) => index as usize,
            None => {
                let index = library.routine_index(&key.procedure)?;
                self.library_code_routines[code] = u32::try_from(index).ok();
                index
            }
        };
        debug_assert_eq!(
            Some(index),
            library.routine_index(&key.procedure),
            "a library code's cached routine row is not the one its name finds"
        );
        Some((library, index))
    }

    /// The procedure [`Interp::library_codes`] row `code` is.
    pub(crate) fn library_code_key(&self, code: usize) -> &LibraryCodeKey {
        &self.library_code_keys[code]
    }

    /// The file of the package row `code`'s shared code reports, or `None`
    /// while no directive has bound it.
    pub(crate) fn library_code_package_path(&self, code: usize) -> Option<Vec<u8>> {
        let program = self.library_code_program(code)?;
        Some(self.package_path(program).to_vec())
    }

    /// The package [`Interp::library_code_package_path`] names.
    pub(crate) fn library_code_program(&self, code: usize) -> Option<ProgramId> {
        self.library_codes.get(code).copied().flatten()
    }

    /// The library procedure a dictionary key's `EXTERNAL` binds it to, or
    /// `None` where the directive names no library or the procedure is not
    /// exported.
    fn library_binding(
        &mut self,
        external: Option<&dispatch::native::MethodExternal>,
        key: &[u8],
    ) -> Option<LibraryBinding> {
        let Some(dispatch::native::MethodExternal::OtherLibrary { library, binds }) = external
        else {
            return None;
        };
        let bind = binds.iter().find(|bind| *bind.key == *key)?;
        let LibraryLoad::Loaded(library) = self.resolve_library(library) else {
            return None;
        };
        let method = library.method_index(&bind.procedure)?;
        Some(LibraryBinding {
            library,
            method: Some(method),
        })
    }

    /// `Method~setPrivate`'s half that a send can see: the dictionary entry
    /// this object *is* stops answering a sender outside its scope.
    pub(crate) fn make_method_private(&mut self, object: ObjRef) {
        let Some(method) = self.unshared_identity(object) else {
            return;
        };
        let package = self
            .source_package(self.installed_executable_source(method))
            .unwrap_or(plan::Package::Rexx);
        match self.special_methods.get_mut(method) {
            Some(existing) => existing.access = Access::Private,
            None => self.special_methods.insert(
                method,
                dispatch::AccessScope {
                    access: Access::Private,
                    protected: false,
                    package,
                },
            ),
        }
    }

    /// The name directive `directive` of `program` gives its `::ROUTINE`:
    /// upcased unless it was quoted.
    pub(crate) fn routine_directive_name(&self, program: ProgramId, directive: usize) -> Vec<u8> {
        match self
            .programs
            .get(program)
            .and_then(|program| program.directives.get(directive))
            .map(|directive| &directive.kind)
        {
            Some(DirectiveKind::Routine(routine)) => routine.name.to_vec(),
            _ => Vec::new(),
        }
    }

    /// One `::ROUTINE` over the arguments given, for the rows of
    /// `executable.rs`'s `RESUMABLE_METHODS`, up to the point its body would
    /// run.
    pub(crate) fn begin_installed_routine(
        &mut self,
        program: ProgramId,
        directive: usize,
        arguments: &mut Vec<Option<ObjRef>>,
        name: &[u8],
    ) -> Result<crate::dispatch::NativeStarted, Failure> {
        let installed = InstalledRoutine { program, directive };
        Ok(
            match self.begin_call(
                crate::run::Resolved::Routine(installed),
                name,
                arguments,
                0,
                crate::activation::CallType::Subroutine,
                crate::run::CallEntry::Written,
            )? {
                crate::run::Begun::Done(crate::run::Ended::Exited(value)) => {
                    return Err(Failure::Exited(value));
                }
                crate::run::Begun::Done(crate::run::Ended::Returned(returned)) => {
                    crate::dispatch::NativeStarted::Ran(returned)
                }
                crate::run::Begun::Entered => {
                    crate::dispatch::NativeStarted::Entered(crate::dispatch::Then::Pass)
                }
            },
        )
    }

    /// `PARSE SOURCE`'s third word: a compiled method's own name, the file a
    /// `::REQUIRES` loaded this package from, or the running program's path.
    pub(crate) fn program_display_name(&self, program: ProgramId) -> &[u8] {
        match self.compiled_method_names.get(&program) {
            Some(name) => name,
            None => self.package_path(program),
        }
    }

    /// Records a just-minted method's access scope and protection, for the
    /// methods the oracle calls *special*.
    fn record_access_scope(
        &mut self,
        method: MethodId,
        program: ProgramId,
        access: Access,
        protection: Protection,
    ) {
        let protected = protection == Protection::Protected;
        let scoped = matches!(access, Access::Private | Access::Package);
        if !protected && !scoped {
            return;
        }
        self.special_methods.insert(
            method,
            dispatch::AccessScope {
                access,
                protected,
                package: Package::Program(program),
            },
        );
    }

    /// Runs the `::CONSTANT` expression of `program`'s directive `directive`
    /// as a method of its own, as `ClassDirective::resolveConstants` does
    /// (`instructions/ClassDirective.cpp:257`): a guarded `METHOD` level named
    /// `::CONSTANT` with `run`'s receiver and scope, stopped on the directive's
    /// clause, whose executable is a `Method` of its own and which the
    /// package's `::OPTIONS TRACE` traces as a method. While a package
    /// installs, below it is the level installing the package, stopped on
    /// directive `level`, the `::CLASS` installed last.
    fn eval_constant_expression(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        run: ConstantRun<'_>,
        directive: usize,
        expr: &Expr,
        level: Option<usize>,
    ) -> Result<ObjRef, Failure> {
        let ConstantRun {
            name,
            receiver,
            scope: class,
            executable,
            arguments,
        } = run;
        let empty_body = CodeBody::default();
        // Taken before the frame is pushed, as every activation's is.
        let caller = self.install_caller.clone().flatten();
        let start = self.package_start(id, caller.as_ref());
        let executable = match executable {
            Some(executable) => executable,
            None => self.constant_executable(id, directive, class)?,
        };
        let (line, text) = self.directive_text(id, program, &program.directives[directive]);
        let saved_clause = self.save_clause_state();
        let installing = self.install_call.map(|call_type| {
            let frame = self.roots.push_slots(0);
            let level_id = self.next_activation_id();
            let mut installing = Activation::new(
                level_id,
                Rc::clone(program),
                id,
                Rc::new(Plan::default()),
                frame,
            );
            installing.call_type = call_type;
            installing.body = None;
            installing.cold_mut().directive_clause = level;
            self.push_activation(installing);
            self.record_call_convention();
            let line = level.map_or(0, |class| {
                directive_clause(program, &program.directives[class]).0
            });
            self.enter_directive_clause(line);
            frame
        });
        let frame = self.roots.push_slots(0);
        let activation_id = self.next_activation_id();
        let name: Rc<[u8]> = Rc::from(name);
        let mut activation = Activation::method(
            activation_id,
            Rc::clone(program),
            id,
            directive,
            Rc::new(Plan::default()),
            frame,
            MethodIdentity {
                name: Rc::clone(&name),
                scope: class,
                receiver,
            },
        );
        // It starts from the package's `::OPTIONS` settings, and under
        // `NUMERIC INHERIT` from the installing activation's. Measured:
        // `::constant k (1+b)` under `::OPTIONS NOVALUE SYNTAX` is 98.986,
        // `(1/3)` under `::OPTIONS DIGITS 5` is `0.33333`, and under
        // `NUMERIC INHERIT` in a file a `DIGITS 4` package requires, `0.3333`.
        if let Some(start) = start {
            start.apply(&mut activation);
        }
        // No body: the expression is all it runs, so a call in it finds no
        // label of its own.
        activation.body = None;
        activation.cold_mut().executable = Some(executable);
        activation.cold_mut().directive_clause = Some(directive);
        let saved_context = std::mem::replace(
            &mut self.activity.call_context,
            CallContext {
                name,
                arguments: Rc::from(arguments),
                receiver: Some(receiver),
            },
        );
        self.push_activation(activation);
        // Guarded, as the method object reports, and waited for here: the
        // level runs no clause where a queued wait would be served.
        self.trace_package_invocation_entry();
        self.reserve_for_method(crate::guards::GuardKey {
            object: receiver,
            scope: class,
        });
        let reserved = self.serve_guard_wait(false);
        self.enter_directive_clause(line);
        self.record_call_convention();
        let self_slot = self.slot_of(b"SELF");
        self.set_variable(frame, self_slot, receiver);
        let super_slot = self.slot_of(b"SUPER");
        // The receiver's behaviour's super scope, as `Interp::begin_method`
        // writes it, `.nil` for the topmost scope.
        let super_scope = self.super_scope_of(receiver, class);
        self.set_variable(frame, super_slot, super_scope.unwrap_or(ObjRef::NIL));
        let result = reserved.and_then(|()| {
            self.trace_clause(line, 0, &text);
            let code = Code {
                body: &empty_body,
                symbols: &program.symbols,
                slots: &[],
                plan: None,
            };
            self.eval(&code, expr)
        });
        if let Ok(value) = result
            && let Some(rendered) = self.result_text(value)?
        {
            self.trace_result(0, &rendered);
        }
        self.trace_invocation_exit(|| result.as_ref().ok().copied());
        self.guard_off_at_end();
        self.activity.call_context = saved_context;
        self.pop_activation();
        self.roots.activity_mut().pop_slots(frame);
        if let Some(level_frame) = installing {
            self.pop_activation();
            self.roots.activity_mut().pop_slots(level_frame);
        }
        self.restore_clause_state(saved_clause);
        result
    }

    /// A `::CONSTANT` expression method sent as `name` to `receiver` in
    /// `resolution`'s scope: the expression evaluated again, answering
    /// nothing (measured, oracle: a `define`d copy sent for a result is 91.999).
    pub(crate) fn run_constant_expression(
        &mut self,
        generated: GeneratedMethod,
        resolution: dispatch::Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(program) = self.programs.get(generated.program).map(Rc::clone) else {
            return Err(Loud::missing_body().into());
        };
        let Some(DirectiveKind::Constant(constant)) =
            program.directives.get(generated.directive).map(|d| &d.kind)
        else {
            return Err(Loud::missing_body().into());
        };
        let ConstantValue::Expression(expr) = &constant.value else {
            return Err(Loud::missing_body().into());
        };
        let executable = self.executable_objects.get(&resolution.method).copied();
        let run = ConstantRun {
            name,
            receiver,
            scope: resolution.scope,
            executable,
            arguments: args,
        };
        let outer = self.install_call.take();
        let evaluated = self.eval_constant_expression(
            generated.program,
            &program,
            run,
            generated.directive,
            expr,
            None,
        );
        self.install_call = outer;
        match evaluated {
            Ok(_) => Ok(None),
            Err(failure) => {
                // The level's own echo and frame, as at install, without the
                // `::CLASS` echo only installing adds.
                let (id, directive) = (generated.program, generated.directive);
                self.seal_constant_level(&failure, id, &program, directive, name, args);
                Err(failure)
            }
        }
    }

    /// Records the `::CONSTANT` level `failure` leaves as a site of its own:
    /// the directive's clause, blamed and echoed, and its `METHOD` frame named
    /// `name`. Answers whether it recorded one: a condition raised again below
    /// the expression, by a `result` it waited on, keeps the position and
    /// echoes it was raised with, and no level it leaves records a site.
    fn seal_constant_level(
        &mut self,
        failure: &Failure,
        id: ProgramId,
        program: &Rc<Program>,
        directive: usize,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> bool {
        if matches!(
            self.activity.failure_site,
            Some(FailureSite::Reraised { .. })
        ) {
            return false;
        }
        self.blame_directive(id, program, &program.directives[directive]);
        self.capture_site_frame(failure, b"METHOD", name, args, id);
        self.seal_site_level();
        true
    }

    /// The `Method` a `::CONSTANT` expression's evaluation runs as: a fresh
    /// one per evaluation, whose scope is `class` (measured, oracle: it is not
    /// the accessor `class~method` answers, and it reports guarded with every
    /// other flag clear).
    fn constant_executable(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let method_class = self.method_class();
        let object = self.native_instance(method_class);
        self.roots.activity_mut().push_temp(object);
        let method = self.classes().mint_method_id();
        self.generated_methods.insert(
            method,
            GeneratedMethod {
                program,
                directive,
                kind: GeneratedKind::ConstantExpression,
            },
        );
        self.executable_objects.insert(method, object);
        let record = crate::ExecutableRecord {
            source: ExecutableSource::ConstantExpression { program, directive },
            installed: Some(method),
            routine: None,
            owns_installed: true,
        };
        self.record_executable(object, record)?;
        if let Some(rexx_core::Body::Native(native)) =
            self.heap.get_mut(object).map(|held| &mut held.body)
        {
            native.set_scope(class);
        }
        Ok(object)
    }

    /// Records `directive`'s own clause as the site a directive-time
    /// condition is reported against, so its report carries the same echo
    /// line the oracle prints above the two `Error` lines, and names the
    /// package `id` the directive is in (measured, oracle: a `::CLASS` of a
    /// called file naming a missing superclass is `Error 98 running` that
    /// file).
    pub(crate) fn blame_directive(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        directive: &Directive,
    ) {
        let (line, text) = self.directive_text(id, program, directive);
        let name = self.program_display_name(id);
        self.activity.failure_site = Some(if *name == *self.program_path {
            FailureSite::Clause {
                line,
                text,
                indent: 0,
            }
        } else {
            FailureSite::Named {
                line,
                indent: 0,
                text,
                name: name.to_vec(),
            }
        });
    }

    /// [`Interp::blame_directive`] for a refusal the oracle's translation
    /// raises, whose level is the `COMPILE` frame (measured, oracle: a called
    /// file's missing library and its cyclic `SUBCLASS` pair both list
    /// `COMPILE` where a missing superclass lists `ROUTINE`).
    fn blame_translation(&mut self, id: ProgramId, program: &Rc<Program>, directive: &Directive) {
        self.blame_directive(id, program, directive);
        let name = self.program_display_name(id).to_vec();
        if let Some(site) = self.activity.failure_site.take() {
            self.record_compile_frame(&site, &name);
            self.activity
                .failure_origin
                .get_or_insert((Package::Program(id), site.line()));
            self.activity.failure_site = Some(site);
        }
    }

    /// `directive`'s line and the text its traceback line shows: its clause,
    /// or for a restored program the line naming the program (measured, a
    /// restored `::CLASS` naming a missing superclass).
    fn directive_text(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        directive: &Directive,
    ) -> (usize, Vec<u8>) {
        let (line, text) = directive_clause(program, directive);
        if program.source.is_traceable() {
            return (line, text);
        }
        let package = self.program_display_name(id).to_vec();
        (line, self.sourceless_line(0, &package))
    }

    /// Closes the level of package `id`, whose directives did not install, as
    /// the `ROUTINE` frame the oracle shows for it, so the native method or
    /// clause that loaded it records its own level.
    pub(crate) fn seal_package_level(&mut self, failure: &Failure, id: ProgramId) {
        let name = self.program_display_name(id).to_vec();
        self.capture_site_frame(failure, b"ROUTINE", &name, &[], id);
        self.seal_site_level();
    }
}

#[cfg(test)]
thread_local! {
    /// Libraries a test offers by name, which [`Interp::resolve_library`]
    /// takes before it searches for a shared object of that name.
    static OFFERED: std::cell::RefCell<Vec<(Vec<u8>, fn() -> rexx_api::load::Library)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Offers the library `make` builds under `name`, on this thread.
#[cfg(test)]
pub(crate) fn offer_library(name: &[u8], make: fn() -> rexx_api::load::Library) {
    OFFERED.with(|offered| offered.borrow_mut().push((name.to_vec(), make)));
}

/// The library a test offered under `name` on this thread, built afresh.
#[cfg(test)]
fn offered_library(name: &[u8]) -> Option<rexx_api::load::Library> {
    OFFERED.with(|offered| {
        offered
            .borrow()
            .iter()
            .find(|(offered, _)| offered == name)
            .map(|(_, make)| make())
    })
}

/// Files every dictionary key the member `directive` claims under the class
/// it attaches to, keeping the first claimant of a key. A class-side key with
/// no class above it is a `::CONSTANT`'s, which claims none there.
fn claim_member_keys(
    directive: &Directive,
    current_class: Option<usize>,
    claimed: &mut FxHashMap<(Option<usize>, bool, Vec<u8>), usize>,
    index: usize,
) {
    for (name, class_side) in member_dictionary_keys(&directive.kind) {
        if current_class.is_none() && class_side {
            continue;
        }
        claimed
            .entry((current_class, class_side, name))
            .or_insert(index);
    }
}

/// A translation-time refusal of a directive's `EXTERNAL`
/// ([`Interp::resolve_external`]), and whether its report echoes the
/// directive, which a loud refusal's does not.
pub(crate) struct ExternalRefusal {
    pub(crate) failure: Failure,
    echoes: bool,
}

/// Who a `::CONSTANT` expression's level runs for: its receiver and scope,
/// the `Method` it is the level of where one already stands for it, and its
/// arguments.
struct ConstantRun<'a> {
    /// The level's name: `::CONSTANT` while the package installs, which is
    /// `GlobalNames::CONSTANT_DIRECTIVE` (`memory/GlobalNames.h:84`), and the
    /// message name on a run.
    name: &'a [u8],
    receiver: ObjRef,
    scope: ObjRef,
    executable: Option<ObjRef>,
    arguments: &'a [Option<ObjRef>],
}
