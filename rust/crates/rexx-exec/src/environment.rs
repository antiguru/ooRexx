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

//! `.environment`, `.local`, `.context` and `.methods` as objects, the order a
//! `.NAME` resolves in, and the one chokepoint `.environment` and `.local`
//! are read through.
//!
//! # The order, and why the obvious one is wrong
//!
//! `PackageClass::findClass` (`classes/PackageClass.cpp:1081`) is the whole
//! rule, and it consults the **running package's own installed classes
//! first**, ahead of `.local` and ahead of `.environment`. Measured, `::class
//! array` in a user file makes `say .ARRAY` print `The ARRAY class` -- the
//! package's own class, whose id an unquoted directive name upcases -- where a
//! file without that directive prints `The Array class` from the environment.
//! `CoreClasses.orx:47`-`:52` depends on this: it names classes its own file
//! declares later with no `PUBLIC` keyword.
//!
//! The steps this module implements, in the C++'s order: the package's
//! installed classes, `.local`, `.environment`, then the interpreter's own
//! reflection names (`RexxActivation::rexxVariable`,
//! `execution/RexxActivation.cpp:2842`), then the name's own text with a
//! period in front of it.
//!
//! The steps between those that this crate has nothing to consult are named
//! rather than skipped silently: public classes imported from another package
//! (`::REQUIRES`), the `REXX` package's own public classes, and a package
//! local. Each is a table this crate does not build yet, so each is a lookup
//! that would find nothing.
//!
//! # `.NIL`, `.TRUE` and `.FALSE` never arrive here from an expression
//!
//! `LanguageParser`'s constructor builds a `SpecialDotVariable` retriever for
//! each of them (`parser/LanguageParser.cpp:782`-`784`), so the
//! expression form is a parse-time constant that resolves nothing. Measured,
//! `::class True` in a file leaves `say .TRUE` printing `1` -- the package's
//! own class does not shadow it -- while `say value('.TRUE')` in the same file
//! prints `The TRUE class`, because `VALUE`'s one-argument form goes through
//! `VariableDictionary::getVariableRetriever` and gets an ordinary dot
//! variable. So `eval.rs` keeps its own arm for those names and only `VALUE`
//! reaches the environment entries for them.
//!
//! # The names the oracle answers and this crate does not build
//!
//! The fallback -- a name nothing resolves renders as its own uppercased text
//! with a period in front -- is right for a name the oracle does not resolve
//! either, and a **silent wrong answer** for one it does. [`ORACLE_ENVIRONMENT`]
//! and [`ORACLE_LOCAL`] are what `.environment` and `.local` hold, read off
//! the oracle, and every name in them that this crate cannot answer fails
//! loudly instead of falling back.
//!
//! # The object names that come from the prologue
//!
//! `.environment` renders as `The Environment Directory` and `.local` as `The
//! Local Directory`, and neither name is `Setup.cpp`'s: `CoreClasses.orx:55`
//! and `:990` assign them with `~objectName=`. They are built in here because
//! the only oracle this crate can run is one that has already executed both
//! assignments, so matching it byte for byte means carrying their result. The
//! prologue assigning the same strings again when Phase 5a runs it changes
//! nothing.

use std::collections::{HashMap, HashSet};

use rexx_core::{BehaviourId, Body, NativeObject, ObjRef};

use crate::plan::{Package, ProgramId};
use crate::{Failure, Interp, Loud};

/// Which directory a lookup is reading.
///
/// The oracle asks its security manager separately for each --
/// `checkLocalAccess` before `.local` and `checkEnvironmentAccess` before
/// `.environment` (`PackageClass.cpp:1137` and `:1154`) -- so a manager
/// installed in a later phase needs to know which, and the seam carries it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum EnvScope {
    Local,
    Environment,
}

/// The directory-lookup security seam.
///
/// Its own module so that the directory handles and the clearance token have
/// their fields private to the smallest possible scope: nothing outside these
/// lines can name a directory, and nothing outside them can build a
/// clearance, whatever else this file grows.
mod env_seam {
    use super::{EnvScope, Failure, Interp, ObjRef};

    /// `.environment` and `.local`, readable only with an [`Admitted`].
    ///
    /// The fields are private, so [`directory`] is the only expression that
    /// yields either handle -- which is what makes the chokepoint below
    /// unavoidable rather than merely conventional.
    pub(super) struct Directories {
        environment: ObjRef,
        local: ObjRef,
    }

    /// Evidence that a directory lookup passed the security seam.
    ///
    /// Zero-sized, with a private field, and neither `Copy` nor `Clone`.
    /// [`admit`] is the only expression that can produce one and [`directory`]
    /// consumes one, so one trip through the seam yields exactly one directory
    /// handle.
    pub(super) struct Admitted(());

    /// Records the directory handles at bootstrap.
    ///
    /// Building the pair is not reading it, so this takes no clearance.
    pub(super) fn hold(environment: ObjRef, local: ObjRef) -> Directories {
        Directories { environment, local }
    }

    /// **The directory chokepoint (D45, site two).** Every read of `.local`
    /// and of `.environment` passes here, and a manager installed in a later
    /// phase gets its hook in this function's body.
    ///
    /// Each of the oracle's checks returns a *substitute* value when the
    /// manager answers one, so what a later phase adds here is the manager
    /// call and a way to say "answered instead", not a second seam. Nothing is refused in
    /// this phase because there is no manager to refuse anything.
    pub(super) fn admit(
        interp: &mut Interp,
        scope: EnvScope,
        name: &[u8],
    ) -> Result<Admitted, Failure> {
        let _ = (interp, scope, name);
        Ok(Admitted(()))
    }

    /// The directory `scope` names, in exchange for a clearance.
    pub(super) fn directory(held: &Directories, admitted: Admitted, scope: EnvScope) -> ObjRef {
        let _ = admitted;
        match scope {
            EnvScope::Local => held.local,
            EnvScope::Environment => held.environment,
        }
    }
}

/// The root name `.environment` is held alive under. Never a Rexx variable
/// name -- `RootSet::add_global` keys by string and a leading period cannot
/// collide with anything a program writes.
const ENVIRONMENT_ROOT: &str = ".environment";

/// The root name `.local` is held alive under.
const LOCAL_ROOT: &str = ".local";

/// `.environment`'s own contents, read off the oracle.
///
/// Measured by iterating `.environment~supplier` on
/// `/home/moritz/dev/repos/ooRexx/build/bin/rexx` and sorting the indices. It
/// is a running interpreter's answer, so it covers what `Setup.cpp` registers
/// *and* what the shipped `.orx` files add on top -- which is why most of it is
/// unreachable here and has to be loud rather than fall back.
///
/// A name in here that this crate does answer resolves normally; the rest
/// becomes [`EnvironmentModel::unbuilt`] at bootstrap, derived rather than
/// listed a second time.
static ORACLE_ENVIRONMENT: &[&str] = &[
    "ALARM",
    "ALARMNOTIFICATION",
    "ARGUTIL",
    "ARRAY",
    "BAG",
    "BUFFER",
    "CASELESSCOLUMNCOMPARATOR",
    "CASELESSCOMPARATOR",
    "CASELESSDESCENDINGCOMPARATOR",
    "CIRCULARQUEUE",
    "CLASS",
    "COLLECTION",
    "COLUMNCOMPARATOR",
    "COMPARABLE",
    "COMPARATOR",
    "DATETIME",
    "DESCENDINGCOMPARATOR",
    "DIRECTORY",
    "ENDOFLINE",
    "ENVIRONMENT",
    "EVENTSEMAPHORE",
    "FALSE",
    "FILE",
    "IDENTITYTABLE",
    "INPUTOUTPUTSTREAM",
    "INPUTSTREAM",
    "INVERTINGCOMPARATOR",
    "LIST",
    "LOCAL",
    "MAPCOLLECTION",
    "MESSAGE",
    "MESSAGENOTIFICATION",
    "METHOD",
    "MONITOR",
    "MUTABLEBUFFER",
    "MUTEXSEMAPHORE",
    "NIL",
    "NUMERICCOMPARATOR",
    "OBJECT",
    "ORDERABLE",
    "ORDEREDCOLLECTION",
    "OUTPUTSTREAM",
    "PACKAGE",
    "POINTER",
    "PROPERTIES",
    "QUEUE",
    "RELATION",
    "REXXCONTEXT",
    "REXXINFO",
    "REXXQUEUE",
    "ROUTINE",
    "SET",
    "SETCOLLECTION",
    "SINGLETON",
    "STACKFRAME",
    "STEM",
    "STREAM",
    "STREAMSUPPLIER",
    "STRING",
    "STRINGTABLE",
    "SUPPLIER",
    "TABLE",
    "TICKER",
    "TIMESPAN",
    "TRACEOBJECT",
    "TRUE",
    "VALIDATE",
    "VARIABLEREFERENCE",
    "WEAKREFERENCE",
];

/// `.local`'s own contents, read off the oracle the same way
/// [`ORACLE_ENVIRONMENT`] was.
///
/// Every one of them is a standard stream, the external queue, or the command
/// line the interpreter was started with, all built by the system interpreter's
/// own startup -- so this crate builds none of them and each is loud, under a
/// different owning phase from the environment's own.
static ORACLE_LOCAL: &[&str] = &[
    "DEBUGINPUT",
    "ERROR",
    "INPUT",
    "OUTPUT",
    "STDERR",
    "STDIN",
    "STDOUT",
    "STDQUE",
    "SYSCARGS",
    "TRACEOUTPUT",
];

/// `.environment` and `.local`, the classes the other reflection names are
/// built from, and what a name that resolves to none of them owes.
pub(crate) struct EnvironmentModel {
    directories: env_seam::Directories,
    /// A name [`ORACLE_ENVIRONMENT`] or [`ORACLE_LOCAL`] holds that neither
    /// directory here answers, with the phase that owes it.
    unbuilt: HashMap<Box<[u8]>, &'static str>,
    /// `.methods`, `.routines` and `.resources` all answer one of these.
    string_table: ObjRef,
    /// What `.context` answers to.
    context: ObjRef,
}

/// Which directive list a reflection name reports on.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum PackageTable {
    /// `.METHODS` -- the methods attached to no class. `LanguageParser::
    /// addMethod` (`parser/DirectiveParser.cpp:610`) files a method under the
    /// package rather than a class exactly while `activeClass` is still unset,
    /// so these are the method-shaped directives ahead of the first `::CLASS`.
    /// Measured, a lone `::constant sep '/'` and a lone `::attribute zz` each
    /// make `.METHODS` a `StringTable`, and a `::method foo` under a `::class
    /// K` leaves it the string `.METHODS`.
    UnattachedMethods,
    /// `.ROUTINES`.
    Routines,
    /// `.RESOURCES`.
    Resources,
}

impl Interp {
    /// The directory model, built on first use.
    ///
    /// Built here rather than in `Interp::new` for the reason
    /// `dispatch::ObjectModel::bootstrap` is: it forces the native class set,
    /// which a program that never names a `.NAME` must not pay for.
    fn environment_model(&mut self) -> &EnvironmentModel {
        if self.environment.is_none() {
            let model = self.build_environment();
            self.environment = Some(model);
        }
        self.environment
            .as_ref()
            .expect("just built above if it was absent")
    }

    fn build_environment(&mut self) -> EnvironmentModel {
        let directory_class = self
            .classes()
            .lookup("Directory")
            .expect("Directory is a native class");
        let string_table = self
            .classes()
            .lookup("StringTable")
            .expect("StringTable is a native class");
        let context = self
            .classes()
            .lookup("RexxContext")
            .expect("RexxContext is a native class");

        // Rooted the instant it exists and before the next allocation, which
        // can collect: the second `alloc_with` below would otherwise be free
        // to sweep the first object.
        let environment = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Native(Box::new(NativeObject::new(
                directory_class,
                b"The Environment Directory",
            ))),
        );
        self.roots.add_global(ENVIRONMENT_ROOT, environment);
        let local = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Native(Box::new(NativeObject::new(
                directory_class,
                b"The Local Directory",
            ))),
        );
        self.roots.add_global(LOCAL_ROOT, local);

        // `completeSystemClass` (`memory/Setup.cpp:199`) puts every native
        // class into the environment under its uppercased id, and that is the
        // whole of what this crate has to put there beyond the entries
        // `addToEnvironment` adds by hand.
        let classes: Vec<(Box<[u8]>, ObjRef)> = self
            .classes()
            .registered()
            .map(|(name, id)| (name.as_bytes().into(), id))
            .collect();
        // `addToEnvironment` (`Setup.cpp:1730`-`1733`) registers these by
        // hand; `LOCAL` is a method-backed entry there
        // (`Setup.cpp:1781`) answering the running activity's own local
        // directory, and this crate runs one activity, so a plain entry
        // answers the same object.
        let true_value = self.text(b"1");
        let false_value = self.text(b"0");
        let extras: [(&[u8], ObjRef); 5] = [
            (b"ENVIRONMENT", environment),
            (b"LOCAL", local),
            (b"NIL", ObjRef::NIL),
            (b"TRUE", true_value),
            (b"FALSE", false_value),
        ];

        let object = self
            .heap
            .get_mut(environment)
            .expect("just allocated and rooted");
        let Body::Native(native) = &mut object.body else {
            unreachable!("allocated as Body::Native just above")
        };
        for (name, id) in classes {
            native.set_entry(&name, id);
        }
        for (name, value) in extras {
            native.set_entry(name, value);
        }

        let answered: HashSet<&str> = ORACLE_ENVIRONMENT
            .iter()
            .copied()
            .filter(|name| native.entry(name.as_bytes()).is_some())
            .collect();
        // The environment's own unanswered names are Phase 5's: what puts them
        // there is `Setup.cpp` plus the shipped `.orx` files installing, and
        // installing those files is this phase's exit. `.local`'s are Phase
        // 7's: every one of them is a stream, the external queue or the
        // command line, none of which this interpreter has.
        let unbuilt: HashMap<Box<[u8]>, &'static str> = ORACLE_ENVIRONMENT
            .iter()
            .filter(|name| !answered.contains(*name))
            .map(|name| (name.as_bytes().into(), "Phase 5"))
            .chain(
                ORACLE_LOCAL
                    .iter()
                    .map(|name| (name.as_bytes().into(), "Phase 7")),
            )
            .collect();

        EnvironmentModel {
            directories: env_seam::hold(environment, local),
            unbuilt,
            string_table,
            context,
        }
    }

    /// `.NAME`'s value: `PackageClass::findClass`'s order, then
    /// `RexxActivation::rexxVariable`'s reflection names, then the name's own
    /// text.
    ///
    /// `dotted` is the whole symbol including its leading period, already
    /// uppercased -- which both callers have in hand, `eval.rs` because the
    /// scanner interned the symbol that way and `builtin::datatype::value`
    /// because it upcased its argument to classify it.
    pub(crate) fn dot_variable(&mut self, dotted: &[u8]) -> Result<ObjRef, Failure> {
        let bare = dotted.strip_prefix(b".").unwrap_or(dotted);

        if let Some(found) = self.installed_class(bare) {
            return Ok(found);
        }

        // **The one place either directory is read.** The loop is what keeps
        // the chokepoint singular while still asking once per directory, which
        // is what the oracle's own per-directory manager calls do.
        for scope in [EnvScope::Local, EnvScope::Environment] {
            let admitted = env_seam::admit(self, scope, bare)?;
            if let Some(found) = self.directory_entry(admitted, scope, bare) {
                return Ok(found);
            }
        }

        if let Some(found) = self.rexx_variable(bare) {
            return Ok(found);
        }

        if let Some(owner) = self.unbuilt_owner(bare) {
            return Err(Loud::environment_symbol(dotted, owner).into());
        }

        // `variableName->concatToCstring(".")`
        // (`expression/ExpressionDotVariable.cpp:167`, and `:218` for the
        // route `VALUE` takes).
        Ok(self.text(dotted))
    }

    /// A class the running package's own directives installed, under its
    /// uppercased name -- `PackageClass::findInstalledClass`, the first step of
    /// the order.
    ///
    /// Keyed by program, not global: two packages may each declare a class of
    /// the same name and each must see its own.
    fn installed_class(&self, upper: &[u8]) -> Option<ObjRef> {
        let program = self.running_program()?;
        self.package_classes.get(&program)?.get(upper).copied()
    }

    /// The entry `scope`'s directory holds for `name`, if any.
    fn directory_entry(
        &mut self,
        admitted: env_seam::Admitted,
        scope: EnvScope,
        name: &[u8],
    ) -> Option<ObjRef> {
        let handle = {
            let model = self.environment_model();
            env_seam::directory(&model.directories, admitted, scope)
        };
        let object = self.heap.get(handle).expect("a rooted directory");
        let Body::Native(native) = &object.body else {
            unreachable!("both directories are allocated as Body::Native")
        };
        native.entry(name)
    }

    /// `RexxActivation::rexxVariable` (`execution/RexxActivation.cpp:2842`):
    /// the names the interpreter answers out of the running activation rather
    /// than out of a directory.
    ///
    /// `.RS` is deliberately absent. The oracle answers it with the string
    /// `.RS` unless a command has set the return status, and a command clause
    /// is Phase 7's and fails loudly here, so falling through to that same
    /// string is the whole of its behaviour in this phase.
    fn rexx_variable(&mut self, bare: &[u8]) -> Option<ObjRef> {
        match bare {
            b"METHODS" => self.package_string_table(PackageTable::UnattachedMethods),
            b"ROUTINES" => self.package_string_table(PackageTable::Routines),
            b"RESOURCES" => self.package_string_table(PackageTable::Resources),
            // `new_integer(current->getLineNumber())`. `clause_state.line()`
            // is the same quantity `SIGL` reads and carries the same
            // `INTERPRET` rule the oracle's own `isInterpret` delegation
            // gives this name: the enclosing clause's line, not the
            // fragment's.
            b"LINE" => Some(self.counted(self.clause_state.line())),
            b"CONTEXT" => Some(self.context_object()),
            _ => None,
        }
    }

    /// `.METHODS`/`.ROUTINES`/`.RESOURCES`: a `StringTable` when the running
    /// package declares at least one directive of that kind, and nothing at
    /// all when it declares none.
    ///
    /// The absent case is not an empty table: `LanguageParser::
    /// resolveDependencies` hands the package each of these tables only when
    /// it is non-empty (`parser/LanguageParser.cpp:1893`-`1907`, one guarded
    /// assignment per table), so the field stays null and the name falls
    /// through to its own text. Measured, `say .ROUTINES`
    /// prints `.ROUTINES` in a file with no `::ROUTINE` and `a StringTable` in
    /// one with a `::ROUTINE`.
    ///
    /// **The table's entries are not populated.** A value in one is a method
    /// or a routine object, which this phase builds no value for; a program
    /// that indexes one sends `[]` to a `StringTable`, which resolves and then
    /// fails loudly for want of an implementation, so nothing here can answer
    /// wrongly.
    fn package_string_table(&mut self, kind: PackageTable) -> Option<ObjRef> {
        let program = self.running_program()?;
        let program = std::rc::Rc::clone(self.programs.get(program.0)?);
        let mut seen_class = false;
        let declares = program.directives.iter().any(|directive| {
            let unattached = !seen_class;
            if matches!(directive.kind, rexx_parse::DirectiveKind::Class(_)) {
                seen_class = true;
            }
            match kind {
                PackageTable::UnattachedMethods => {
                    unattached
                        && matches!(
                            directive.kind,
                            rexx_parse::DirectiveKind::Method(_)
                                | rexx_parse::DirectiveKind::Attribute(_)
                                | rexx_parse::DirectiveKind::Constant(_)
                        )
                }
                PackageTable::Routines => {
                    matches!(directive.kind, rexx_parse::DirectiveKind::Routine(_))
                }
                PackageTable::Resources => {
                    matches!(directive.kind, rexx_parse::DirectiveKind::Resource(_))
                }
            }
        });
        if !declares {
            return None;
        }
        let class = self.environment_model().string_table;
        Some(self.native_instance(class))
    }

    /// `.CONTEXT`: `RexxActivation::getContextObject`.
    ///
    /// **A fresh object per evaluation where the oracle caches one per
    /// activation**, and the difference is not observable in this phase: a
    /// `RexxContext` answers no method this crate implements, and identity
    /// comparison needs `~==`, which is 5b's. Caching one would need somewhere
    /// to root it for the activation's whole life, and an activation holds no
    /// object references at all today.
    fn context_object(&mut self) -> ObjRef {
        let class = self.environment_model().context;
        self.native_instance(class)
    }

    /// An instance of `class` with no entries, rendered the way
    /// `RexxObject::defaultName` renders one.
    ///
    /// Pushed onto the temporaries stack rather than left unrooted: the value
    /// is returned into an expression that may allocate again before anything
    /// stores it.
    fn native_instance(&mut self, class: ObjRef) -> ObjRef {
        let rendered = default_object_name(self.classes().id_string(class));
        let object = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Native(Box::new(NativeObject::new(class, rendered.as_bytes()))),
        );
        self.roots.push_temp(object);
        object
    }

    /// The phase owing a name the oracle answers and this crate does not
    /// build, or `None` for a name the oracle does not answer either.
    fn unbuilt_owner(&mut self, bare: &[u8]) -> Option<&'static str> {
        self.environment_model().unbuilt.get(bare).copied()
    }

    /// The program whose directives and installed classes a `.NAME` resolves
    /// against -- the running activation's own.
    ///
    /// **A `::CONSTANT` expression is evaluated inside an activation too**
    /// (`Interp::eval_constant_expression` pushes one carrying the installing
    /// program's own id), so it reaches the same table -- but only the part of
    /// it the directives ahead of that `::CONSTANT` have filled in, where the
    /// oracle's class table is complete by the time any directive installs.
    /// Nothing observes the difference in this phase: a `::CONSTANT`'s value is
    /// recorded nowhere, so no send can read one back.
    ///
    /// `None` when no activation is running, which is how a unit test that
    /// resolves a name against a bare `Interp` reaches this.
    pub(crate) fn running_program(&self) -> Option<ProgramId> {
        self.running_activation().map(|frame| frame.program_id)
    }

    /// Records a class a `::CLASS` directive installed, under the running
    /// package's own id.
    ///
    /// Separate from `ClassRegistry`'s flat name table, which models
    /// `.environment`'s class entries and which the oracle never adds a
    /// `::CLASS` to: `completeSystemClass` is an image-build path and
    /// `PackageClass::install` files an installed class against the package
    /// (`addInstalledClass`). Resolution reads this table and the environment
    /// object; it never reads `ClassRegistry::lookup`.
    pub(crate) fn record_package_class(&mut self, program: ProgramId, name: &[u8], class: ObjRef) {
        self.package_classes
            .entry(program)
            .or_default()
            .insert(name.to_ascii_uppercase().into(), class);
        self.class_packages.insert(class, program);
    }

    /// The package object `class~package` answers, built on first use.
    ///
    /// A class this crate's own bootstrap registered belongs to the `REXX`
    /// package; one a `::CLASS` installed belongs to its program's. Measured,
    /// `(.K~package == .Array~package)` is `0` for a `::class K`.
    pub(crate) fn package_object_for(&mut self, class: ObjRef) -> ObjRef {
        // A class this crate's own bootstrap registered is in no program's
        // table, and that absence is what `Package::Rexx` names -- the
        // distinction the enum exists to keep out of an `Option`.
        let package = match self.class_packages.get(&class).copied() {
            None => Package::Rexx,
            Some(program) => Package::Program(program),
        };
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
        object
    }

    /// `Package~name`'s answer for a package object this crate built, or
    /// `None` for a handle [`Interp::package_object_for`] did not produce.
    ///
    /// `REXX` for the primitive classes' package; for a program's own package
    /// the path `PARSE SOURCE`'s third word carries, which is what the oracle
    /// names -- measured, a `::CLASS` in a file answers that file's own
    /// absolute path.
    ///
    /// The `None` is an internal inconsistency and not a program's doing:
    /// `receiver_kind` admits a `Body::Native` as a package by its class, and
    /// this crate builds one only above. Answered rather than panicked, so the
    /// caller can refuse loudly.
    pub(crate) fn package_name(&self, package: ObjRef) -> Option<Vec<u8>> {
        let which = self
            .package_objects
            .iter()
            .find(|(_, object)| **object == package)
            .map(|(which, _)| *which)?;
        Some(match which {
            Package::Rexx => b"REXX".to_vec(),
            // A program's own package. This phase loads one program, so its
            // path is the running program's.
            Package::Program(_) => self.program_path.clone().into_bytes(),
        })
    }
}

/// The [`rexx_core::RootSet::add_global`] key one package object is held
/// under.
///
/// Keyed by string, so each package needs a distinct one; the `REXX` package
/// and each program's are different objects and must not displace each other.
fn package_root_key(package: Package) -> String {
    match package {
        Package::Rexx => "the REXX package".to_string(),
        Package::Program(ProgramId(id)) => format!("the package of program {id}"),
    }
}

/// `RexxObject::defaultName` (`classes/ObjectClass.cpp:1760`): the owning
/// class's id with an article in front, `an` before a vowel and `a`
/// otherwise.
///
/// The C++ has an arm ahead of the article for a behaviour marked *enhanced*,
/// which renders `enhanced <id>` instead. No object this crate builds carries
/// an enhanced behaviour, and `~enhanced` is not a message it answers, so
/// there is nothing here for that arm to describe.
pub(crate) fn default_object_name(id: &str) -> String {
    let vowel = id
        .as_bytes()
        .first()
        .is_some_and(|byte| b"aeiouAEIOU".contains(byte));
    let article = if vowel { "an " } else { "a " };
    format!("{article}{id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The article rule, both ways round, because a version that always says
    /// `a ` renders `a Array` and one that always says `an ` renders `an
    /// StringTable`.
    #[test]
    fn the_default_object_name_picks_its_article_by_the_ids_first_letter() {
        assert_eq!(default_object_name("StringTable"), "a StringTable");
        assert_eq!(default_object_name("Array"), "an Array");
        assert_eq!(default_object_name("RexxContext"), "a RexxContext");
    }

    /// Every name the oracle's `.environment` and `.local` hold either
    /// resolves here or fails loudly -- never the dotted-text fallback, which for one of these
    /// names would be a silent wrong answer at rc 0.
    ///
    /// **The fallback's own direction is asserted beside it**, because a build
    /// that made every unresolved name loud would satisfy the first half and
    /// break `VALUE`'s measured answer for an undefined name.
    #[test]
    fn every_name_the_oracle_answers_resolves_or_is_loud() {
        let mut interp = Interp::new();
        for name in ORACLE_ENVIRONMENT.iter().chain(ORACLE_LOCAL.iter()) {
            let dotted = format!(".{name}").into_bytes();
            match interp.dot_variable(&dotted) {
                Err(Failure::Loud(_)) => {}
                Err(other) => panic!(".{name} failed with {other:?} rather than loudly"),
                Ok(value) => {
                    let text = interp.to_text(value).to_vec();
                    assert_ne!(
                        text, dotted,
                        ".{name} fell back to its own text, which is what the \
                         oracle answers only for a name it does not resolve"
                    );
                }
            }
        }
        let undefined = interp
            .dot_variable(b".FUNNYCONST")
            .expect("a name neither directory holds falls back rather than failing");
        assert_eq!(interp.to_text(undefined).to_vec(), b".FUNNYCONST".to_vec());
    }

    /// The environment holds the classes the registry registered, and the
    /// entries `addToEnvironment` adds by hand.
    #[test]
    fn the_environment_holds_the_native_classes_and_the_hand_registered_entries() {
        let mut interp = Interp::new();
        let array = interp.classes().lookup("Array").expect("Array is native");
        assert_eq!(interp.dot_variable(b".ARRAY").expect("resolves"), array);
        assert_eq!(interp.dot_variable(b".NIL").expect("resolves"), ObjRef::NIL);
        let true_value = interp.dot_variable(b".TRUE").expect("resolves");
        assert_eq!(interp.to_text(true_value).to_vec(), b"1".to_vec());
        let environment = interp.dot_variable(b".ENVIRONMENT").expect("resolves");
        assert_eq!(
            interp.to_text(environment).to_vec(),
            b"The Environment Directory".to_vec()
        );
        // The rendering is an assigned `~objectName` and says nothing about
        // what the object is; its class is what makes it a directory rather
        // than a labelled blob, so both are asserted.
        let directory = interp
            .classes()
            .lookup("Directory")
            .expect("Directory is native");
        let object = interp.heap.get(environment).expect("a rooted directory");
        let Body::Native(native) = &object.body else {
            panic!("the environment is a Body::Native")
        };
        assert_eq!(native.class(), directory);
        let local = interp.dot_variable(b".LOCAL").expect("resolves");
        assert_eq!(
            interp.to_text(local).to_vec(),
            b"The Local Directory".to_vec()
        );
        // `.environment` holds itself, which is what makes the entry above a
        // registration rather than a rendering special case.
        assert_eq!(
            interp.dot_variable(b".ENVIRONMENT").expect("resolves"),
            environment
        );
        assert_ne!(environment, local);
    }
}
