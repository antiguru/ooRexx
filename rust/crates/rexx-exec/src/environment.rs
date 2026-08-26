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

use crate::plan::{ClassPackage, Package, ProgramId};
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

    /// Which of the two `handle` is, or `None` for any other object.
    ///
    /// **Takes no clearance and yields no handle**, which is why it may sit
    /// beside [`directory`] without weakening it: a caller that already holds
    /// an object can learn which directory it is and still has no way to
    /// obtain one it does not hold.
    pub(super) fn which(held: &Directories, handle: ObjRef) -> Option<EnvScope> {
        if handle == held.environment {
            Some(EnvScope::Environment)
        } else if handle == held.local {
            Some(EnvScope::Local)
        } else {
            None
        }
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
    /// A name [`ORACLE_ENVIRONMENT`] or [`ORACLE_LOCAL`] holds that the
    /// directory holding it here does not answer.
    unbuilt: HashMap<Box<[u8]>, Unbuilt>,
    /// `.methods`, `.routines` and `.resources` all answer one of these.
    string_table: ObjRef,
    /// What `.context` answers to.
    context: ObjRef,
}

/// One name the oracle's own directory answers and this crate builds nothing
/// for.
#[derive(Copy, Clone)]
struct Unbuilt {
    /// The phase owing it.
    owner: &'static str,
    /// Which directory holds it, which a message send to one directory has to
    /// know and a `.NAME` lookup -- searching both -- does not. The two lists
    /// are disjoint, so one entry per name is enough: measured, no name in
    /// [`ORACLE_LOCAL`] appears in [`ORACLE_ENVIRONMENT`], which
    /// `the_two_oracle_directories_share_no_name` asserts.
    scope: EnvScope,
}

/// What one `::ANNOTATE` directive's pairs belong to, once the object that
/// carries them exists.
///
/// **One key space rather than one table per kind**, because the readback is
/// one pair of methods whatever the receiver: `memory/Setup.cpp` binds
/// `Annotations`/`Annotation` at `Class` (`:498`), `Method` (`:1111`),
/// `Routine` (`:1140`) and `Package` (`:1172`), and the C++ bodies behind
/// those rows differ from one another only in which field they reach for.
///
/// The variants are what this crate can put a program's hands on. A member is
/// keyed by its class, its dictionary side and its name rather than by the
/// directive that declared it, because that is what
/// `dispatch::native_method` holds when a program asks: `~method` takes a
/// name and a class object and has no directive.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Annotated {
    /// `::ANNOTATE PACKAGE`. The `REXX` package is a variant of [`Package`]
    /// and no `::ANNOTATE` can name it, so it reaches this key only through
    /// `.Array~package~annotations`, whose table starts empty.
    Package(Package),
    /// A class, by the object its `::CLASS` installed. A class this crate's
    /// own bootstrap built is here too, for the same reason.
    Class(ObjRef),
    /// One entry of a class's method dictionary: the class, which side, and
    /// the name.
    Member(ObjRef, bool, Box<[u8]>),
    /// A method-shaped directive ahead of the file's first `::CLASS`, which
    /// attaches to no class -- `.METHODS`'s own key.
    Unattached(ProgramId, Box<[u8]>),
    /// A `::ROUTINE`, by the directive that declares it.
    Routine(ProgramId, usize),
}

/// Which directive list a reflection name reports on.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum PackageTable {
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

/// What one entry of a package table holds.
///
/// The value's own kind is fixed by which table it is in, and each is a value
/// a program can reach: measured with one directive of each kind in a file,
/// `.methods~z` and `.routines~r` render `a Method` and `a Routine`, and
/// `.resources~x~class` is `The Array class`.
enum TableValue {
    /// A `Method` or a `Routine`: the class it answers to, the annotations
    /// it carries, and -- for a `Method` -- which directive of this package
    /// is its body. Nothing here dispatches a `::METHOD` body through
    /// `.METHODS`, and a message neither the class's dictionary nor
    /// `NATIVE_METHODS` holds is 97.1 on either side.
    ///
    /// **The directive is carried for `Class~defineClassMethod`**, which is
    /// the one caller that takes an object out of `.METHODS` and installs it
    /// where it can be sent to: `CoreClasses.orx:73` hands
    /// `.methods[("string_cls_" || name)~upper]` to `.String`. Nothing else
    /// reads it.
    Instance(&'static str, Annotated, Option<(ProgramId, usize)>),
    /// A `::RESOURCE`'s own body lines, which is what `.RESOURCES` holds --
    /// `resources->put(resource, internalname)` over an `ArrayClass`
    /// (`parser/DirectiveParser.cpp:2344`). Measured, a two-line resource's
    /// `~items` is `2` and `SAY` of it prints both lines.
    Lines(Vec<Vec<u8>>),
}

impl Interp {
    /// The directory model, built on first use.
    ///
    /// What is deferred is this model and not the class set it reads: the
    /// classes are already built by the time any program clause runs, since
    /// `Interp::bootstrap_library` builds them before the first one.
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
        // `Setup.cpp:1736`-`:1737`: `REXXINFO` is a pre-built *instance* of a
        // class no environment name reaches, which is why the entry renders
        // as `a RexxInfo` and answers `~class~id` `RexxInfo` where every
        // other entry built from a class renders as `The X class`. Measured
        // on the oracle at rc 0, `.RexxInfo~isA(.Class)` is `0` and
        // `.RexxInfo~class~superClass` is `The Object class`.
        //
        // **Allocated after every other allocation this function makes**, so
        // that nothing between this line and the `set_entry` below can
        // collect it: the environment is the only root it has, and
        // `alloc_with` collects before it allocates.
        //
        // The other direction is safe for a reason of its own rather than by
        // ordering: `true_value` and `false_value` are held in locals across
        // this allocation, and `Interp::text` answers `ObjRef::inline_text`
        // for a one-byte slice, so neither names an arena slot for a
        // collection here to sweep.
        let rexx_info_class = self
            .classes()
            .system_lookup("RexxInfo")
            .expect("RexxInfo is a native class in the kernel directory");
        let rexx_info = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Native(Box::new(NativeObject::new(rexx_info_class, b"a RexxInfo"))),
        );
        let extras: [(&[u8], ObjRef); 6] = [
            (b"ENVIRONMENT", environment),
            (b"LOCAL", local),
            (b"NIL", ObjRef::NIL),
            (b"TRUE", true_value),
            (b"FALSE", false_value),
            (b"REXXINFO", rexx_info),
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
        let unbuilt: HashMap<Box<[u8]>, Unbuilt> = ORACLE_ENVIRONMENT
            .iter()
            .filter(|name| !answered.contains(*name))
            .map(|name| {
                (
                    name.as_bytes().into(),
                    Unbuilt {
                        owner: "Phase 5",
                        scope: EnvScope::Environment,
                    },
                )
            })
            .chain(ORACLE_LOCAL.iter().map(|name| {
                (
                    name.as_bytes().into(),
                    Unbuilt {
                        owner: "Phase 7",
                        scope: EnvScope::Local,
                    },
                )
            }))
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

        if let Some(found) =
            self.directory_lookup(&[EnvScope::Local, EnvScope::Environment], bare)?
        {
            return Ok(found);
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

    /// The class a `::CLASS` directive's `SUBCLASS`, `INHERIT` or `METACLASS`
    /// keyword names, when the file's own directives do not declare it.
    ///
    /// `PackageClass::findClass`'s order (`classes/PackageClass.cpp:1081`),
    /// **with the steps this crate has nothing to consult named rather than
    /// skipped silently**: installed classes, then the package's imported
    /// public classes, then `TheRexxPackage`'s public classes, then the
    /// package local, then the directories. The **imported** public classes
    /// need `::REQUIRES`, which is Phase 5c's, and the package local needs
    /// `Package~local`, which nothing here builds. **`TheRexxPackage`'s
    /// public classes are substituted for rather than skipped**:
    /// `MemoryObject::completeSystemClass` (`memory/Setup.cpp:199`-`:206`)
    /// puts every system class into `TheEnvironment` *and* into
    /// `TheRexxPackage` in the same two lines, so the `.environment` step
    /// answers what that one would. What is left is the running package's
    /// installed classes, then `.environment`, then the native name table.
    ///
    /// **No program can see the difference today**: measured, a two-file
    /// probe -- `::requires 'dep.rex'` with a public `::class Comparable` in
    /// the dependency and `::class K subclass Comparable` in the main file
    /// -- is `rexx-exec: ::REQUIRES is not implemented (Phase 5)` here where
    /// the oracle resolves through the imported class.
    ///
    /// **`.environment` is the step the interpreter's own library needs and a
    /// program rarely does.** `StreamClasses.orx:506` inherits `Comparable`,
    /// which `CoreClasses.orx` declares and whose prologue puts in
    /// `.environment`; the two are separate packages, so nothing but that
    /// directory connects them. Before the bootstrap ran, every class in
    /// `.environment` was one the native name table already answered, so this
    /// step changed no answer a program could get.
    ///
    /// A directory entry that is not a class object is stepped over rather
    /// than returned, so a `::CLASS K SUBCLASS ENDOFLINE` still gets its
    /// 98.909 rather than a class-shaped failure further on.
    pub(crate) fn directive_class(&mut self, upper: &[u8]) -> Option<ObjRef> {
        if let Some(found) = self.installed_class(upper) {
            return Some(found);
        }
        // `.environment` alone, not `.NAME`'s pair: `ClassDirective`'s own
        // search is the package's classes and then the environment
        // directory, and `.local` is not in it.
        if let Ok(Some(found)) = self.directory_lookup(&[EnvScope::Environment], upper)
            && found.class_id().is_some()
        {
            return Some(found);
        }
        // A name that answers something which is not a class, and a name
        // whose directory entry this crate has not built, both fall through
        // to the native table -- which is where a `::CLASS` target was
        // resolved before `.environment` was ever consulted, so a miss is the
        // same 98.909 it was.
        self.classes().lookup(&String::from_utf8_lossy(upper))
    }

    /// The first of `scopes` whose directory holds `bare`, or the refusal an
    /// unbuilt entry carries.
    ///
    /// **The one place either directory is read.** The loop is what keeps
    /// the chokepoint singular while still asking once per directory, which
    /// is what the oracle's own per-directory manager calls do, and
    /// `tests/environment_seam.rs` asserts that `env_seam::admit` has this
    /// one call site.
    fn directory_lookup(
        &mut self,
        scopes: &[EnvScope],
        bare: &[u8],
    ) -> Result<Option<ObjRef>, Failure> {
        for &scope in scopes {
            let admitted = env_seam::admit(self, scope, bare)?;
            if let Some(found) = self.directory_entry(admitted, scope, bare) {
                return Ok(Some(found));
            }
        }
        Ok(None)
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
    /// **One object per package, not one per evaluation**, which the oracle's
    /// own identity says: measured, `.methods~identityHash` answers the same
    /// number twice in a row. `.CONTEXT` beside this is the other way round
    /// and says why -- see [`Interp::context_object`].
    ///
    /// The entries are the directives themselves, keyed by their upcased
    /// spelling: `unattachedMethods->setEntry`
    /// (`parser/DirectiveParser.cpp:617`) upcases, and so do `.ROUTINES`'s
    /// and `.RESOURCES`'s own keys. Measured, `::method "MiXeD"` puts `MIXED`
    /// in `.METHODS` and `::routine "r"` puts `R` in `.ROUTINES`.
    fn package_string_table(&mut self, kind: PackageTable) -> Option<ObjRef> {
        let program = self.running_program()?;
        if let Some(found) = self.package_tables.get(&(program, kind)).copied() {
            return Some(found);
        }
        let source = std::rc::Rc::clone(self.programs.get(program.0)?);
        let entries = package_table_entries(program, &source, kind);
        if entries.is_empty() {
            return None;
        }
        let class = self.environment_model().string_table;
        // The table is rooted before anything it will hold is allocated, and
        // each value goes into it as soon as it exists: `alloc_with` collects
        // before it allocates, and a `Body::Native`'s entries are traced, so
        // a value stored here survives the next value's allocation.
        let table = self.native_instance(class);
        self.roots
            .add_global(&package_table_root_key(program, kind), table);
        self.package_tables.insert((program, kind), table);
        let frame = self.roots.push_frame();
        for (name, value) in entries {
            let value = match value {
                TableValue::Instance(id, site, body) => {
                    let class = self
                        .classes()
                        .lookup(id)
                        .expect("every TableValue::Instance names a native class");
                    let object = self.native_instance(class);
                    self.attach_annotations(object, site);
                    if let Some((program, directive)) = body {
                        self.table_method_bodies
                            .insert(object, crate::InstalledMethodBody { program, directive });
                    }
                    object
                }
                TableValue::Lines(lines) => self.line_array(&lines),
            };
            let object = self.heap.get_mut(table).expect("just allocated and rooted");
            let Body::Native(native) = &mut object.body else {
                unreachable!("allocated as Body::Native by native_instance")
            };
            native.set_entry(&name, value);
        }
        self.roots.pop_frame(frame);
        Some(table)
    }

    /// One `::RESOURCE`'s body as the `Array` of strings `.RESOURCES` holds.
    ///
    /// Each line is pushed as a temporary as it is built, because the next
    /// line's own allocation may collect and nothing else holds it yet.
    fn line_array(&mut self, lines: &[Vec<u8>]) -> ObjRef {
        let mut slots = Vec::with_capacity(lines.len());
        for line in lines {
            let text = self.text(line);
            self.roots.push_temp(text);
            slots.push(Some(text));
        }
        let array = self.alloc_with(BehaviourId::ARRAY, Body::Array(slots));
        self.roots.push_temp(array);
        array
    }

    /// `.CONTEXT`: `RexxActivation::getContextObject`, which builds the
    /// object on the first ask and keeps it in the activation's own field.
    ///
    /// **One object per activation, which is observable and is measured.**
    /// Oracle rc 0: `c = .context` then
    /// `(c~identityHash == .context~identityHash)` is `1`, and
    /// `.context~objectName = "tagged"` then `say .context~objectName` prints
    /// `tagged`. The same comparison against a context passed into a method
    /// is `0`, so the object is the activation's and not the program's.
    ///
    /// [`Activation::context_object`] is where it lives and carries how it is
    /// rooted in each of the two states an activation can be in.
    ///
    /// The `None` arm is a resolution with no activation running, which is how
    /// a unit test against a bare `Interp` reaches this; nothing a program can
    /// write does, since `.CONTEXT` is resolved from inside the activation
    /// evaluating it.
    fn context_object(&mut self) -> ObjRef {
        if let Some(found) = self.running_activation().and_then(|a| a.context_object) {
            return found;
        }
        let class = self.environment_model().context;
        let object = self.native_instance(class);
        if let Some(activation) = self.running.as_deref_mut() {
            activation.context_object = Some(object);
        }
        object
    }

    /// An instance of `class` with no entries, rendered the way
    /// `RexxObject::defaultName` renders one.
    ///
    /// Pushed onto the temporaries stack rather than left unrooted: the value
    /// is returned into an expression that may allocate again before anything
    /// stores it.
    pub(crate) fn native_instance(&mut self, class: ObjRef) -> ObjRef {
        let rendered = default_object_name(self.classes().id_string(class));
        let object = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Native(Box::new(NativeObject::new(class, rendered.as_bytes()))),
        );
        self.roots.push_temp(object);
        object
    }

    /// The entry `index` names on a `Directory` or a `StringTable`, or the
    /// refusal for an index whose entry the oracle has and this crate does
    /// not.
    ///
    /// **`.nil` is the answer for an absent entry and a wrong answer for an
    /// unbuilt one**, which is the same split [`Interp::dot_variable`] makes
    /// one step further down: measured, `.local['STDOUT']` is `STDOUT` on the
    /// oracle and `.environment['STDOUT']` is `The NIL object`, so the refusal
    /// has to be per directory rather than over the union of the two names.
    /// [`Unbuilt::scope`] is what carries that.
    ///
    /// **Not through the seam.** `env_seam::admit` is what a `.NAME` lookup
    /// passes to *find* a directory it was not handed; a send already holds the
    /// receiver, and the oracle asks its manager per `getLocal`/`getEnvironment`
    /// call (`PackageClass.cpp:1137`, `:1154`) and not per `~at`.
    pub(crate) fn hash_entry_read(
        &mut self,
        receiver: ObjRef,
        index: &[u8],
    ) -> Result<ObjRef, Failure> {
        if let Some(found) = self.native_entry(receiver, index) {
            return Ok(found);
        }
        // The unbuilt refusal is a directory's alone: `directory_scope`
        // answers `None` for a package table, whose entries this crate builds
        // in full.
        if let Some(scope) = self.directory_scope(receiver)
            && let Some(unbuilt) = self.environment_model().unbuilt.get(index).copied()
            && unbuilt.scope == scope
        {
            return Err(Loud::environment_entry(index, unbuilt.owner).into());
        }
        Ok(ObjRef::NIL)
    }

    /// Stores `item` under `index` on a `Directory` or a `StringTable`.
    ///
    /// The entry replaces whatever the bootstrap put there, which is what makes
    /// a stored name answer where the unbuilt refusal above would otherwise
    /// fire: `hash_entry_read` asks the map first, exactly as
    /// [`Interp::dot_variable`] does.
    pub(crate) fn hash_entry_write(
        &mut self,
        receiver: ObjRef,
        index: &[u8],
        item: ObjRef,
    ) -> Result<(), Failure> {
        let Some(object) = self.heap.get_mut(receiver) else {
            return Err(Loud::receiver_class("a value whose object is no longer live").into());
        };
        let Body::Native(native) = &mut object.body else {
            return Err(Loud::receiver_class("a value that is not a hash collection").into());
        };
        native.set_entry(index, item);
        Ok(())
    }

    /// Every key a `Body::Native`'s own map holds, or an empty list for any
    /// other object. Owned, and in no particular order -- see
    /// [`rexx_core::NativeObject::keys`].
    pub(crate) fn native_keys(&self, object: ObjRef) -> Vec<Box<[u8]>> {
        match self.heap.get(object).map(|held| &held.body) {
            Some(Body::Native(native)) => native.keys(),
            _ => Vec::new(),
        }
    }

    /// One entry of a `Body::Native`'s own map, by the key the caller holds.
    pub(crate) fn native_entry(&self, object: ObjRef, index: &[u8]) -> Option<ObjRef> {
        match &self.heap.get(object)?.body {
            Body::Native(native) => native.entry(index),
            _ => None,
        }
    }

    /// Which of the two directories this model built `directory` is, or `None`
    /// for any other object.
    ///
    /// Reads the handles without a clearance, which is what
    /// [`env_seam::Directories`]'s privacy allows and its doc intends: the
    /// question is "is this handle one of those two", not "give me one of
    /// those two", and answering it hands out neither.
    fn directory_scope(&mut self, directory: ObjRef) -> Option<EnvScope> {
        let model = self.environment_model();
        env_seam::which(&model.directories, directory)
    }

    /// The phase owing a name the oracle answers and this crate does not
    /// build, or `None` for a name the oracle does not answer either.
    fn unbuilt_owner(&mut self, bare: &[u8]) -> Option<&'static str> {
        Some(self.environment_model().unbuilt.get(bare)?.owner)
    }

    /// The phase owing the entries of `object` that this crate does not
    /// build, or `None` for a collection whose entries it fills.
    ///
    /// **The question [`Interp::hash_entry_read`] asks per name, asked about
    /// the whole collection**, for a caller that walks the entries instead of
    /// reading one. Walking a `Body::Native`'s own map answers what this
    /// crate put there, which for `.local` is nothing at all: every entry it
    /// has on the oracle is in the `unbuilt` table, so a walk sees an empty
    /// collection and a caller that acts on what it sees does nothing at all
    /// where the oracle acts.
    ///
    /// The owner is the smallest of the owners in that scope, so the refusal
    /// names one phase rather than depending on a `HashMap`'s order.
    pub(crate) fn unbuilt_collection_owner(&mut self, object: ObjRef) -> Option<&'static str> {
        let scope = self.directory_scope(object)?;
        self.environment_model()
            .unbuilt
            .values()
            .filter(|entry| entry.scope == scope)
            .map(|entry| entry.owner)
            .min()
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
    pub(crate) fn record_package_class(
        &mut self,
        program: ProgramId,
        name: &[u8],
        class: ObjRef,
        public: bool,
    ) {
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
    ///
    /// Separate from [`Interp::record_package_class`] because the two write
    /// different tables: a class built by message goes into no package's
    /// class table at all. Measured, oracle rc 0:
    /// `.context~package~classes~items` is `0` both before and after
    /// `k = .object~subclass("k")`.
    pub(crate) fn record_packageless_class(&mut self, class: ObjRef) {
        self.class_packages.insert(class, ClassPackage::Null);
    }

    /// `Package~addClass` and `Package~addPublicClass`, which differ only in
    /// whether the public table gets the entry too --
    /// `PackageClass::addInstalledClass` (`classes/PackageClass.cpp:1401`),
    /// which both `addClassRexx` (`:1932`) and `addPublicClassRexx`
    /// (`:1950`) reach with the flag set differently.
    ///
    /// The name is stored upcased, because `setEntry` upcases its index
    /// (`StringHashCollection::setEntry`, `classes/support/HashCollection.cpp:854`).
    /// Measured on the oracle at rc 0: after `p~addClass("zz", .K)`,
    /// `p~classes["ZZ"]` is `The K class` and `p~classes["zz"]` is `The NIL
    /// object`.
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

    /// `Package~publicClasses` for a program's own package: a fresh
    /// `StringTable` holding what the `::CLASS ... PUBLIC` directives and
    /// `~addPublicClass` have put there.
    ///
    /// **Fresh on every ask, not one kept table**, which is the oracle's own
    /// answer: `getPublicClassesRexx` returns `installedPublicClasses->copy()`
    /// (`classes/PackageClass.cpp:1570`) or a new empty one. Measured at rc
    /// 0, `p~publicClasses == p~publicClasses` is `0` where
    /// `.context~package == .context~package` is `1`.
    ///
    /// The table is filled in sorted name order, and **that order is
    /// observable**: `DO OVER` a `StringTable` iterates its indexes, and
    /// `Interp::hash_collection_indexes` carries what this crate's order
    /// costs against the oracle's. Sorting is also what keeps the allocation
    /// sequence the same from run to run.
    pub(crate) fn public_classes_table(&mut self, program: ProgramId) -> ObjRef {
        let class = self.environment_model().string_table;
        let table = self.native_instance(class);
        let mut entries: Vec<(Box<[u8]>, ObjRef)> = self
            .package_public_classes
            .get(&program)
            .map(|held| held.iter().map(|(k, v)| (k.clone(), *v)).collect())
            .unwrap_or_default();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let object = self.heap.get_mut(table).expect("just allocated and rooted");
        let Body::Native(native) = &mut object.body else {
            unreachable!("allocated as Body::Native by native_instance")
        };
        // Every value is a class identity, which names no arena slot, so
        // nothing here can be collected between two of these writes and the
        // table needs no per-value rooting.
        for (name, id) in entries {
            native.set_entry(&name, id);
        }
        table
    }

    /// The package object `class~package` answers, or `.nil` for a class that
    /// belongs to no package.
    ///
    /// A class this crate's own bootstrap registered belongs to the `REXX`
    /// package; one a `::CLASS` installed belongs to its program's. Measured,
    /// `(.K~package == .Array~package)` is `0` for a `::class K`.
    ///
    /// **`.nil` is the third answer and it is the oracle's**, for a class
    /// `~subclass` or `~mixinClass` built: those forward `OREF_NULL` as the
    /// package (`classes/ClassClass.cpp:1546`, `:1496`) and `getPackage`
    /// answers `resultOrNil` of the field. Measured, oracle rc 159 with
    /// stdout empty: `k = .object~subclass("k")` then `say k~package~name`
    /// reports `Object "The NIL object" does not understand message "NAME".`
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
    ///
    /// Split out of [`Interp::package_object_for`] because
    /// `RexxContext~package` names the running program's package with no
    /// class to read it off, and the two must answer the same object:
    /// measured, `.context~package == .K~package` is `1` for a `::class K`
    /// in the running file.
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
    ///
    /// **One object per dictionary entry, because the oracle's identity is
    /// observable.** `RexxClass::method` retrieves the method out of
    /// `instanceMethodDictionary` and answers it
    /// (`classes/ClassClass.cpp:984`, the retrieval at `:991`), so two sends
    /// of `~method` for one name answer one object. Measured, oracle rc 0
    /// and both engines identical:
    /// `(.K~method("M")~identityHash == .K~method("M")~identityHash)` is `1`,
    /// and `.K~method("M")~objectName = "x"` then `say .K~method("M")` prints
    /// `x`. A fresh object per send answers `0` and `a Method`.
    ///
    /// The comparison is `==` and not `=`. The oracle's `identityHash` is an
    /// address-derived integer of more than nine digits, and `=` compares two
    /// of those at `NUMERIC DIGITS`, where they round to the same value, so
    /// two genuinely different objects compare equal under it. Measured with
    /// the two answers in variables, which is how they arrive from
    /// `~identityHash`: `a = "-140404878001713"; b = "-140404878167489"` then
    /// `say (a = b) (a == b)` prints `1 0`.
    ///
    /// **Written as variables because the literal form measures something
    /// else.** Unary minus is arithmetic, so `say (-140404878001713 ==
    /// -140404878167489)` evaluates each literal at `NUMERIC DIGITS` first
    /// and compares two copies of `-1.40404878E+14`: it prints `1`, and it is
    /// not this rule.
    ///
    /// Rooted as a global for the reason a package object is: it outlives
    /// every send that reaches it and is reachable from no other object
    /// between two of them.
    ///
    /// `scope` is the scope the dictionary entry was defined at, which is
    /// what `Method~scope` answers -- see [`Interp::method_scope`]. The
    /// caller supplies it rather than this function assuming `class`,
    /// because `MethodDictionary::setMethodScope` rewrites a dictionary's
    /// entries to another class's scope and `~inheritInstanceMethods` runs it
    /// over the donor's own dictionary (`classes/ClassClass.cpp:560`-`:563`).
    pub(crate) fn method_object(&mut self, class: ObjRef, name: &[u8], scope: ObjRef) -> ObjRef {
        let key = (class, Box::<[u8]>::from(name));
        if let Some(found) = self.method_objects.get(&key).copied() {
            return found;
        }
        let method_class = self.method_class();
        let object = self.native_instance(method_class);
        let held = self.heap.get_mut(object).expect("just allocated");
        let Body::Native(native) = &mut held.body else {
            unreachable!("allocated as Body::Native by native_instance")
        };
        native.set_scope(scope);
        self.roots
            .add_global(&method_object_root_key(class, name), object);
        self.method_objects.insert(key, object);
        // The dictionary entry the caller found, which is what its
        // annotations are keyed by: this class, the instance side, this name.
        self.attach_annotations(object, Annotated::Member(class, false, name.into()));
        object
    }

    /// The package object of the program that is running -- what
    /// `RexxContext~package` answers, `RexxContext::getPackage`
    /// (`classes/ContextClass.cpp:160`, bound by `memory/Setup.cpp:1218`).
    ///
    /// `None` when no activation is running, which is the position
    /// [`Interp::running_program`] is in and is how a unit test against a
    /// bare `Interp` reaches this.
    ///
    /// **The same object `~package` answers for a class the program
    /// declared**, because both go through [`Interp::package_object_for`]'s
    /// cache under one key. Measured, oracle rc 0: with `::class K public`,
    /// `.context~package == .K~package` is `1`,
    /// `.context~package == .context~package` is `1`, and
    /// `.context~package == .Array~package` is `0`.
    pub(crate) fn running_package_object(&mut self) -> Option<ObjRef> {
        let program = self.running_program()?;
        Some(self.package_object(Package::Program(program)))
    }

    /// `MethodClass::newScope` (`classes/MethodClass.cpp:183`): the same
    /// method object with `scope` filled in when it had none, and a copy
    /// carrying `scope` when it already had one.
    ///
    /// This is what decides whether `~define` stores the very object it was
    /// handed. Measured, oracle rc 0, with `::method z` above `::class K`
    /// and `::class K2`: `m = .methods~z; .K2~define("Y", m)` then
    /// `m == .K2~method("Y")` is `1`, and `.K2~define("X", .K~method("M"))`
    /// then `.K~method("M") == .K2~method("X")` is `0` -- `.K`'s own method
    /// already carries `.K` as its scope.
    ///
    /// The copy is shallow, which is `RexxObject::copy`: it shares the
    /// annotation table rather than duplicating it, so the copy answers the
    /// `::ANNOTATE` pairs the original was given. Measured at rc 0,
    /// `.K2~method("X")~annotation("A")` answers what `::annotate method m A`
    /// set.
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
        Some(object)
    }

    /// `~define` with a method object: install it in `class`'s own instance
    /// dictionary under `name` and make [`Interp::method_object`] answer it.
    ///
    /// `RexxClass::defineMethod` (`classes/ClassClass.cpp:819`) puts the
    /// object `newMethodObject` gave it straight into the dictionary
    /// (`:864`), so `~method` afterwards answers that object and not a fresh
    /// one -- which is the difference `~defineMethods` beside it does not
    /// have.
    pub(crate) fn define_method_object(
        &mut self,
        class: ObjRef,
        name: &[u8],
        source: ObjRef,
    ) -> Option<()> {
        let object = self.method_new_scope(source, class)?;
        let method = self.classes().mint_method_id();
        self.classes()
            .define_instance_method(class, &String::from_utf8_lossy(name), method);
        self.hold_method_object(class, name, object);
        Some(())
    }

    /// `defineClassMethod`: the same shape as [`Interp::define_method_object`]
    /// on the class side, plus the row that makes the installed method
    /// runnable.
    ///
    /// **The body row is the difference and it is load-bearing.** `~define`
    /// mints an id and stores the object; a send to that id finds no body,
    /// which is right there because the oracle's `~define` reaches an
    /// instance side no class object answers from. `defineClassMethod`
    /// installs where a send *does* land -- `.String~NL` after
    /// `CoreClasses.orx:73` -- so the minted id has to name the directive the
    /// object came from. `None` is a method object this crate handed out
    /// with no directive behind it, which the caller refuses.
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
    ///
    /// Each entry goes through [`Interp::method_new_scope`] **twice**, which
    /// is the oracle's own path and not a doubling:
    /// `createMethodDictionary` calls `newMethodObject`
    /// (`classes/ClassClass.cpp:1265`), and `replaceMethods` calls
    /// `newScope` again on what that produced (`MethodDictionary.cpp:233`).
    /// The second call always finds a scope set by the first, so the object
    /// stored is always a copy -- measured, oracle rc 0:
    /// `m = .methods~z; .K~defineMethods(.methods)` then
    /// `m == .K~method("Z")` is `0` while `m == .methods~z` is `1`.
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
            installed.push((name_text, Some(self.classes().mint_method_id())));
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
        self.roots
            .add_global(&method_object_root_key(class, name), object);
        self.method_objects.insert((class, name.into()), object);
    }

    /// Forget the `Method` object this dictionary entry answered, for a
    /// `~delete` or a `~define` that took the entry away.
    ///
    /// The identity `~method` hands out is per dictionary entry, so an entry
    /// that stops existing must not leave its object behind to be answered by
    /// whatever occupies the name next.
    ///
    /// **The map entry goes and the global root stays.** `RootSet` has no
    /// counterpart to `add_global`, so the object [`Interp::hold_method_object`]
    /// rooted stays reachable for the rest of the run. That costs one live
    /// object per (class, name) pair a program ever installs and answers
    /// nothing: `~method` reads this map, and it no longer has the entry. A
    /// later `~define` under the same name replaces the root as well as the
    /// entry, because `add_global` replaces by name.
    pub(crate) fn drop_method_object(&mut self, class: ObjRef, name: &[u8]) {
        self.method_objects.remove(&(class, name.into()));
    }

    /// The `StringTable` `~annotations` answers for `site`, built empty on
    /// first ask and kept.
    ///
    /// **Kept, because the oracle's is a live table and not a snapshot**:
    /// `RexxClass::getAnnotations` and `BaseExecutable::getAnnotations` both
    /// create the table on the first ask and store it in the object's own
    /// field (`classes/ClassClass.cpp:325`, `execution/BaseExecutable.cpp:378`).
    /// Measured, oracle rc 0 in each of three shapes: `.K~annotations~put('v',
    /// 'N')` then `.K~annotation('N')` answers `v`, and so do the same pair
    /// through `.K~method('M')` and through `.routines~r`, where a build
    /// answering a fresh table each time answers `The NIL object`.
    ///
    /// **Rooted as a global**, the position `.environment` and a package
    /// object are in: the table outlives every send that reaches it, and the
    /// objects that carry a handle on it are rooted the same way, so nothing
    /// on the temporaries stack keeps it alive between two sends.
    pub(crate) fn annotation_table(&mut self, site: Annotated) -> ObjRef {
        if let Some(found) = self.annotations.get(&site).copied() {
            return found;
        }
        let class = self.environment_model().string_table;
        let table = self.native_instance(class);
        self.roots.add_global(&annotation_root_key(&site), table);
        self.annotations.insert(site, table);
        table
    }

    /// Records what one `::ANNOTATE` directive named, under every key that
    /// reaches it.
    ///
    /// More than one key where a `::CONSTANT` is annotated: one directive
    /// files a single method object in both of its class's dictionaries
    /// (`instructions/ClassDirective.cpp:520`-`:524`), so both sides answer
    /// the same table rather than two tables that could come to disagree.
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
    ///
    /// **A handle on the table [`Interp::annotation_table`] keeps, never a
    /// copy of it.** More than one object can name one table -- a
    /// `::CONSTANT`'s single method is filed on both sides of its class's
    /// dictionary -- and a program adding to the table through any of them
    /// must be answered through all of them.
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
    ///
    /// A class object answers from [`Interp::annotations`] directly, because
    /// a class handle is the key; every other carrier answers the handle it
    /// was built with. The refusal is an internal inconsistency rather than a
    /// program's doing -- `NATIVE_METHODS` binds `ANNOTATION` and
    /// `ANNOTATIONS` at `Class`, `Method`, `Routine` and `Package` alone, and
    /// a `Method`, a `Routine` and a `Package` object each get their table as
    /// this crate builds them.
    ///
    /// [`Interp::annotations`]: Interp::annotations
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
    ///
    /// `MethodClass::getScopeRexx` (`classes/MethodClass.cpp:361`) is
    /// `resultOrNil(getScope())`, bound at `Method` by `memory/Setup.cpp:1113`,
    /// so the absent scope is an answer and not a raise. Measured, oracle
    /// rc 0: an unattached `::METHOD z` reached through `.methods~z~scope`
    /// prints `The NIL object`, and the same object answers `K2` once
    /// `.K2~define("Y", ...)` has taken it.
    ///
    /// The refusal is an internal inconsistency rather than a program's
    /// doing: a receiver reaches here only by resolving `SCOPE` at `Method`,
    /// and a method object is a `Body::Native`. Loud rather than `.nil`, so
    /// a receiver that is neither cannot pass for one that carries no scope.
    pub(crate) fn method_scope(&self, receiver: ObjRef) -> Result<ObjRef, Failure> {
        match self.heap.get(receiver).map(|held| &held.body) {
            Some(Body::Native(native)) => Ok(native.scope().unwrap_or(ObjRef::NIL)),
            _ => Err(Loud::receiver_class("a value that carries no method scope").into()),
        }
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
        Some(match self.which_package(package)? {
            Package::Rexx => crate::LIBRARY_PACKAGE_NAME.to_vec(),
            // A program's own package, answered as the *running* program's
            // path and not as that program's own.
            //
            // **A run loads more than one program**: the interpreter's own
            // library is three of them, and each has a package object of its
            // own. What keeps that from being a wrong answer is that none of
            // those objects is reachable from a program -- a program's
            // `.context~package` is its own, and a class the library
            // installed answers `Package::Rexx` because
            // `Interp::record_package_class` leaves it out of
            // `class_packages`. The `ProgramId` is discarded here rather
            // than looked up because `Interp::programs` holds no path per
            // program; the day one of those objects becomes reachable, this
            // is the line that has to grow one.
            Package::Program(_) => self.program_path.clone().into_bytes(),
        })
    }

    /// Which package a package object stands for, or `None` for a handle
    /// [`Interp::package_object`] did not produce.
    pub(crate) fn which_package(&self, package: ObjRef) -> Option<Package> {
        self.package_objects
            .iter()
            .find(|(_, object)| **object == package)
            .map(|(which, _)| *which)
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

/// The [`rexx_core::RootSet::add_global`] key one `Method` object is held
/// under.
fn method_object_root_key(class: ObjRef, name: &[u8]) -> String {
    format!(
        "the instance method {} of the class at {}",
        String::from_utf8_lossy(name),
        class.bits()
    )
}

/// The [`rexx_core::RootSet::add_global`] key one annotation table is held
/// under.
///
/// Spelled so that no two [`Annotated`] keys can collide: a class handle and
/// a program id are numbers from different spaces, so each variant names
/// itself as well as its parts.
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
    }
}

/// The [`rexx_core::RootSet::add_global`] key one package table is held under.
///
/// A program's `.METHODS`, `.ROUTINES` and `.RESOURCES` are distinct objects
/// and must not displace each other, so the kind is in the key alongside the
/// program.
fn package_table_root_key(ProgramId(program): ProgramId, kind: PackageTable) -> String {
    let which = match kind {
        PackageTable::UnattachedMethods => "methods",
        PackageTable::Routines => "routines",
        PackageTable::Resources => "resources",
    };
    format!("the {which} of program {program}")
}

/// What `kind`'s table holds for `program`, keyed the way the oracle keys it.
///
/// Empty for a package that declares no directive of that kind, which is the
/// state `.METHODS` renders as its own text in --
/// [`Interp::package_string_table`] is where that distinction is read.
fn package_table_entries(
    id: ProgramId,
    program: &rexx_parse::Program,
    kind: PackageTable,
) -> Vec<(Vec<u8>, TableValue)> {
    // The body is carried for a written `::METHOD` alone. An `::ATTRIBUTE`
    // and a `::CONSTANT` file generated accessors, whose bodies are
    // `Interp::generated_methods` rather than `Interp::method_bodies`, and
    // handing one of those to `Class~defineClassMethod` would install a row
    // naming a body of the wrong kind. Nothing in the interpreter's own
    // library does that -- `CoreClasses.orx:73` hands it plain `::METHOD`s --
    // so the absence is a refusal there rather than a gap here.
    let written_method = |name: &[u8], index: usize| {
        TableValue::Instance(
            "Method",
            Annotated::Unattached(id, name.into()),
            Some((id, index)),
        )
    };
    let generated_method =
        |name: &[u8]| TableValue::Instance("Method", Annotated::Unattached(id, name.into()), None);
    let mut entries = Vec::new();
    let mut seen_class = false;
    for (index, directive) in program.directives.iter().enumerate() {
        let unattached = !seen_class;
        match &directive.kind {
            rexx_parse::DirectiveKind::Class(_) => seen_class = true,
            rexx_parse::DirectiveKind::Method(method)
                if unattached && kind == PackageTable::UnattachedMethods =>
            {
                let upper = method.name.to_ascii_uppercase();
                // `ATTRIBUTE` files an accessor pair, the plain name and
                // that name with `=`. Measured, `::method z attribute` puts
                // `Z` and `Z=` in `.METHODS`; that spelling takes
                // `methodDirective`'s `else` at
                // `parser/DirectiveParser.cpp:889` and files each half
                // through `createAttributeGetterMethod` and
                // `createAttributeSetterMethod` (`:895`, `:896`), whose own
                // `addMethod` calls are at `:2418` and `:2474`.
                if method.attribute {
                    let setter = crate::accessor_setter_name(&upper);
                    let value = generated_method(&setter);
                    entries.push((setter, value));
                }
                let value = written_method(&upper, index);
                entries.push((upper, value));
            }
            rexx_parse::DirectiveKind::Attribute(attribute)
                if unattached && kind == PackageTable::UnattachedMethods =>
            {
                let upper = attribute.name.to_ascii_uppercase();
                let setter = crate::accessor_setter_name(&upper);
                // The same split `Interp::install_attribute` makes over the
                // same field, because the getter and the setter here are the
                // ones it installs, under the same names. Measured,
                // `::attribute zz` puts `ZZ` and `ZZ=` in `.METHODS`,
                // `::attribute zz get` puts `ZZ` alone and `::attribute zz
                // set` puts `ZZ=` alone.
                match attribute.style {
                    rexx_parse::AttributeStyle::Both => {
                        let getter_value = generated_method(&upper);
                        let setter_value = generated_method(&setter);
                        entries.push((upper, getter_value));
                        entries.push((setter, setter_value));
                    }
                    rexx_parse::AttributeStyle::Get => {
                        let value = generated_method(&upper);
                        entries.push((upper, value));
                    }
                    rexx_parse::AttributeStyle::Set => {
                        let value = generated_method(&setter);
                        entries.push((setter, value));
                    }
                }
            }
            rexx_parse::DirectiveKind::Constant(constant)
                if unattached && kind == PackageTable::UnattachedMethods =>
            {
                // One getter, `createConstantGetterMethod`'s own
                // `addMethod(name, method, false)`
                // (`parser/DirectiveParser.cpp:2536`). Measured,
                // `::constant sep '/'` leaves `.methods~items` `1`.
                let upper = constant.name.to_ascii_uppercase();
                let value = generated_method(&upper);
                entries.push((upper, value));
            }
            rexx_parse::DirectiveKind::Routine(routine) if kind == PackageTable::Routines => {
                // Both access scopes, not the public ones alone: `.ROUTINES`
                // is `package->routines` (`parser/LanguageParser.cpp:1893`)
                // and `publicRoutines` is a second table nothing here reads.
                // Measured, a file with one `PUBLIC` and one plain `::ROUTINE`
                // leaves `.routines~items` `2`.
                entries.push((
                    routine.name.to_ascii_uppercase(),
                    TableValue::Instance("Routine", Annotated::Routine(id, index), None),
                ));
            }
            rexx_parse::DirectiveKind::Resource(resource) if kind == PackageTable::Resources => {
                let lines = resource
                    .lines
                    .iter()
                    .map(|span| {
                        program
                            .source
                            .span_bytes(span.clone())
                            .expect("a resource body line is a span of its own program's source")
                            .to_vec()
                    })
                    .collect();
                entries.push((resource.name.to_ascii_uppercase(), TableValue::Lines(lines)));
            }
            _ => {}
        }
    }
    entries
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
    /// **The premise [`Unbuilt`] rests on**: one entry per name is enough,
    /// because a name cannot be in both directories at once.
    ///
    /// A shared name would make the `scope` field pick a side, and the wrong
    /// side turns a refusal into `.nil` for the directory the oracle answers
    /// from. Asserted rather than written in prose because both lists are in
    /// this file and either can gain a row.
    #[test]
    fn the_two_oracle_directories_share_no_name() {
        let environment: HashSet<&str> = ORACLE_ENVIRONMENT.iter().copied().collect();
        let shared: Vec<&str> = ORACLE_LOCAL
            .iter()
            .copied()
            .filter(|name| environment.contains(name))
            .collect();
        assert!(
            shared.is_empty(),
            "these names are in both oracle directory listings, so an `Unbuilt` row for one \
             of them would refuse a lookup in the other: {shared:?}"
        );
        // Neither list is empty, so the filter above ran against something.
        assert!(!ORACLE_ENVIRONMENT.is_empty());
        assert!(!ORACLE_LOCAL.is_empty());
    }
}
