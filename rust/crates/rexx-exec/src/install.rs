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

use crate::activation::{Activation, CallType};
use crate::directives::{
    AnnotatedSite, FileClasses, annotation_target, attribute_dictionary_keys, class_install_order,
    class_members, constant_root_key, directive_clause, directive_gap, member_dictionary_keys,
    method_dictionary_keys, unresolved_external,
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
    Protection, parse_program,
};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

impl Interp {
    /// Resolves every `::` directive of `program`, filling [`Interp::routines`]
    /// and refusing the ones this crate cannot resolve.
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
    /// ::annotate routine nosuchrtn          99.945 rc 157
    /// duplicate ::routine of the same name  99.903 rc 157
    /// ```
    pub(super) fn install_directives(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
    ) -> Result<(), Failure> {
        // **The oracle's first walk, and everything it can answer is answered
        // here in source order** -- the duplicate names (`::CLASS`,
        // `::ROUTINE`, `::RESOURCE`, and a member directive's own dictionary
        // keys), a parenthesised `::CONSTANT` with no `::CLASS` before it, a
        // `CLASS` keyword with no `::CLASS` before it, an `::ANNOTATE` target
        // and an `EXTERNAL` library. Measured, `::constant sep (1+2)` alone in
        // a file is rc 157 with `Error 99.906`, where the identical directive
        // under a preceding `::CLASS` reaches the install-time evaluation
        // below instead; and see the table above `directive_gap` for the order this walk is
        // the first of, and for every probe placing a form in it.
        let mut saw_class = false;
        // The class a member directive's keys are claimed against, and the
        // keys claimed so far. `None` is `LanguageParser`'s `unattachedMethods`
        // table, which is one table for the whole file rather than one per
        // class (`parser/DirectiveParser.cpp:518`).
        let mut current_class: Option<usize> = None;
        // Which directive claimed each key, and not merely that one did:
        // `::ANNOTATE ATTRIBUTE` and `::ANNOTATE CONSTANT` accept only a key
        // whose claimant is of the matching kind, which is `isAttribute()`
        // and `isConstant()` on the method object the C++ finds.
        let mut claimed: HashMap<(Option<usize>, bool, Vec<u8>), usize> = HashMap::new();
        // The other two tables the duplicate checks keep, each keyed by the
        // upcased name and separate from the others, so that a `::CLASS` and a
        // `::ROUTINE` of one name are not a collision. See
        // `Raised::duplicate_class` for the probes on both halves.
        // The class and routine tables carry the declaring directive's index
        // as well, because an `::ANNOTATE CLASS` or `::ANNOTATE ROUTINE`
        // resolves its target against exactly these two -- `classDependencies`
        // and `routines`, the same tables `findClassDirective` and
        // `findRoutine` read (`parser/DirectiveParser.cpp:230`, `:258`).
        let mut declared_classes: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut declared_routines: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut declared_resources: std::collections::HashSet<Vec<u8>> =
            std::collections::HashSet::new();
        // What each `::ANNOTATE` recorded, keyed by what its target names.
        // Filled in this walk and converted below, because a target's own
        // object does not exist yet: a `::CLASS` has no class object until
        // the install pass creates one.
        let mut staged: BTreeMap<AnnotatedSite, Vec<(Box<[u8]>, Box<[u8]>)>> = BTreeMap::new();
        // The package's routine records, handed over where this walk finishes
        // as `resolveDependencies` hands its tables to the package
        // (`parser/LanguageParser.cpp:1893-1900`), so a translation that raises
        // leaves none.
        let mut routines: HashMap<Box<[u8]>, InstalledRoutine> = HashMap::new();
        let mut public_routines: HashMap<Box<[u8]>, InstalledRoutine> = HashMap::new();
        let mut routine_codes: Vec<(InstalledRoutine, usize)> = Vec::new();
        let mut rexx_routines: Vec<(
            InstalledRoutine,
            &'static internal_routines::InternalRoutine,
        )> = Vec::new();
        self.untranslated.insert(id);
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
            // **Before the arms below, because the oracle checks before it
            // adds.** `constantDirective` calls `checkDuplicateMethod` ahead
            // of `createConstantGetterMethod`, which is what raises 99.906
            // (`parser/DirectiveParser.cpp:1926`, `:1933`), and the two part:
            // measured, `::constant c 5` then `::constant c (1+2)` with no
            // `::CLASS` in the file is 99.932 and not 99.906.
            self.check_member_keys(program, directive, current_class, &mut claimed, index)?;
            match &directive.kind {
                DirectiveKind::Class(class) => {
                    if declared_classes
                        .insert(class.name.to_ascii_uppercase(), index)
                        .is_some()
                    {
                        self.blame_directive(program, directive);
                        return Err(Raised::duplicate_class().into());
                    }
                    saw_class = true;
                    current_class = Some(index);
                }
                DirectiveKind::Resource(resource) => {
                    if !declared_resources.insert(resource.name.to_ascii_uppercase()) {
                        self.blame_directive(program, directive);
                        return Err(Raised::duplicate_resource().into());
                    }
                }
                DirectiveKind::Constant(constant) => {
                    if matches!(constant.value, ConstantValue::Expression(_)) && !saw_class {
                        self.blame_directive(program, directive);
                        return Err(Raised::constant_needs_class().into());
                    }
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
                    if routines.insert(name, installed).is_some() {
                        // A *translation* error on the oracle, not an install
                        // one: measured, two `::routine zork` directives give
                        // `Error 99.903: Duplicate ::ROUTINE directive
                        // instruction.` at rc 157, echoing the second
                        // directive's own clause. `rexx-parse` does not
                        // detect it, so it is detected here, where the
                        // accumulated table is what answers.
                        self.blame_directive(program, directive);
                        return Err(Raised::duplicate_routine().into());
                    }
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
                            self.blame_directive(program, directive);
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
                        let pairs = staged.entry(site).or_default();
                        for annotation in &annotate.annotations {
                            let name = program.symbols.name(annotation.name).as_bytes();
                            pairs.retain(|(held, _)| **held != *name);
                            pairs.push((name.into(), annotation.value.clone()));
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
                    let outcome = {
                        let package = self.package_options.entry(id).or_default();
                        options.iter().try_for_each(|option| package.apply(option))
                    };
                    if let Err(error) = outcome {
                        self.blame_directive(program, directive);
                        return Err(run::raised_from_settings(error).into());
                    }
                    // The required-string protocol's third arming route:
                    // `::OPTIONS NOSTRING SYNTAX` turns a rendering into a
                    // raise exactly as a `NOSTRING` trap does. See
                    // [`Interp::reqstr_armed`] for why this only ever sets.
                    if self
                        .options_of(id)
                        .is_some_and(PackageOptions::escalates_nostring)
                    {
                        self.reqstr_armed = true;
                    }
                    // `Interp::lostdigits_armed`'s only write, and the same
                    // set-once rule: the arithmetic path's gate.
                    if self
                        .options_of(id)
                        .is_some_and(PackageOptions::escalates_lostdigits)
                    {
                        self.lostdigits_armed = true;
                    }
                }
                _ => {}
            }

            // **After the arms above, not before them**, because when one
            // directive is both a duplicate `::ROUTINE` and an `EXTERNAL`
            // the oracle answers the duplicate: measured, `::routine dup`
            // followed by `::routine dup external "LIBRARY nosuchlib
            // nosuchfn"` is 99.903 rc 157 echoing the second directive, not
            // 98.903. Reverse that pair and the `EXTERNAL` comes first in the
            // file and wins, which the walk gives.
            if matches!(
                directive.kind,
                DirectiveKind::Annotate(_)
                    | DirectiveKind::Routine(_)
                    | DirectiveKind::Method(_)
                    | DirectiveKind::Attribute(_)
            ) && let Some(loud) = directive_gap(&directive.kind)
            {
                return Err(loud.into());
            }

            // **The eager bind** (D37), in this walk because the oracle does
            // it while the directive is being translated:
            // `createNativeMethod` raises from inside `methodDirective`
            // (`parser/DirectiveParser.cpp:1385`), so the file is refused
            // before its own first clause runs. Measured, oracle: a file
            // opening `say "prolog ran"` and carrying one `::METHOD
            // EXTERNAL` on a missing entry point is rc 166 with stdout empty,
            // and the same file naming `file_separator` is rc 0 printing the
            // prologue.
            if let Some(missing) = unresolved_external(&directive.kind) {
                self.blame_directive(program, directive);
                return Err(Raised::external_method_not_found(&missing).into());
            }

            // The same walk and the same position for a library-backed
            // `EXTERNAL`: measured, oracle, a file opening `say "prolog ran"`
            // and carrying `::method x external "LIBRARY zorkolib z"` is
            // 98.903 rc 158 with stdout empty. Each procedure's shared code is
            // bound to this package as its directive resolves, where nothing
            // bound it first, so a later directive's refusal leaves the
            // binding in place (`createNativeMethod`,
            // `parser/DirectiveParser.cpp:1381-1388`).
            for key in self.resolve_directive_library(id, program, directive)? {
                let routine = key.routine;
                let row = self.library_code(key);
                self.library_codes[row].get_or_insert(id);
                if routine {
                    let installed = InstalledRoutine {
                        program: id,
                        directive: index,
                    };
                    routine_codes.push((installed, row));
                }
            }

            // The `REXX` package's routine table, found as a library's is:
            // measured, oracle, `"LIBRARY REXX nosuch"` is 90.999 rc 166 on
            // the directive's line.
            if let DirectiveKind::Routine(routine) = &directive.kind
                && let Some(entry) = dispatch::native::rexx_routine_entry(routine)
            {
                let Some(row) = internal_routines::rexx_package_routine(&entry) else {
                    self.blame_directive_in(id, program, directive);
                    return Err(Raised::external_routine_not_found(&entry).into());
                };
                let installed = InstalledRoutine {
                    program: id,
                    directive: index,
                };
                rexx_routines.push((installed, row));
            }
        }
        if !routines.is_empty() {
            self.routines.entry(id).or_default().extend(routines);
        }
        if !public_routines.is_empty() {
            self.package_public_routines
                .entry(id)
                .or_default()
                .extend(public_routines);
        }
        self.library_routine_codes.extend(routine_codes);
        self.rexx_routine_rows.extend(rexx_routines);
        self.untranslated.remove(&id);

        // **A second pass, because the oracle's own translation-time
        // refusals above happen before every install-time one below**
        // (98.9xx/43.901/the `::CONSTANT` expression evaluation), so a
        // program with both gets the translation error -- which is what
        // running the whole first pass before any of this reproduces.
        let mut declared: HashMap<Box<[u8]>, usize> = HashMap::new();
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
                self.blame_directive(program, &program.directives[blame]);
                let path = self.program_path.clone();
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
        let mut classes: HashMap<usize, ObjRef> = HashMap::new();
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
    pub(super) fn package_path(&self, id: ProgramId) -> &str {
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
        if let Some(code) = self.defined_library_codes.get(&method) {
            return self.library_code_package_path(*code);
        }
        let program = *self.external_packages.get(&method)?;
        Some(self.package_path(program).as_bytes().to_vec())
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
        self.requires_installing.push(self.package_path(id).into());
        let outcome = self.install_requires(id, program);
        self.requires_installing.pop();
        outcome
    }

    /// [`Interp::load_required_packages`]' walk: every `::REQUIRES ... LIBRARY`
    /// in source order, then every other `::REQUIRES` in source order, as
    /// `PackageClass::processInstall` installs them
    /// (`classes/PackageClass.cpp:1227-1260`), stopping at the first that
    /// fails.
    fn install_requires(&mut self, id: ProgramId, program: &Rc<Program>) -> Result<(), Failure> {
        for library in [true, false] {
            for directive in &program.directives {
                let DirectiveKind::Requires(requires) = &directive.kind else {
                    continue;
                };
                if requires.library != library {
                    continue;
                }
                if library {
                    match self.require_library(&requires.name) {
                        Ok(loaded) => self.merge_library(id, &requires.name, &loaded),
                        Err(failure) => {
                            self.seal_site_level();
                            self.blame_directive_in(id, program, directive);
                            return Err(failure);
                        }
                    }
                    continue;
                }
                match self.load_requires(Some(id), &requires.name) {
                    Ok(required) => {
                        self.add_imported_package(id, Package::Program(required));
                        self.merge_required(id, required);
                        // `RequiresDirective::install`
                        // (`instructions/RequiresDirective.cpp:137`): the
                        // registration is what the directive does *after* the
                        // load and the merge, so a namespace neither narrows
                        // the merge nor replaces it.
                        if let Some(namespace) = requires.namespace {
                            let name = program.symbols.name(namespace).as_bytes().into();
                            self.package_namespaces
                                .entry(id)
                                .or_default()
                                .insert(name, Package::Program(required));
                        }
                    }
                    Err(failure) => {
                        self.seal_site_level();
                        self.blame_directive_in(id, program, directive);
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
    fn load_requires(
        &mut self,
        from: Option<ProgramId>,
        name: &[u8],
    ) -> Result<ProgramId, Failure> {
        // **The manager sees the short name before the cache**, and the
        // resolved one after the search: `PackageManager::loadRequires`
        // (`package/PackageManager.cpp:718`-`:766`) checks each in turn, so a
        // manager that renames or forbids a package is asked twice.
        let mut inherited = None;
        let Some(name) = self.check_requires_access(name, &mut inherited)? else {
            return Err(Raised::requires_file_not_found(name).into());
        };
        let name = &name[..];
        if let Some(&loaded) = self.required_packages.get(name) {
            self.check_not_installing(loaded)?;
            return Ok(loaded);
        }
        let resolved = match from {
            Some(id) => self.resolve_requires(id, name),
            None => self.resolve_search(None, name, true),
        };
        let resolved = match resolved {
            Some(resolved) => {
                let Some(checked) =
                    self.check_requires_access(resolved.as_bytes(), &mut inherited)?
                else {
                    return Err(Raised::requires_file_not_found(name).into());
                };
                Some(String::from_utf8_lossy(&checked).into_owned())
            }
            None => None,
        };
        if let Some(resolved) = &resolved
            && let Some(&loaded) = self.required_packages.get(resolved.as_bytes())
        {
            self.check_not_installing(loaded)?;
            self.required_packages.insert(name.into(), loaded);
            return Ok(loaded);
        }
        let Some(resolved) = resolved else {
            return Err(Raised::requires_file_not_found(name).into());
        };
        // The search already answered that this names a regular file, so a
        // read failing here is a permission or a race rather than a miss --
        // and the oracle reports the same 43.901 for it, since
        // `PackageManager::loadRequires` answers `OREF_NULL` either way.
        let Ok(text) = std::fs::read(&resolved) else {
            return Err(Raised::requires_file_not_found(name).into());
        };
        let parsed = match parse_program(text) {
            Ok(parsed) => Rc::new(parsed),
            Err(error) => {
                return Err(Loud::required_source(&resolved, &format!("{error}")).into());
            }
        };
        let required = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&parsed));
        self.required_paths
            .insert(required, resolved.as_str().into());
        // Installed before the prologue runs, which is where
        // `getRequiresFile` puts it (`package/PackageManager.cpp:830`-`:833`).
        if let Some(manager) = inherited {
            self.install_security_manager(required, Some(manager));
        }
        // **Cached before the prologue runs, not after**, which is
        // `addRequiresFile` standing ahead of `runProlog` at
        // `InterpreterInstance.cpp:1060`: a name reached again from inside
        // that prologue must find this entry, or the circularity check has
        // nothing to fire on.
        self.required_packages.insert(name.into(), required);
        self.required_packages
            .insert(resolved.as_bytes().into(), required);
        if let Err(failure) = self.run_loaded(parsed, required, CallType::Requires, None, None) {
            // `getRequiresFile` caches a package only once it has translated
            // (`package/PackageManager.cpp:828-836`), so a later ask for one
            // whose translation raised translates it again; a package that
            // failed to install or in its prologue stays cached.
            if self.untranslated.contains(&required) {
                self.required_packages.remove(name);
                self.required_packages.remove(resolved.as_bytes());
            }
            return Err(failure);
        }
        Ok(required)
    }

    /// The security manager's `REQUIRES` checkpoint
    /// (`execution/SecurityManager.cpp:318`-`:346`).
    ///
    /// Answers the name to load under -- the manager's own replacement where
    /// it made one -- or `None` where it removed the entry, which forbids the
    /// load. `inherited` collects the `SECURITYMANAGER` the new package is to
    /// carry, across both of a load's two checks.
    fn check_requires_access(
        &mut self,
        name: &[u8],
        inherited: &mut Option<ObjRef>,
    ) -> Result<Option<Vec<u8>>, Failure> {
        if self.effective_security_manager().is_none() {
            return Ok(Some(name.to_vec()));
        }
        let requested = self.text(name);
        self.roots.push_temp(requested);
        let entries = [(crate::security::key::NAME, requested)];
        let Some(info) = self.security_check(crate::security::message::REQUIRES, &entries)? else {
            return Ok(Some(name.to_vec()));
        };
        if let Some(manager) = self.security_entry(info, crate::security::key::SECURITYMANAGER)? {
            self.roots.push_temp(manager);
            *inherited = Some(manager);
        }
        let Some(replaced) = self.security_entry(info, crate::security::key::NAME)? else {
            return Ok(None);
        };
        self.roots.push_temp(replaced);
        let text = self.required_string_value(replaced)?;
        Ok(Some(self.to_text(text).into_owned()))
    }

    /// 98.952 when `loaded`'s own `::REQUIRES` directives are still
    /// installing, and `Ok` otherwise -- `Activity::checkRequires`
    /// (`concurrency/Activity.cpp:3702`).
    fn check_not_installing(&self, loaded: ProgramId) -> Result<(), Failure> {
        let path = self.package_path(loaded);
        if self.requires_installing.iter().any(|open| &**open == path) {
            return Err(Raised::circular_requires(path).into());
        }
        Ok(())
    }

    /// `PackageClass::addPackage` (`classes/PackageClass.cpp:1367`): records
    /// `from` as one of `into`'s imports, once.
    pub(super) fn add_imported_package(&mut self, into: ProgramId, from: Package) -> bool {
        let held = self.package_imports.entry(into).or_default();
        if held.contains(&from) {
            return false;
        }
        held.push(from);
        true
    }

    /// [`Interp::merge_required`] for either kind of package.
    pub(super) fn merge_package(&mut self, into: ProgramId, from: Package) {
        let from = match from {
            Package::Program(program) => return self.merge_required(into, program),
            Package::Rexx => self.rexx_package_class_table(true),
        };
        self.invalidate_rexx_class_cache();
        let target = self.merged_public_classes.entry(into).or_default();
        for (name, class) in from {
            target.entry(name).or_insert(class);
        }
    }

    /// The public routines and classes `from` contributes to `into`: its own
    /// first, then the ones it imported.
    pub(super) fn merge_required(&mut self, into: ProgramId, from: ProgramId) {
        let routines: Vec<(Box<[u8]>, MergedRoutine)> = self
            .package_public_routines
            .get(&from)
            .into_iter()
            .flatten()
            .map(|(name, installed)| (name.clone(), MergedRoutine::Installed(*installed)))
            .chain(
                self.merged_public_routines
                    .get(&from)
                    .into_iter()
                    .flatten()
                    .map(|(name, merged)| (name.clone(), *merged)),
            )
            .collect();
        self.merge_routines(into, routines);
        let classes: Vec<(Box<[u8]>, ObjRef)> = self
            .package_public_classes
            .get(&from)
            .into_iter()
            .chain(self.merged_public_classes.get(&from))
            .flatten()
            .map(|(name, class)| (name.clone(), *class))
            .collect();
        self.invalidate_rexx_class_cache();
        let target = self.merged_public_classes.entry(into).or_default();
        for (name, class) in classes {
            target.entry(name).or_insert(class);
        }
    }

    /// The package a namespace qualifier written in `package` names, or `None`
    /// when nothing registered it.
    fn find_namespace(&self, package: ProgramId, upper: &[u8]) -> Option<Namespace> {
        if upper == LIBRARY_PACKAGE_NAME {
            return Some(Namespace::Rexx);
        }
        self.package_namespaces
            .get(&package)?
            .get(upper)
            .copied()
            .map(|package| match package {
                Package::Rexx => Namespace::Rexx,
                Package::Program(program) => Namespace::Package(program),
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
            let path = self.package_path(package).to_owned();
            return Err(Raised::namespace_not_found(namespace, &path).into());
        };
        let found = match target {
            Namespace::Rexx => self.rexx_package_class(name),
            Namespace::Package(program) => self.public_class_of(program, name),
        };
        found.ok_or_else(|| Raised::namespace_class_not_found(name, namespace).into())
    }

    /// `findPublicClass` for one package: its own `::CLASS ... PUBLIC`
    /// declarations, then the ones it imported (`classes/PackageClass.cpp:760`
    /// region). Measured, oracle rc 0: a class a *required* file of the
    /// namespace package declares public is reachable through the qualifier.
    fn public_class_of(&self, program: ProgramId, upper: &[u8]) -> Option<ObjRef> {
        if let Some(found) = self
            .package_public_classes
            .get(&program)
            .and_then(|table| table.get(upper))
        {
            return Some(*found);
        }
        self.merged_public_classes
            .get(&program)
            .and_then(|table| table.get(upper))
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
            let path = self.package_path(package).to_owned();
            return Err(Raised::namespace_not_found(namespace, &path).into());
        };
        let found = match target {
            Namespace::Rexx => None,
            Namespace::Package(program) => self
                .package_public_routines
                .get(&program)
                .and_then(|table| table.get(name))
                .map(|installed| run::Resolved::Routine(*installed))
                .or_else(|| {
                    self.merged_public_routines
                        .get(&program)
                        .and_then(|table| table.get(name))
                        .map(|merged| merged.resolved())
                }),
        };
        found.ok_or_else(|| Raised::namespace_routine_not_found(name, namespace).into())
    }

    /// The file a `::REQUIRES` of `name` in package `id` resolves to, or
    /// `None` when no route holds one.
    fn resolve_requires(&self, id: ProgramId, name: &[u8]) -> Option<String> {
        self.resolve_search(Some(self.package_path(id)), name, true)
    }

    /// [`Interp::resolve_requires`] for a caller that names the searching
    /// package's path itself and chooses the resolve type.
    pub(crate) fn resolve_search(
        &self,
        program: Option<&str>,
        name: &[u8],
        requires: bool,
    ) -> Option<String> {
        let name = std::str::from_utf8(name).ok()?;
        // The interpreter's own environment and directory, never the process's
        // (`Interp::env`, `Interp::cwd`): the harnesses run interpreters on
        // threads, and `ootest/testOORexx.rex` sets `PATH` through `VALUE` and
        // changes directory before calling the program it then has to find.
        let rexx_path = self.shadow_var(b"REXX_PATH");
        let sys_path = self.shadow_var(b"PATH");
        let entries = require::search_entries(
            program.and_then(require::program_directory),
            rexx_path.as_deref(),
            sys_path.as_deref(),
        );
        let cwd = self.cwd.to_str()?;
        let extension = program.and_then(require::program_extension);
        let groups = require::candidates(name, &entries, extension, requires);
        // A path that exists and is not a regular file abandons the rest of
        // its spelling and extension rather than the search -- see
        // [`require::first_regular`], which is where that rule lives so it can
        // be tested without a file system.
        let hit = require::first_regular(&groups, |candidate| {
            match std::fs::metadata(require::normalize(candidate, cwd)) {
                Ok(meta) if meta.is_file() => require::Found::Regular,
                Ok(_) => require::Found::Other,
                Err(_) => require::Found::Missing,
            }
        })?;
        Some(require::normalize(&hit, cwd))
    }

    /// `PackageClass::loadPackageRexx`'s load: the same
    /// [`Interp::load_requires`] a `::REQUIRES` performs, which is what the
    /// C++ calls too (`classes/PackageClass.cpp:1842`).
    pub(crate) fn load_package(
        &mut self,
        program: ProgramId,
        name: &[u8],
    ) -> Result<ProgramId, Failure> {
        self.load_requires(Some(program), name)
    }

    /// `PackageClass::newRexx`'s load: the same file load, searched from the
    /// **global** context rather than from any package's directory.
    pub(crate) fn load_package_global(&mut self, name: &[u8]) -> Result<ProgramId, Failure> {
        self.load_requires(None, name)
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
        parent: Option<Package>,
    ) -> Result<ProgramId, Failure> {
        let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
        let parsed = rexx_parse::parse_lines(&borrowed).map_err(|error| {
            Failure::from(Loud::required_source(
                &String::from_utf8_lossy(name),
                &format!("{error}"),
            ))
        })?;
        let parsed = Rc::new(parsed);
        let id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&parsed));
        self.compiled_method_names.insert(id, name.into());
        // Recorded ahead of the prologue for the reason
        // `new_file_executable` records its own parent early: the prologue
        // resolves routines, and must already reach the context.
        if let Some(parent) = parent {
            self.package_parents.insert(id, parent);
        }
        self.run_loaded(parsed, id, CallType::Requires, None, None)?;
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

    /// `LanguageParser::checkDuplicateMethod`
    /// (`parser/DirectiveParser.cpp:507`-`:530`): every dictionary key a
    /// member directive is about to claim, refused if the class it attaches
    /// to has already been given that key on that side.
    fn check_member_keys(
        &mut self,
        program: &Rc<Program>,
        directive: &Directive,
        current_class: Option<usize>,
        claimed: &mut HashMap<(Option<usize>, bool, Vec<u8>), usize>,
        index: usize,
    ) -> Result<(), Failure> {
        let constant = matches!(directive.kind, DirectiveKind::Constant(_));
        for (name, class_side) in member_dictionary_keys(&directive.kind) {
            if current_class.is_none() {
                if constant && class_side {
                    continue;
                }
                if class_side {
                    self.blame_directive(program, directive);
                    return Err(Raised::class_keyword_needs_class().into());
                }
            }
            if claimed
                .insert((current_class, class_side, name), index)
                .is_some()
            {
                self.blame_directive(program, directive);
                return Err(Raised::duplicate_member(&directive.kind).into());
            }
        }
        Ok(())
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
                ConstantValue::Name => self.interned_literal(&constant.name),
                ConstantValue::Text(text) => self.interned_literal(text),
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
            match self.eval_constant_expression(id, program, class, expr) {
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
                    self.blame_directive(program, directive);
                    self.seal_site_level();
                    self.blame_directive(
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

    /// Records what a `::CONSTANT` accessor answers, and roots it.
    fn record_constant_value(&mut self, program: ProgramId, directive: usize, value: ObjRef) {
        self.constant_values.insert((program, directive), value);
        self.roots
            .add_global(&constant_root_key(program, directive), value);
    }

    /// The value a `::CONSTANT` accessor answers, or `None` for an expression
    /// form whose pass has not run yet -- [`Interp::constant_values`] has
    /// which of those is reachable.
    pub(crate) fn constant_value(&self, generated: GeneratedMethod) -> Option<ObjRef> {
        self.constant_values
            .get(&(generated.program, generated.directive))
            .copied()
    }

    /// The constant's name as its accessor was installed under, which is what
    /// a 97.4 report names.
    pub(crate) fn constant_name(&self, generated: GeneratedMethod) -> Result<Vec<u8>, Failure> {
        let program = &self.programs[generated.program.0];
        // `get` rather than an index, and a refusal rather than a panic, for
        // the reason `Interp::enter_method_body`'s own reads carry.
        let Some(directive) = program.directives.get(generated.directive) else {
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
        self.blame_directive(program, blame);
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
        self.roots.pop_slots(frame);
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
    ) -> ObjRef {
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
        let id = self.mint_class();
        self.classes()
            .define_unregistered_class(id, &name, Some(superclass), kind, metaclass);
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
        self.record_package_class(program, &class.name, id, class.access == Access::Public);
        id
    }

    /// Installs the `::CLASS` at `index`, whose declared targets
    /// [`class_install_order`] has already put before it.
    fn install_class_at(
        &mut self,
        program_id: ProgramId,
        program: &Rc<Program>,
        index: usize,
        declared: &HashMap<Box<[u8]>, usize>,
        installed: &HashMap<usize, ObjRef>,
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
            self.blame_directive(program, directive);
            return Err(Raised::bad_metaclass(&name).into());
        }
        let id = self.install_class(program_id, class, superclass, metaclass);
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
            self.inherit_mixin(program, directive, id, mixin)?;
        }
        self.install_class_members(program_id, program, id, attached, false);
        // `RexxClass::makeAbstract` (`ClassClass.cpp:1754`-`:1761`): a
        // metaclass cannot be made abstract, and any other class takes the
        // keyword by setting the flag `~new`'s `checkAbstract` reads.
        if class.abstract_ {
            if self.classes().is_metaclass(id) {
                let class_id = self.class_id_text(id).as_bytes().to_vec();
                self.blame_directive(program, directive);
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
                self.blame_directive(program, directive);
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
                    self.blame_directive(program, directive);
                    Err(not_found(&target.name).into())
                }
                Err(failure) => {
                    self.blame_directive(program, directive);
                    Err(failure)
                }
            },
        }
    }

    /// One `INHERIT` entry, as the send the oracle makes for it.
    fn inherit_mixin(
        &mut self,
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
        self.blame_native_method(b"INHERIT", &scope);
        self.blame_directive(program, directive);
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
            let library = self.library_binding(external.as_ref(), &name);
            let body = match (generated, native, library) {
                (Some(kind), _, _) => InstallBody::Generated(kind),
                (None, Some(entry), _) => InstallBody::Native(entry),
                (None, None, Some(binding)) => InstallBody::Library(binding),
                (None, None, None) => InstallBody::Written,
            };
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
            let library = self.library_binding(external.as_ref(), &name);
            let body = match (
                generated,
                dispatch::native::bound_entry(external.as_ref(), &name),
                library,
            ) {
                (Some(kind), _, _) => InstallBody::Generated(kind),
                (None, Some(entry), _) => InstallBody::Native(entry),
                (None, None, Some(binding)) => InstallBody::Library(binding),
                (None, None, None) => InstallBody::Written,
            };
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
        if installed == dispatch::MAKESTRING {
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
            InstallBody::Native(entry) => {
                self.external_packages.insert(method, program);
                self.native_externals.insert(method, entry).is_some()
            }
            InstallBody::Library(binding) => {
                self.external_packages.insert(method, program);
                self.library_externals.insert(method, binding).is_some()
            }
        };
        debug_assert!(
            !previous,
            "a MethodId was recorded twice, so one of the two bodies is lost"
        );
    }

    /// Files a body compiled from method source text as a program of its own
    /// and hangs it on the `Method` object, so a send can enter it.
    pub(super) fn record_compiled_body(&mut self, object: ObjRef, name: &[u8], parsed: Program) {
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
        let program_id = ProgramId(self.programs.len());
        self.programs.push(program);
        self.compiled_method_names.insert(program_id, name.into());
        self.table_method_bodies.insert(
            object,
            InstalledMethodBody {
                program: program_id,
                directive: 0,
            },
        );
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Main {
                    program: program_id,
                },
                installed: None,
                routine: None,
            },
        );
    }

    /// [`Interp::record_compiled_body`] for a `Routine`: the same program of
    /// its own, carrying a `::ROUTINE` rather than a `::METHOD`.
    pub(super) fn record_compiled_routine(&mut self, object: ObjRef, name: &[u8], parsed: Program) {
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
                kind: DirectiveKind::Routine(Box::new(rexx_parse::RoutineDirective {
                    name: name.into(),
                    access: Access::default(),
                    external: None,
                    body: Some(main),
                })),
                clause_span: 0..0,
            }],
            symbols,
        });
        let program_id = ProgramId(self.programs.len());
        self.programs.push(program);
        self.compiled_method_names.insert(program_id, name.into());
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Main {
                    program: program_id,
                },
                installed: None,
                routine: Some((program_id, 0)),
            },
        );
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
        parent: Option<Package>,
    ) -> Result<ObjRef, Failure> {
        let path = String::from_utf8_lossy(name).into_owned();
        // **Read through the interpreter's own directory, recorded under the
        // name as given.** `Invocation::with_directory` is what a caller
        // running several interpreters on threads sets, so a relative name
        // must resolve there rather than against the process; but the package
        // keeps the spelling it was asked for -- measured, `~package~name`
        // for `newFile('rel.rex')` answers `rel.rex`.
        let resolved = crate::paths::normalize(&path, &self.cwd_text());
        let Ok(text) = std::fs::read(&resolved) else {
            return Err(Raised::executable_file_unreadable(name).into());
        };
        let parsed = match rexx_parse::parse_program(text) {
            Ok(parsed) => parsed,
            Err(error) => {
                return Err(Loud::method_from_source(&format!(
                    "reporting a file that does not parse ({path}, {error})"
                ))
                .into());
            }
        };
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
                    name: name.into(),
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
        let id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&program));
        self.required_paths.insert(id, path.into());
        // **Recorded before the directives install, not after.** Installing
        // runs a `::REQUIRES` prologue, and code running there resolves
        // routines -- which must already reach the package that built this
        // executable. `load_requires` caches ahead of its own prologue for
        // the same reason.
        let parent = parent.or_else(|| {
            self.running_activation()
                .map(|activation| Package::Program(activation.program_id))
        });
        if let Some(parent) = parent {
            self.package_parents.insert(id, parent);
        }
        self.install_directives(id, &program)?;
        let class = if routine {
            self.routine_class()
        } else {
            self.method_class()
        };
        let object = self.native_instance(class);
        let site = environment::Annotated::Compiled(self.compiled_methods);
        self.compiled_methods += 1;
        self.attach_annotations(object, site);
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Main { program: id },
                installed: None,
                routine: routine.then_some((id, body)),
            },
        );
        if !routine {
            self.table_method_bodies.insert(
                object,
                InstalledMethodBody {
                    program: id,
                    directive: body,
                },
            );
        }
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
            },
        );
    }

    /// Records a `Method` or `Routine` object a `loadExternal*` send answered
    /// over a library procedure, whose package is `code`'s.
    pub(crate) fn record_loaded_executable(&mut self, object: ObjRef, code: usize) {
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Loaded { code },
                installed: None,
                routine: None,
            },
        );
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
            ExecutableSource::Main { program } | ExecutableSource::External { program } => {
                Some(Package::Program(program))
            }
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
            Ok(Some(library)) => {
                let held = self.libraries.hold(name, Rc::new(library));
                self.register_package_routines(name, &held);
                LibraryLoad::Loaded(held)
            }
            Ok(None) => LibraryLoad::Missing,
            Err(refused) => {
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
    fn merge_library(&mut self, id: ProgramId, name: &[u8], library: &rexx_api::load::Library) {
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
        self.merge_routines(id, routines);
    }

    /// Adds each of `routines` to `into`'s imported routines where the name
    /// is not there yet (`HashContents::mergePut`), moving
    /// [`Interp::routine_generation`] when one is added.
    fn merge_routines(&mut self, into: ProgramId, routines: Vec<(Box<[u8]>, MergedRoutine)>) {
        let target = self.merged_public_routines.entry(into).or_default();
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
        }
    }

    /// The library and the procedures one directive's `EXTERNAL` names,
    /// resolved at install time so that a program naming a library that is
    /// not there is refused before its own first clause.
    ///
    /// Answers the procedures it bound, and none for a directive whose
    /// `EXTERNAL` names the `REXX` package or none at all; those are
    /// [`unresolved_external`]'s and [`directive_gap`]'s. `id` is the package
    /// the directive is in, whose file a failure is reported against.
    fn resolve_directive_library(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        directive: &Directive,
    ) -> Result<Vec<LibraryCodeKey>, Failure> {
        let external = match &directive.kind {
            DirectiveKind::Method(method) => dispatch::native::method_external(method),
            DirectiveKind::Attribute(attribute) => dispatch::native::attribute_external(attribute),
            DirectiveKind::Routine(routine) => dispatch::native::routine_external(routine),
            _ => return Ok(Vec::new()),
        };
        let Some(dispatch::native::MethodExternal::OtherLibrary { library, binds }) = external
        else {
            return Ok(Vec::new());
        };
        let routine = matches!(directive.kind, DirectiveKind::Routine(_));
        let loaded = match self.require_library(&library) {
            Ok(loaded) => loaded,
            Err(failure) => {
                self.blame_directive_in(id, program, directive);
                return Err(failure);
            }
        };
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
                self.blame_directive_in(id, program, directive);
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

    /// The `REXX` package routine a `::ROUTINE` directive bound, or `None`
    /// for any other routine.
    pub(crate) fn rexx_routine_row(
        &self,
        installed: InstalledRoutine,
    ) -> Option<&'static internal_routines::InternalRoutine> {
        self.rexx_routine_rows.get(&installed).copied()
    }

    /// The `REXX` package routine a `loadExternalRoutine` answer is, or
    /// `None` for any other object.
    pub(crate) fn rexx_routine_object(
        &self,
        object: ObjRef,
    ) -> Option<&'static internal_routines::InternalRoutine> {
        self.rexx_routine_objects.get(&object).copied()
    }

    /// Records `object` as the `loadExternalRoutine` answer over `row`.
    pub(crate) fn record_rexx_routine_object(
        &mut self,
        object: ObjRef,
        row: &'static internal_routines::InternalRoutine,
    ) {
        self.record_native_executable(object);
        self.rexx_routine_objects.insert(object, row);
    }

    /// The procedure [`Interp::library_codes`] row `code` is.
    pub(crate) fn library_code_key(&self, code: usize) -> &LibraryCodeKey {
        &self.library_code_keys[code]
    }

    /// The file of the package row `code`'s shared code reports, or `None`
    /// while no directive has bound it.
    pub(crate) fn library_code_package_path(&self, code: usize) -> Option<Vec<u8>> {
        let program = self.library_codes.get(code).copied().flatten()?;
        Some(self.package_path(program).as_bytes().to_vec())
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
        library.method(&bind.procedure)?;
        Some(LibraryBinding {
            library,
            procedure: bind.procedure.clone(),
        })
    }

    /// `Method~setPrivate`'s half that a send can see: the dictionary entry
    /// this object *is* stops answering a sender outside its scope.
    pub(crate) fn make_method_private(&mut self, object: ObjRef) {
        let Some(method) = self
            .executable_sources
            .get(&object)
            .and_then(|record| record.installed)
        else {
            return;
        };
        let package = self
            .source_package(self.installed_executable_source(method))
            .unwrap_or(plan::Package::Rexx);
        let row = self.special_method_row(method);
        match row {
            Some(existing) => existing.access = Access::Private,
            None => {
                *self.special_method_row_mut(method) = Some(dispatch::AccessScope {
                    access: Access::Private,
                    protected: false,
                    package,
                });
            }
        }
    }

    /// The access-scope row `method` already has, if it has one.
    pub(crate) fn special_method_row(
        &mut self,
        method: MethodId,
    ) -> Option<&mut dispatch::AccessScope> {
        self.special_methods.get_mut(method.0 as usize)?.as_mut()
    }

    /// The slot `method`'s row lives in, growing the table to reach it.
    pub(crate) fn special_method_row_mut(
        &mut self,
        method: MethodId,
    ) -> &mut Option<dispatch::AccessScope> {
        let index = method.0 as usize;
        if index >= self.special_methods.len() {
            self.special_methods.resize(index + 1, None);
        }
        &mut self.special_methods[index]
    }

    /// One `::ROUTINE` run over the arguments given, for `Routine~call` and
    /// the two rows beside it.
    pub(crate) fn enter_installed_routine(
        &mut self,
        program: ProgramId,
        directive: usize,
        arguments: Vec<Option<ObjRef>>,
    ) -> Result<Option<ObjRef>, Failure> {
        let installed = InstalledRoutine { program, directive };
        self.call_over_installed_routine(installed, arguments)
    }

    /// `PARSE SOURCE`'s third word: a compiled method's own name, the file a
    /// `::REQUIRES` loaded this package from, or the running program's path.
    pub(crate) fn program_display_name(&self, program: ProgramId) -> &[u8] {
        match self.compiled_method_names.get(&program) {
            Some(name) => name,
            None => self.package_path(program).as_bytes(),
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
        *self.special_method_row_mut(method) = Some(dispatch::AccessScope {
            access,
            protected,
            package: Package::Program(program),
        });
    }

    /// Evaluates a `::CONSTANT` directive's parenthesised expression in the
    /// second install pass, and answers what it produced.
    fn eval_constant_expression(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        class: ObjRef,
        expr: &Expr,
    ) -> Result<ObjRef, Failure> {
        let empty_body = CodeBody::default();
        let frame = self.push_directive_activation(id, program);
        // The oracle's own message name for this run, which is the string
        // `GlobalNames::CONSTANT_DIRECTIVE` holds (`memory/GlobalNames.h:84`).
        // The failure path this crate has a witness for does not print it:
        // measured, a condition raised inside a class method the expression
        // calls echoes that method's clause above the `::CONSTANT` and the
        // `::CLASS`, and names no method.
        let saved_context = std::mem::replace(
            &mut self.call_context,
            CallContext {
                name: b"::CONSTANT".to_vec(),
                arguments: Rc::from(&[][..]),
                receiver: Some(class),
            },
        );
        let self_slot = self.slot_of(b"SELF");
        self.set_variable(frame, self_slot, class);
        let super_slot = self.slot_of(b"SUPER");
        // `.nil` for the topmost scope, which is what `superScope` answers
        // there and what `Interp::enter_method_body` writes for it.
        let super_scope = self.classes().class_super_scope(class, class);
        self.set_variable(frame, super_slot, super_scope.unwrap_or(ObjRef::NIL));
        let code = Code {
            body: &empty_body,
            symbols: &program.symbols,
            slots: &[],
            plan: None,
        };
        let result = self.eval(&code, expr);
        self.call_context = saved_context;
        self.pop_directive_activation(frame);
        result
    }

    /// Records `directive`'s own clause as the site a directive-time
    /// condition is reported against, so its report carries the same echo
    /// line the oracle prints above the two `Error` lines.
    fn blame_directive(&mut self, program: &Rc<Program>, directive: &Directive) {
        let (line, text) = directive_clause(program, directive);
        self.failure_site = Some(FailureSite::Clause {
            line,
            text,
            indent: 0,
        });
    }

    /// [`Interp::blame_directive`] for a directive in the package `id`, whose
    /// report names that package's own file when a `::REQUIRES` loaded it.
    fn blame_directive_in(&mut self, id: ProgramId, program: &Rc<Program>, directive: &Directive) {
        let (line, text) = directive_clause(program, directive);
        self.failure_site = Some(match self.required_paths.get(&id) {
            Some(path) => FailureSite::Named {
                line,
                indent: 0,
                text,
                name: path.as_bytes().to_vec(),
            },
            None => FailureSite::Clause {
                line,
                text,
                indent: 0,
            },
        });
    }
}
