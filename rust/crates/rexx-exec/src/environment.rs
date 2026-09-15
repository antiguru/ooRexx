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

use std::collections::HashMap;

use rexx_core::{BehaviourId, Body, NativeObject, ObjRef};

use crate::dispatch::hash;
use crate::plan::{ClassPackage, Package, ProgramId};
use crate::{Failure, Interp, Loud};

/// Which directory a lookup is reading.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum EnvScope {
    Local,
    Environment,
}

/// The directory-lookup security seam.
mod env_seam {
    use super::{EnvScope, Failure, Interp, ObjRef};
    use crate::security::{key, message};

    /// `.environment` and `.local`, readable only with an [`Admitted`].
    pub(super) struct Directories {
        environment: ObjRef,
        local: ObjRef,
    }

    /// Evidence that a directory lookup passed the security seam.
    pub(super) struct Admitted(());

    /// Why a directory is being read, which decides whether the security
    /// manager sees the read at all. **Deliberately not `Copy`**, for the
    /// reason the clearance beside it is not: one trip through the seam
    /// carries one reason.
    pub(super) enum Access {
        /// `PackageClass::findClass` (`classes/PackageClass.cpp:1134`-`1158`),
        /// whose two manager checks bracket the `.local` read.
        Resolve,
        /// Every other read. `ActivityManager::getLocalEnvironment` and
        /// `ClassDirective`'s own search reach the directories directly, so
        /// a manager never sees an output route or a directive's target.
        Direct,
    }

    /// What the seam decided about one lookup.
    pub(super) enum Admission {
        /// The manager answered the name itself, and the directory is not
        /// read at all.
        Replaced(ObjRef),
        /// Read the directory.
        Permitted(Admitted),
    }

    /// Records the directory handles at bootstrap.
    pub(super) fn hold(environment: ObjRef, local: ObjRef) -> Directories {
        Directories { environment, local }
    }

    /// **The directory chokepoint (D45, site two).** Every read of `.local`
    /// and of `.environment` passes here, and an [`Access::Resolve`] read is
    /// the one the security manager's `LOCAL` and `ENVIRONMENT` checkpoints
    /// see.
    pub(super) fn admit(
        interp: &mut Interp,
        scope: EnvScope,
        name: &[u8],
        access: &Access,
    ) -> Result<Admission, Failure> {
        if matches!(access, Access::Direct) || interp.effective_security_manager().is_none() {
            return Ok(Admission::Permitted(Admitted(())));
        }
        let checkpoint = match scope {
            EnvScope::Local => message::LOCAL,
            EnvScope::Environment => message::ENVIRONMENT,
        };
        let index = interp.text(name);
        interp.roots.push_temp(index);
        let Some(info) = interp.security_check(checkpoint, &[(key::NAME, index)])? else {
            return Ok(Admission::Permitted(Admitted(())));
        };
        // A manager that handles the name and sets no `RESULT` has hidden it:
        // `checkLocalAccess` answers `OREF_NULL` for that, which its caller
        // reads as a miss.
        match interp.security_entry(info, key::RESULT)? {
            Some(value) => Ok(Admission::Replaced(value)),
            None => Ok(Admission::Permitted(Admitted(()))),
        }
    }

    /// Which of the two `handle` is, or `None` for any other object.
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

/// `.environment`'s store capacity, which gives the 69 buckets the oracle's
/// holds its contents at: measured, the contents in `ORACLE_ENVIRONMENT`'s order
/// are non-decreasing in `hash % 69` and in no other bucket count from 17 to 4999,
/// and a `.Directory~new(68)` filled in that order answers the oracle's
/// `allIndexes` before and after 200 more entries.
const ENVIRONMENT_CAPACITY: usize = 68;

/// `.environment~allIndexes` on the oracle, in its order: the contents at
/// [`ENVIRONMENT_CAPACITY`], then `LOCAL`, which is a method-table entry.
static ORACLE_ENVIRONMENT: &[&str] = &[
    "INPUTOUTPUTSTREAM",
    "ALARM",
    "ENDOFLINE",
    "COLLECTION",
    "PACKAGE",
    "FALSE",
    "EVENTSEMAPHORE",
    "COMPARATOR",
    "IDENTITYTABLE",
    "REXXINFO",
    "TICKER",
    "OUTPUTSTREAM",
    "NUMERICCOMPARATOR",
    "MAPCOLLECTION",
    "SET",
    "NIL",
    "STRING",
    "TIMESPAN",
    "MESSAGE",
    "OBJECT",
    "CLASS",
    "REXXQUEUE",
    "ORDERABLE",
    "INVERTINGCOMPARATOR",
    "SINGLETON",
    "METHOD",
    "TRACEOBJECT",
    "CASELESSCOMPARATOR",
    "BAG",
    "ORDEREDCOLLECTION",
    "COLUMNCOMPARATOR",
    "LIST",
    "SUPPLIER",
    "DATETIME",
    "STREAMSUPPLIER",
    "TRUE",
    "REXXCONTEXT",
    "STEM",
    "MUTEXSEMAPHORE",
    "QUEUE",
    "FILE",
    "ROUTINE",
    "MONITOR",
    "MUTABLEBUFFER",
    "MESSAGENOTIFICATION",
    "POINTER",
    "STACKFRAME",
    "ALARMNOTIFICATION",
    "INPUTSTREAM",
    "RELATION",
    "DIRECTORY",
    "TABLE",
    "SETCOLLECTION",
    "VARIABLEREFERENCE",
    "STREAM",
    "ENVIRONMENT",
    "DESCENDINGCOMPARATOR",
    "CASELESSDESCENDINGCOMPARATOR",
    "ARGUTIL",
    "WEAKREFERENCE",
    "CASELESSCOLUMNCOMPARATOR",
    "VALIDATE",
    "ARRAY",
    "COMPARABLE",
    "PROPERTIES",
    "STRINGTABLE",
    "CIRCULARQUEUE",
    "BUFFER",
    "LOCAL",
];

/// `.local~allIndexes` on the oracle, in its order, at the default size.
static ORACLE_LOCAL: &[&str] = &[
    "SYSCARGS",
    "INPUT",
    "TRACEOUTPUT",
    "DEBUGINPUT",
    "STDOUT",
    "OUTPUT",
    "STDERR",
    "STDIN",
    "STDQUE",
    "ERROR",
];

/// Whether [`Interp::mint_local_directory`] builds this name: every
/// [`ORACLE_LOCAL`] name but `STDQUE`, which is a `RexxQueue` over the
/// external-queue API the RXAPI daemon serves.
fn minted_local_name(name: &[u8]) -> bool {
    matches!(
        name,
        b"STDIN"
            | b"STDOUT"
            | b"STDERR"
            | b"INPUT"
            | b"OUTPUT"
            | b"ERROR"
            | b"DEBUGINPUT"
            | b"TRACEOUTPUT"
            | b"SYSCARGS"
    )
}

/// `.environment` and `.local`, and the classes the other reflection names are
/// built from.
pub(crate) struct EnvironmentModel {
    directories: env_seam::Directories,
    /// The item standing in for an entry the oracle's directory holds and this
    /// crate does not build, one per phase owing such entries. A write
    /// replaces one in place, which keeps the oracle's order.
    owed: [(ObjRef, &'static str); 2],
    /// `.methods`, `.routines` and `.resources` all answer one of these.
    string_table: ObjRef,
    /// What `.context` answers to.
    context: ObjRef,
}

/// What one `::ANNOTATE` directive's pairs belong to, once the object that
/// carries them exists.
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
    /// A method compiled from source text, by a count of its own.
    Compiled(usize),
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
enum TableValue {
    /// A `Method` or a `Routine`: the class it answers to, the annotations
    /// it carries, and -- for a `Method` -- which directive of this package
    /// is its body. Nothing here dispatches a `::METHOD` body through
    /// `.METHODS`, and a message neither the class's dictionary nor
    /// `NATIVE_METHODS` holds is 97.1 on either side.
    Instance {
        class: &'static str,
        site: Annotated,
        declared: (ProgramId, usize),
        runnable: bool,
        /// Whether the directive is a `::ROUTINE`, which is the body
        /// `Routine~call` enters.
        routine: bool,
    },
    /// A `::RESOURCE`'s own body lines, which is what `.RESOURCES` holds --
    /// `resources->put(resource, internalname)` over an `ArrayClass`
    /// (`parser/DirectiveParser.cpp:2344`). Measured, a two-line resource's
    /// `~items` is `2` and `SAY` of it prints both lines.
    Lines(Vec<Vec<u8>>),
}

impl Interp {
    /// The directory model, built on first use.
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
        let string_table = self
            .classes()
            .lookup("StringTable")
            .expect("StringTable is a native class");
        let context = self
            .classes()
            .lookup("RexxContext")
            .expect("RexxContext is a native class");
        let object_class = self
            .classes()
            .lookup("Object")
            .expect("Object is a native class");

        // Every object below is rooted the instant it exists and before the
        // next allocation, which can collect.
        let frame = self.roots.push_frame();
        let mut owed = [(ObjRef::NIL, "Phase 5"), (ObjRef::NIL, "Phase 10")];
        for (at, (held, owner)) in owed.iter_mut().enumerate() {
            *held = self.alloc_with(
                BehaviourId::OBJECT,
                Body::Native(Box::new(NativeObject::new(
                    object_class,
                    format!("an entry owed by {owner}").as_bytes(),
                ))),
            );
            self.roots.add_global(&format!(".owed{at}"), *held);
        }
        let environment = self.interpreter_directory(
            ENVIRONMENT_ROOT,
            ENVIRONMENT_CAPACITY,
            b"The Environment Directory",
        );
        let local = self.interpreter_directory(LOCAL_ROOT, 0, b"The Local Directory");

        // `completeSystemClass` (`memory/Setup.cpp:199`) puts every native
        // class into the environment under its uppercased id, and that is the
        // whole of what this crate has to put there beyond the entries
        // `addToEnvironment` adds by hand.
        let mut known: HashMap<Box<[u8]>, ObjRef> = self
            .classes()
            .registered()
            .map(|(name, id)| (name.as_bytes().into(), id))
            .collect();
        // `addToEnvironment` (`Setup.cpp:1730`-`1733`) registers these by
        // hand.
        let true_value = self.text(b"1");
        self.roots.push_temp(true_value);
        let false_value = self.text(b"0");
        self.roots.push_temp(false_value);
        // `Setup.cpp:1736`-`:1737`: `REXXINFO` is a pre-built *instance* of a
        // class no environment name reaches, which is why the entry renders
        // as `a RexxInfo` and answers `~class~id` `RexxInfo`. Measured
        // on the oracle at rc 0, `.RexxInfo~isA(.Class)` is `0` and
        // `.RexxInfo~class~superClass` is `The Object class`.
        let rexx_info_class = self
            .classes()
            .system_lookup("RexxInfo")
            .expect("RexxInfo is a native class in the kernel directory");
        let rexx_info = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Native(Box::new(NativeObject::new(rexx_info_class, b"a RexxInfo"))),
        );
        self.roots.push_temp(rexx_info);
        // `.ENDOFLINE` is the platform's line terminator as a String --
        // measured, `c2x(.endOfLine)` is `0A` here and its length is 1. The
        // ooTest framework's own prologue reads it (`OOREXXUNIT.CLS:77`).
        let end_of_line = self.text(crate::parse_template::LINE_END);
        self.roots.push_temp(end_of_line);
        for (name, value) in [
            (b"ENVIRONMENT".as_slice(), environment),
            (b"NIL", ObjRef::NIL),
            (b"TRUE", true_value),
            (b"FALSE", false_value),
            (b"REXXINFO", rexx_info),
            (b"ENDOFLINE", end_of_line),
        ] {
            known.insert(name.into(), value);
        }

        // In the oracle's order, so every entry sits where the oracle's does
        // and a later write of a name replaces it in place. A name this crate
        // has not built yet is owed until the library bootstrap puts it.
        for name in ORACLE_ENVIRONMENT {
            if *name == "LOCAL" {
                continue;
            }
            let value = known.remove(name.as_bytes()).unwrap_or(owed[0].0);
            self.put_interpreter_entry(environment, name.as_bytes(), value);
        }
        let mut rest: Vec<(Box<[u8]>, ObjRef)> = known.into_iter().collect();
        rest.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, value) in rest {
            self.put_interpreter_entry(environment, &name, value);
        }
        // `Setup.cpp:1781`: `LOCAL` is a method answering the running
        // activity's local directory, and this crate runs one activity.
        hash::directory_put_method_value(self, environment, b"LOCAL", local)
            .expect("a string-keyed store accepts a string index");
        // `LocalServer~initInstance` (`CoreClasses.orx:987`) builds these
        // before any program runs; `Interp::mint_local_directory` does the
        // same once the classes it needs exist.
        for name in ORACLE_LOCAL {
            let owner = if minted_local_name(name.as_bytes()) {
                owed[0].0
            } else {
                owed[1].0
            };
            self.put_interpreter_entry(local, name.as_bytes(), owner);
        }
        self.roots.pop_frame(frame);

        EnvironmentModel {
            directories: env_seam::hold(environment, local),
            owed,
            string_table,
            context,
        }
    }

    /// A `Directory` for `.environment` or `.local`, held alive under `root`
    /// and rendered as `rendered`.
    fn interpreter_directory(&mut self, root: &str, capacity: usize, rendered: &[u8]) -> ObjRef {
        let directory =
            hash::new_directory(self, capacity).expect("Directory is not an abstract class");
        self.roots.add_global(root, directory);
        if let Some(object) = self.heap.get_mut(directory)
            && let Body::Instance { name, .. } = &mut object.body
        {
            *name = Some(rendered.into());
        }
        directory
    }

    /// One entry of a directory [`Interp::build_environment`] is filling.
    fn put_interpreter_entry(&mut self, directory: ObjRef, name: &[u8], value: ObjRef) {
        hash::directory_put(self, directory, name, value)
            .expect("a string-keyed store accepts a string index");
    }

    /// The phase owing `item`, when it stands in for an entry the oracle's
    /// directory holds and this crate does not build.
    pub(crate) fn owed_entry_owner(&self, item: ObjRef) -> Option<&'static str> {
        let model = self.environment.as_ref()?;
        model
            .owed
            .iter()
            .find(|(held, _)| *held == item)
            .map(|(_, owner)| *owner)
    }

    /// `.NAME`'s value: `PackageClass::findClass`'s order, then
    /// `RexxActivation::rexxVariable`'s reflection names, then the name's own
    /// text.
    pub(crate) fn dot_variable(&mut self, dotted: &[u8]) -> Result<ObjRef, Failure> {
        let bare = dotted.strip_prefix(b".").unwrap_or(dotted);

        if let Some(found) = self.installed_class(bare) {
            return Ok(found);
        }

        if let Some(found) = self.imported_class(bare) {
            return Ok(found);
        }

        if let Some(found) = self.rexx_package_class(bare) {
            return Ok(found);
        }

        if let Some(found) = self.package_local_entry(bare) {
            return Ok(found);
        }

        match self.directory_lookup(
            &[EnvScope::Local, EnvScope::Environment],
            bare,
            &env_seam::Access::Resolve,
        )? {
            hash::DirectoryEntry::Found(found) => return Ok(found),
            hash::DirectoryEntry::Owed(owner) => {
                return Err(Loud::environment_symbol(dotted, owner).into());
            }
            hash::DirectoryEntry::Absent => {}
        }

        if let Some(found) = self.rexx_variable(bare) {
            return Ok(found);
        }

        // `variableName->concatToCstring(".")`
        // (`expression/ExpressionDotVariable.cpp:167`, and `:218` for the
        // route `VALUE` takes).
        Ok(self.text(dotted))
    }

    /// The class an **unqualified** `::CLASS` directive's `SUBCLASS`,
    /// `INHERIT` or `METACLASS` keyword names, when the file's own directives
    /// do not declare it. A `ns:Name` target never comes here --
    /// `Interp::resolve_class_target` answers it from the namespace table
    /// instead, which is `ClassResolver::lookup`'s own split.
    pub(crate) fn directive_class(
        &mut self,
        installing: ProgramId,
        upper: &[u8],
    ) -> Option<ObjRef> {
        if let Some(found) = self.installed_class(upper) {
            return Some(found);
        }
        if let Some(found) = self
            .merged_public_classes
            .get(&installing)
            .and_then(|table| table.get(upper))
        {
            return Some(*found);
        }
        // `.environment` alone, not `.NAME`'s pair: `ClassDirective`'s own
        // search is the package's classes and then the environment
        // directory, and `.local` is not in it.
        if let Ok(hash::DirectoryEntry::Found(found)) =
            self.directory_lookup(&[EnvScope::Environment], upper, &env_seam::Access::Direct)
            && self.heap.is_class(found)
        {
            return Some(found);
        }
        // A name that answers something which is not a class, and a name
        // whose directory entry this crate has not built, both fall through
        // to the native table -- which is where a `::CLASS` target was
        // resolved before `.environment` was ever consulted, so a miss is the
        // same 98.909 it was.
        self.classes().lookup_upper(upper)
    }

    /// What the first of `scopes` whose directory holds `bare` answers for it.
    fn directory_lookup(
        &mut self,
        scopes: &[EnvScope],
        bare: &[u8],
        access: &env_seam::Access,
    ) -> Result<hash::DirectoryEntry, Failure> {
        for &scope in scopes {
            match env_seam::admit(self, scope, bare, access)? {
                env_seam::Admission::Replaced(value) => {
                    return Ok(hash::DirectoryEntry::Found(value));
                }
                env_seam::Admission::Permitted(admitted) => {
                    let handle = {
                        let model = self.environment_model();
                        env_seam::directory(&model.directories, admitted, scope)
                    };
                    match hash::directory_get(self, handle, bare)? {
                        hash::DirectoryEntry::Absent => {}
                        entry => return Ok(entry),
                    }
                }
            }
        }
        Ok(hash::DirectoryEntry::Absent)
    }

    /// What `.local` holds for one of the route names, or `None` when it holds
    /// nothing.
    ///
    /// **Not [`Interp::dot_variable`]**, which is what a `.NAME` in a program
    /// resolves through: on a miss that answers the name *as text*, and `SAY`
    /// sent to the string `".OUTPUT"` would be a wrong answer. The routes need
    /// the plain question -- is there an entry -- which is what
    /// `directory_lookup` answers, and it is reused rather than reopened so the
    /// seam still has one read chokepoint. An owed entry is no entry here: it
    /// is owed only before `Interp::mint_local_directory` runs.
    ///
    /// `.local` only: measured, `RexxActivation::resolveStream` and
    /// `Activity::sayOutput` both read `getLocalEnvironment` and never
    /// `.environment`.
    pub(crate) fn local_route(&mut self, name: &[u8]) -> Result<Option<ObjRef>, Failure> {
        match self.directory_lookup(&[EnvScope::Local], name, &env_seam::Access::Direct)? {
            hash::DirectoryEntry::Found(found) => Ok(Some(found)),
            hash::DirectoryEntry::Owed(_) | hash::DirectoryEntry::Absent => Ok(None),
        }
    }

    /// Records that a route's far end may have moved, so the next `SAY`
    /// decides afresh.
    pub(crate) fn bump_route_generation(&mut self) {
        self.route_generation = self.route_generation.wrapping_add(1);
    }

    /// Where `SAY` sends, or `None` to write straight to the buffer.
    ///
    /// **The decision is cached, not either half of it**: deciding afresh is a
    /// probe of `.local` and a walk of the monitors' destination queues, 764
    /// instructions a line between them, and caching one half would leave most
    /// of that standing.
    pub(crate) fn output_route(&mut self) -> Result<Option<ObjRef>, Failure> {
        if let Some((generation, cached)) = self.output_route
            && generation == self.route_generation
        {
            // A writer this crate forgot to count would otherwise be a wrong
            // answer no corpus program could see, because none of them
            // redirect `.OUTPUT`.
            #[cfg(debug_assertions)]
            {
                let fresh = self.output_route_uncached()?;
                assert_eq!(
                    cached, fresh,
                    "the cached SAY route is stale, so some writer does not bump \
                     Interp::route_generation"
                );
            }
            return Ok(cached);
        }
        let fresh = self.output_route_uncached()?;
        self.output_route = Some((self.route_generation, fresh));
        Ok(fresh)
    }

    /// [`Interp::output_route`] with no cache in front of it.
    fn output_route_uncached(&mut self) -> Result<Option<ObjRef>, Failure> {
        match self.local_route(b"OUTPUT")? {
            Some(route)
                if route != ObjRef::NIL && !self.route_ends_at(route, self.bootstrap_stdout) =>
            {
                Ok(Some(route))
            }
            _ => Ok(None),
        }
    }

    /// Whether `route` still ends at `terminal`, following each monitor's
    /// destination queue head (`CoreClasses.orx`'s `Monitor`, whose `UNKNOWN`
    /// forwards to `destination~peek`).
    ///
    /// A route that ends at the stream the bundle minted writes the same bytes
    /// whether it is delivered or written straight to the buffer, and the
    /// direct write is the one that runs no Rexx. That matters twice over.
    /// Delivering re-enters the interpreter mid-clause, which perturbs the
    /// argument stack, `PROCEDURE` permission and a `SELECT`'s own search --
    /// measured, all three. And it is not cheap: measured, delivering every
    /// `SAY` costs 16,360 instructions a line, which is `sayloop` at 9.63x.
    pub(crate) fn route_ends_at(&mut self, route: ObjRef, terminal: Option<ObjRef>) -> bool {
        let Some(terminal) = terminal else {
            return false;
        };
        let Some(monitor_class) = self.rexx_package_class(b"MONITOR") else {
            return false;
        };
        let Some(array_class) = self.classes().lookup("Array") else {
            return false;
        };
        let mut at = route;
        // The chain the mint builds is `TRACEOUTPUT` over `ERROR` over
        // `STDERR`, and a program may leave it shorter or longer; the bound
        // is what keeps a cycle from spinning here.
        for _ in 0..8 {
            if at == terminal {
                return true;
            }
            let Some(queue) = self.pool_entry(at, monitor_class, b"DESTINATION") else {
                return false;
            };
            let store = match self.array_slots(queue) {
                Some(_) => queue,
                None => match self.pool_entry(queue, array_class, b"ITEMS") {
                    Some(store) => store,
                    None => return false,
                },
            };
            let Some(Some(head)) = self.array_slots(store).and_then(<[_]>::first).copied() else {
                return false;
            };
            at = head;
        }
        false
    }

    /// What `name` holds in `owner`'s pool for `scope`, or `None`.
    fn pool_entry(&self, owner: ObjRef, scope: ObjRef, name: &[u8]) -> Option<ObjRef> {
        match self.heap.get(owner).map(|object| &object.body) {
            Some(Body::Instance { pools, .. }) => pools.get(scope, name),
            _ => None,
        }
    }

    /// One finished trace line, delivered to `.TRACEOUTPUT` as a `TraceObject`
    /// -- `Activity::traceOutput` (`concurrency/Activity.cpp:3171`).
    ///
    /// `start` is where the line begins in the buffer: a `.STDERR` `CHAROUT`
    /// leaves an unterminated fragment there, so the line cannot be found by
    /// scanning back for a newline. A route that cannot take the line leaves
    /// it on stdout, which is `corpus/oracle-crashes.txt` entry 11's licensed
    /// answer for a route the oracle dies on.
    pub(crate) fn route_trace_line(&mut self, start: usize) {
        if self.routing_trace {
            return;
        }
        let Ok(Some(route)) = self.local_route(b"TRACEOUTPUT") else {
            return;
        };
        if route == ObjRef::NIL || self.route_ends_at(route, self.bootstrap_stderr) {
            return;
        }
        let Some(class) = self.rexx_package_class(b"TRACEOBJECT") else {
            return;
        };
        let mut line = self.trace.split_off(start);
        while line.last() == Some(&b'\n') {
            line.pop();
        }
        self.routing_trace = true;
        // **The nested-send protocol, not a bare send**, and around the whole
        // delivery because `NEW`, each `PUT` and the `LINEOUT` all run Rexx
        // clauses. A clause boundary clears the shared argument stack, so
        // delivering on the caller's own stack wipes the arguments an
        // enclosing call has already pushed: measured, `zz = length('abcd')`
        // under `trace i` refuses with `call_op_off_its_node`, where a call
        // with no arguments is untouched.
        let (values, mark) = self.take_value_buffer();
        let delivered = self.deliver_trace_line(class, route, &line);
        self.give_value_buffer(values, mark);
        self.routing_trace = false;
        if delivered.is_err() {
            self.out.extend_from_slice(&line);
            self.out.push(b'\n');
        }
    }

    /// Each line of an error report, delivered the way a trace line is:
    /// measured, the oracle sends one `LINEOUT` per line -- three for an
    /// uncaught `1/0` -- and not one for the report.
    pub(crate) fn write_trace_report(&mut self, report: &[u8]) {
        let body = report.strip_suffix(b"\n").unwrap_or(report);
        for line in body.split(|&byte| byte == b'\n') {
            let start = self.trace.len();
            self.trace.extend_from_slice(line);
            self.trace.push(b'\n');
            self.route_trace_line(start);
        }
    }

    /// The send itself: a `TraceObject` carrying `line`, then `LINEOUT` to
    /// `route`. Every entry is a `String` on the oracle, `NUMBER` included.
    fn deliver_trace_line(
        &mut self,
        class: ObjRef,
        route: ObjRef,
        line: &[u8],
    ) -> Result<(), Failure> {
        let caller = self.caller();
        let Some(object) = self.send_message(class, b"NEW", None, &[], caller)? else {
            return Err(Failure::Raised(Box::new(crate::error::Raised::syntax(
                97,
                1,
                vec![b"TraceObject".to_vec()],
            ))));
        };
        self.roots.push_temp(object);
        // A `StringTable` keeps its entries in a bucket table, not in the
        // `Body::Native` map `.local` uses, so the put goes through the
        // message. `t[i] = v` sends `t~"[]="(v, i)`, so the value leads.
        for (name, value) in [
            (b"THREAD".as_slice(), Some(b"1".as_slice())),
            (b"INVOCATION".as_slice(), Some(b"0".as_slice())),
            (b"INTERPRETER".as_slice(), Some(b"1".as_slice())),
            (b"TRACELINE".as_slice(), Some(line)),
            (b"STACKFRAME".as_slice(), None),
        ] {
            let held = match value {
                Some(bytes) => {
                    let held = self.text(bytes);
                    self.roots.push_temp(held);
                    held
                }
                None => ObjRef::NIL,
            };
            let index = self.text(name);
            self.roots.push_temp(index);
            let caller = self.caller();
            self.send_message(object, b"PUT", None, &[Some(held), Some(index)], caller)?;
        }
        let caller = self.caller();
        self.send_message(
            route,
            crate::dispatch::LINEOUT,
            None,
            &[Some(object)],
            caller,
        )?;
        Ok(())
    }

    /// A class the running package's own directives installed, under its
    /// uppercased name -- `PackageClass::findInstalledClass`, the first step of
    /// the order.
    fn installed_class(&self, upper: &[u8]) -> Option<ObjRef> {
        let program = self.running_program()?;
        self.package_classes.get(&program)?.get(upper).copied()
    }

    /// A public class the running package's own `::REQUIRES` directives
    /// imported -- `PackageClass::findPublicClass`, the step between the
    /// package's own installed classes and the directories.
    fn imported_class(&self, upper: &[u8]) -> Option<ObjRef> {
        let program = self.running_program()?;
        self.merged_public_classes
            .get(&program)?
            .get(upper)
            .copied()
    }

    /// The running package's own local environment directory entry for `bare`
    /// -- `packageLocal->get(internalName)` in `PackageClass::findClass`
    /// (`classes/PackageClass.cpp:1122`), the step between the REXX package's
    /// public classes and `.local`.
    fn package_local_entry(&self, bare: &[u8]) -> Option<ObjRef> {
        let program = self.running_program()?;
        let directory = *self.package_locals.get(&Package::Program(program))?;
        // The map alone: a package's local directory is one of this crate's
        // own `native_instance`s and never a collection with a store, and
        // this caller is on a `&self` path.
        self.native_map_entry(directory, bare)
    }

    /// A public class of the interpreter's own package --
    /// `TheRexxPackage->findPublicClass`, step 4 of the documented
    /// environment-symbol search order (`rexxpg` `classes.xml:838`) and the
    /// step `PackageClass::findClass` takes between a package's imports and
    /// its own local (`classes/PackageClass.cpp:1105`).
    pub(crate) fn rexx_package_class(&mut self, upper: &[u8]) -> Option<ObjRef> {
        if let Some(&hit) = self.rexx_class_cache.get(upper) {
            debug_assert_eq!(
                Some(hit),
                self.rexx_package_class_uncached(upper),
                "the .NAME cache answered {upper:?} with a class the search no longer finds, so \
                 some table this cache is derived from was mutated without \
                 Interp::invalidate_rexx_class_cache"
            );
            return Some(hit);
        }
        let found = self.rexx_package_class_uncached(upper)?;
        // Only a hit is cached. A miss must stay a miss: a later `::CLASS`
        // install or a `~addPackage` can turn one into a hit, and a cached
        // miss would outlive that where a cached hit is invalidated with the
        // table it came from.
        self.rexx_class_cache.insert(upper.into(), found);
        Some(found)
    }

    /// [`Interp::rexx_package_class`]'s own search, with no cache in front of
    /// it. Kept separate so the cache's debug assertion has something to
    /// compare against.
    fn rexx_package_class_uncached(&mut self, upper: &[u8]) -> Option<ObjRef> {
        for program in &self.library_programs {
            if let Some(found) = self
                .package_public_classes
                .get(program)
                .and_then(|table| table.get(upper))
            {
                return Some(*found);
            }
        }
        self.classes().lookup_upper(upper)
    }

    /// Drops every cached `.NAME` answer.
    pub(crate) fn invalidate_rexx_class_cache(&mut self) {
        self.rexx_class_cache.clear();
    }

    /// `Package~local`: the package's own environment directory, allocated on
    /// the first ask.
    pub(crate) fn package_local(&mut self, package: Package) -> ObjRef {
        if let Some(found) = self.package_locals.get(&package).copied() {
            return found;
        }
        let class = self
            .classes()
            .lookup("Directory")
            .expect("Directory is a native class");
        let directory = self.native_instance(class);
        self.roots
            .add_global(&package_local_root_key(package), directory);
        self.package_locals.insert(package, directory);
        directory
    }

    /// The `.local` entries `LocalServer~initInstance` builds before any
    /// program runs (`CoreClasses.orx:987`): the command-line words, then the
    /// streams and monitors, each stored as it is built. `STDQUE` stays owed.
    ///
    /// **Here rather than when the directory is built**, because the embedded
    /// bootstrap evaluates `.environment` while it installs
    /// (`CoreClasses.orx:55`), before `StreamClasses.orx` has declared `Stream`.
    /// Each write replaces an owed entry in place, so the order is the
    /// oracle's whatever order these are built in.
    ///
    /// **Through the package publics, not the class registry.** A `::CLASS`
    /// directive calls `define_unregistered_class`, which by its own contract
    /// records the class "without registering the name", so
    /// `classes().lookup("Stream")` never finds one the embedded
    /// `StreamClasses.orx` declared.
    ///
    /// **No panic on a miss, at any step.** A step that cannot build its
    /// entry leaves that entry and every later one owed.
    pub(crate) fn mint_local_directory(&mut self) {
        let frame = self.roots.push_frame();
        self.mint_local_entries();
        self.roots.pop_frame(frame);
    }

    /// [`Interp::mint_local_directory`]'s steps, inside its root frame.
    fn mint_local_entries(&mut self) {
        // **`Body::array`, not `.Array~of`'s shape** -- measured, a
        // `.SYSCARGS` built from no words answers `~dimension` `0` as
        // `.array~new` does, where `.array~of()` answers `1`.
        let mut slots: Vec<Option<ObjRef>> = Vec::with_capacity(self.command_words.len());
        for at in 0..self.command_words.len() {
            let word = std::mem::take(&mut self.command_words[at]);
            let text = self.text(&word);
            self.roots.push_temp(text);
            slots.push(Some(text));
            self.command_words[at] = word;
        }
        let arguments = self.alloc_with(BehaviourId::ARRAY, Body::array(slots));
        self.roots.push_temp(arguments);
        if self
            .set_directory_entry(EnvScope::Local, b"SYSCARGS", arguments)
            .is_err()
        {
            return;
        }
        let Some(stream_class) = self.rexx_package_class(b"STREAM") else {
            return;
        };
        for (name, which) in [
            (b"STDIN".as_slice(), rexx_core::StandardStream::In),
            (b"STDOUT".as_slice(), rexx_core::StandardStream::Out),
            (b"STDERR".as_slice(), rexx_core::StandardStream::Err),
        ] {
            let Ok(built) = crate::dispatch::stream::standard_stream(self, stream_class, which)
            else {
                return;
            };
            if name == b"STDERR".as_slice() {
                self.bootstrap_stderr = Some(built);
            }
            if name == b"STDOUT".as_slice() {
                self.bootstrap_stdout = Some(built);
            }
            if self
                .set_directory_entry(EnvScope::Local, name, built)
                .is_err()
            {
                return;
            }
        }
        let Some(monitor_class) = self.rexx_package_class(b"MONITOR") else {
            return;
        };
        for (name, over, rendered) in [
            (
                b"INPUT".as_slice(),
                b"STDIN".as_slice(),
                "The INPUT monitor",
            ),
            (
                b"DEBUGINPUT".as_slice(),
                b"INPUT".as_slice(),
                "The DEBUG INPUT monitor",
            ),
            (
                b"OUTPUT".as_slice(),
                b"STDOUT".as_slice(),
                "The OUTPUT monitor",
            ),
            (
                b"ERROR".as_slice(),
                b"STDERR".as_slice(),
                "The ERROR monitor",
            ),
            (
                b"TRACEOUTPUT".as_slice(),
                b"ERROR".as_slice(),
                "The TRACE OUTPUT monitor",
            ),
        ] {
            let Ok(Some(target)) = self.local_route(over) else {
                return;
            };
            let caller = self.caller();
            let Ok(Some(built)) =
                self.send_message(monitor_class, b"NEW", None, &[Some(target)], caller)
            else {
                return;
            };
            self.roots.push_temp(built);
            if let Some(object) = self.heap.get_mut(built)
                && let Body::Instance { name: held, .. } = &mut object.body
            {
                *held = Some(rendered.as_bytes().into());
            }
            if self
                .set_directory_entry(EnvScope::Local, name, built)
                .is_err()
            {
                return;
            }
        }
    }

    /// Writes `name` into `scope`'s directory, through the same seam a read
    /// passes: `VALUE(name, new, '')`, whose empty selector names
    /// `.environment` itself (`expression/BuiltinFunctions.cpp:1848`), and
    /// [`Interp::mint_local_directory`].
    pub(crate) fn set_directory_entry(
        &mut self,
        scope: EnvScope,
        name: &[u8],
        value: ObjRef,
    ) -> Result<(), Failure> {
        // A write the manager never sees: `VALUE(name, new, 'ENVIRONMENT')`
        // puts into the directory directly (`expression/BuiltinFunctions.cpp:
        // 1848`).
        let admitted = match env_seam::admit(self, scope, name, &env_seam::Access::Direct)? {
            env_seam::Admission::Permitted(admitted) => admitted,
            env_seam::Admission::Replaced(_) => unreachable!("a direct access is never replaced"),
        };
        let handle = {
            let model = self.environment_model();
            env_seam::directory(&model.directories, admitted, scope)
        };
        hash::directory_put(self, handle, name, value)
    }

    /// `RexxActivation::rexxVariable` (`execution/RexxActivation.cpp:2842`):
    /// the names the interpreter answers out of the running activation rather
    /// than out of a directory.
    fn rexx_variable(&mut self, bare: &[u8]) -> Option<ObjRef> {
        match bare {
            b"METHODS" => {
                let program = self.running_program()?;
                self.package_string_table(program, PackageTable::UnattachedMethods)
            }
            b"ROUTINES" => {
                let program = self.running_program()?;
                self.package_string_table(program, PackageTable::Routines)
            }
            b"RESOURCES" => {
                let program = self.running_program()?;
                self.package_string_table(program, PackageTable::Resources)
            }
            // `new_integer(current->getLineNumber())`. `clause_state.line()`
            // is the same quantity `SIGL` reads and carries the same
            // `INTERPRET` rule the oracle's own `isInterpret` delegation
            // gives this name: the enclosing clause's line, not the
            // fragment's.
            // Answering `None` before any command has run is what makes
            // `.RS` render as its own name: `dot_variable`'s tail falls back
            // to the name's text, which is `isReturnStatusSet`'s else branch
            // (`RexxActivation::rexxVariable`). Measured -- `say .rs` ahead of
            // every command prints `.RS`, and 1 after one that failed.
            b"RS" => {
                let rs = self.activation().rs?;
                Some(self.text(rs.to_string().as_bytes()))
            }
            b"LINE" => Some(self.counted(self.clause_state.line())),
            b"CONTEXT" => Some(self.context_object()),
            _ => None,
        }
    }

    /// `.METHODS`/`.ROUTINES`/`.RESOURCES`: a `StringTable` when the running
    /// package declares at least one directive of that kind, and nothing at
    /// all when it declares none.
    pub(crate) fn package_string_table(
        &mut self,
        program: ProgramId,
        kind: PackageTable,
    ) -> Option<ObjRef> {
        self.build_package_string_table(program, kind, false)
    }

    /// [`Interp::package_string_table`] for a caller that is about to write an
    /// entry into it, which builds the table even when the program's own
    /// directives declare none of that kind.
    pub(crate) fn package_string_table_for_write(
        &mut self,
        program: ProgramId,
        kind: PackageTable,
    ) -> Option<ObjRef> {
        self.build_package_string_table(program, kind, true)
    }

    fn build_package_string_table(
        &mut self,
        program: ProgramId,
        kind: PackageTable,
        force: bool,
    ) -> Option<ObjRef> {
        if let Some(found) = self.package_tables.get(&(program, kind)).copied() {
            return Some(found);
        }
        let source = std::rc::Rc::clone(self.programs.get(program.0)?);
        let entries = if self.untranslated.contains(&program) {
            Vec::new()
        } else {
            package_table_entries(program, &source, kind)
        };
        if entries.is_empty() && !force {
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
                TableValue::Instance {
                    class: id,
                    site,
                    declared: (program, directive),
                    runnable,
                    routine,
                } => {
                    let class = self
                        .classes()
                        .lookup(id)
                        .expect("every TableValue::Instance names a native class");
                    let object = self.native_instance(class);
                    self.attach_annotations(object, site);
                    self.executable_sources.insert(
                        object,
                        crate::ExecutableRecord {
                            source: crate::ExecutableSource::Directive { program, directive },
                            installed: None,
                            routine: routine.then_some((program, directive)),
                        },
                    );
                    if runnable {
                        self.table_method_bodies
                            .insert(object, crate::InstalledMethodBody { program, directive });
                    }
                    if routine {
                        self.routine_objects
                            .insert(crate::InstalledRoutine { program, directive }, object);
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

    /// A fresh `StringTable` holding `entries`, sorted by name.
    pub(crate) fn string_table_of(&mut self, mut entries: Vec<(Box<[u8]>, ObjRef)>) -> ObjRef {
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let frame = self.roots.push_frame();
        for (_, value) in &entries {
            self.roots.push_temp(*value);
        }
        let class = self.environment_model().string_table;
        let table = self.native_instance(class);
        let object = self.heap.get_mut(table).expect("just allocated and rooted");
        let Body::Native(native) = &mut object.body else {
            unreachable!("allocated as Body::Native by native_instance")
        };
        for (name, value) in entries {
            native.set_entry(&name, value);
        }
        self.roots.pop_frame(frame);
        self.roots.push_temp(table);
        table
    }

    /// A fresh `Array` holding `items` in the order given.
    pub(crate) fn object_array(&mut self, items: Vec<ObjRef>) -> ObjRef {
        let frame = self.roots.push_frame();
        for item in &items {
            self.roots.push_temp(*item);
        }
        let slots: Vec<Option<ObjRef>> = items.into_iter().map(Some).collect();
        let array = self.alloc_with(BehaviourId::ARRAY, Body::array(slots));
        self.roots.pop_frame(frame);
        self.roots.push_temp(array);
        array
    }

    /// The one `Routine` object standing for `installed`.
    pub(crate) fn routine_object(&mut self, installed: crate::InstalledRoutine) -> Option<ObjRef> {
        if let Some(found) = self.routine_objects.get(&installed).copied() {
            return Some(found);
        }
        self.package_string_table(installed.program, PackageTable::Routines);
        self.routine_objects.get(&installed).copied()
    }

    /// One `::RESOURCE`'s body as the `Array` of strings `.RESOURCES` holds.
    fn line_array(&mut self, lines: &[Vec<u8>]) -> ObjRef {
        let mut slots = Vec::with_capacity(lines.len());
        for line in lines {
            let text = self.text(line);
            self.roots.push_temp(text);
            slots.push(Some(text));
        }
        let array = self.alloc_with(BehaviourId::ARRAY, Body::array(slots));
        self.roots.push_temp(array);
        array
    }

    /// `.CONTEXT`: `RexxActivation::getContextObject`, which builds the
    /// object on the first ask and keeps it in the activation's own field.
    fn context_object(&mut self) -> ObjRef {
        self.context_object_at(0).unwrap_or_else(|| {
            let class = self.environment_model().context;
            self.native_instance(class)
        })
    }

    /// [`Interp::context_object`] for the activation at `depth`, which
    /// `RexxContext~stackFrames` needs: `RexxActivation::createStackFrame`
    /// passes `getContextObject()`, so **building a frame creates the
    /// context object of an activation that never asked for one**. Measured,
    /// oracle rc 0: every frame of a four-frame stack answers
    /// `~context~class~id` `RexxContext` in a program where only the
    /// innermost ever named `.context`.
    pub(crate) fn context_object_at(&mut self, depth: usize) -> Option<ObjRef> {
        if let Some(found) = self.frame_at(depth)?.context_object {
            return Some(found);
        }
        let class = self.environment_model().context;
        let object = self.native_instance(class);
        self.frame_at_mut(depth)?.context_object = Some(object);
        Some(object)
    }

    /// An instance of `class` with no entries, rendered the way
    /// `RexxObject::defaultName` renders one.
    pub(crate) fn native_instance(&mut self, class: ObjRef) -> ObjRef {
        let rendered = default_object_name(self.classes().id_string(class));
        let object = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Native(Box::new(NativeObject::new(class, rendered.as_bytes()))),
        );
        self.roots.push_temp(object);
        object
    }

    /// The entry `index` names on a `Body::Native` `Directory` or
    /// `StringTable`, or `.nil`.
    pub(crate) fn hash_entry_read(&mut self, receiver: ObjRef, index: &[u8]) -> ObjRef {
        self.native_entry(receiver, index).unwrap_or(ObjRef::NIL)
    }

    /// Stores `item` under `index` on a `Directory` or a `StringTable`.
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

    /// Every key a string-keyed table holds, owned and in the table's own
    /// order -- `NativeObject`'s map for a table this crate built on one, and
    /// the hash store for any other `Directory` or `StringTable`.
    pub(crate) fn native_keys(&mut self, object: ObjRef) -> Vec<Box<[u8]>> {
        match self.heap.get(object).map(|held| &held.body) {
            Some(Body::Native(native)) => native.keys(),
            Some(Body::Instance { .. }) => self
                .store_indexes(object)
                .into_iter()
                .map(|index| self.to_text(index).into_owned().into_boxed_slice())
                .collect(),
            _ => Vec::new(),
        }
    }

    /// [`crate::dispatch::hash::store_indexes`], reachable from this module.
    fn store_indexes(&mut self, object: ObjRef) -> Vec<ObjRef> {
        crate::dispatch::hash::store_indexes(self, object)
    }

    /// [`crate::dispatch::hash::store_item`], reachable from this module.
    fn store_item(&mut self, object: ObjRef, index: ObjRef) -> Option<ObjRef> {
        crate::dispatch::hash::store_item(self, object, index)
    }

    /// One entry of a `Body::Native`'s own map, for a caller that holds
    /// `&self` and is asking about a table this crate built.
    fn native_map_entry(&self, object: ObjRef, index: &[u8]) -> Option<ObjRef> {
        match &self.heap.get(object)?.body {
            Body::Native(native) => native.entry(index),
            _ => None,
        }
    }

    /// One entry of a string-keyed table, by the key the caller holds.
    pub(crate) fn native_entry(&mut self, object: ObjRef, index: &[u8]) -> Option<ObjRef> {
        match &self.heap.get(object)?.body {
            Body::Native(native) => native.entry(index),
            Body::Instance { .. } => {
                for held in self.store_indexes(object) {
                    if self.to_text(held).as_ref() == index {
                        return self.store_item(object, held);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Stores one entry on a `Body::Native`, and does nothing for a receiver
    /// that is not one.
    pub(crate) fn set_native_entry(&mut self, object: ObjRef, index: &[u8], value: ObjRef) {
        if let Some(held) = self.heap.get_mut(object)
            && let Body::Native(native) = &mut held.body
        {
            native.set_entry(index, value);
        }
    }

    /// Which of the two directories this model built `directory` is, or `None`
    /// for any other object.
    fn directory_scope(&mut self, directory: ObjRef) -> Option<EnvScope> {
        let model = self.environment_model();
        env_seam::which(&model.directories, directory)
    }

    /// The phase owing an entry `object` holds and this crate does not build,
    /// or `None` for a collection whose entries it fills. Only `.environment`
    /// and `.local` hold such entries.
    pub(crate) fn unbuilt_collection_owner(&mut self, object: ObjRef) -> Option<&'static str> {
        self.directory_scope(object)?;
        hash::owed_table_owner(self, object)
    }

    /// The program whose directives and installed classes a `.NAME` resolves
    /// against -- the running activation's own.
    pub(crate) fn running_program(&self) -> Option<ProgramId> {
        self.running_activation().map(|frame| frame.program_id)
    }

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
        self.classes()
            .define_instance_method(class, &String::from_utf8_lossy(name), method);
        self.hold_method_object(class, name, object);
        Some(())
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
    pub(crate) fn package_find_class(
        &mut self,
        package: Option<ProgramId>,
        upper: &[u8],
    ) -> ObjRef {
        if let Some(found) = self.installed_class_of(package, upper) {
            return found;
        }
        if let Some(found) = self.package_public_class_of(package, upper) {
            return found;
        }
        let local = self.package_local(match package {
            Some(program) => Package::Program(program),
            None => Package::Rexx,
        });
        if let Some(found) = self.native_map_entry(local, upper) {
            return found;
        }
        if let Ok(hash::DirectoryEntry::Found(found)) = self.directory_lookup(
            &[EnvScope::Local, EnvScope::Environment],
            upper,
            &env_seam::Access::Resolve,
        ) {
            return found;
        }
        ObjRef::NIL
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

    /// `PackageClass::findRoutine` (`classes/PackageClass.cpp:897`):
    /// `findLocalRoutine` and then `findPublicRoutine`, as the `Routine`
    /// object or `.nil`.
    pub(crate) fn package_find_routine(
        &mut self,
        package: Option<ProgramId>,
        upper: &[u8],
    ) -> ObjRef {
        let Some(program) = package else {
            return ObjRef::NIL;
        };
        let found = [
            &self.routines,
            &self.package_public_routines,
            &self.merged_public_routines,
        ]
        .into_iter()
        .find_map(|table| {
            table
                .get(&program)
                .and_then(|held| held.get(upper))
                .copied()
        });
        match found.and_then(|installed| self.routine_object(installed)) {
            Some(object) => object,
            None => ObjRef::NIL,
        }
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

/// The [`rexx_core::RootSet::add_global`] key one package object is held
/// under.
fn package_root_key(package: Package) -> String {
    match package {
        Package::Rexx => "the REXX package".to_string(),
        Package::Program(ProgramId(id)) => format!("the package of program {id}"),
    }
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

/// The [`rexx_core::RootSet::add_global`] key one package's own local
/// environment directory is held under.
fn package_local_root_key(package: Package) -> String {
    format!("the package local of {}", package_root_key(package))
}

/// The [`rexx_core::RootSet::add_global`] key one package table is held under.
fn package_table_root_key(ProgramId(program): ProgramId, kind: PackageTable) -> String {
    let which = match kind {
        PackageTable::UnattachedMethods => "methods",
        PackageTable::Routines => "routines",
        PackageTable::Resources => "resources",
    };
    format!("the {which} of program {program}")
}

/// What `kind`'s table holds for `program`, keyed the way the oracle keys it.
fn package_table_entries(
    id: ProgramId,
    program: &rexx_parse::Program,
    kind: PackageTable,
) -> Vec<(Vec<u8>, TableValue)> {
    // `runnable` is set for a written `::METHOD` alone. An `::ATTRIBUTE`
    // and a `::CONSTANT` file generated accessors, whose bodies are
    // `Interp::generated_methods` rather than `Interp::method_bodies`, and
    // handing one of those to `Class~defineClassMethod` would install a row
    // naming a body of the wrong kind. Nothing in the interpreter's own
    // library does that -- `CoreClasses.orx:73` hands it plain `::METHOD`s --
    // so the absence is a refusal there rather than a gap here.
    let written_method = |name: &[u8], index: usize| TableValue::Instance {
        class: "Method",
        site: Annotated::Unattached(id, name.into()),
        declared: (id, index),
        runnable: true,
        routine: false,
    };
    let generated_method = |name: &[u8], index: usize| TableValue::Instance {
        class: "Method",
        site: Annotated::Unattached(id, name.into()),
        declared: (id, index),
        runnable: false,
        routine: false,
    };
    let mut entries = Vec::new();
    let mut seen_class = false;
    for (index, directive) in program.directives.iter().enumerate() {
        // A synthetic directive is in no package table either -- `lib.rs`'s
        // `Interp::install_directives` carries the rule and what asserts it.
        if directive.clause_span.is_empty() {
            continue;
        }
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
                    let value = generated_method(&setter, index);
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
                        let getter_value = generated_method(&upper, index);
                        let setter_value = generated_method(&setter, index);
                        entries.push((upper, getter_value));
                        entries.push((setter, setter_value));
                    }
                    rexx_parse::AttributeStyle::Get => {
                        let value = generated_method(&upper, index);
                        entries.push((upper, value));
                    }
                    rexx_parse::AttributeStyle::Set => {
                        let value = generated_method(&setter, index);
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
                let value = generated_method(&upper, index);
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
                    TableValue::Instance {
                        class: "Routine",
                        site: Annotated::Routine(id, index),
                        declared: (id, index),
                        runnable: false,
                        routine: true,
                    },
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
        let Body::Instance { class, .. } = &object.body else {
            panic!("the environment is a Directory instance with a store")
        };
        assert_eq!(*class, directory);
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
    /// The names and order the bootstrap leaves in both directories are the
    /// oracle's, and the only entry still owed is `.local`'s `STDQUE`.
    #[test]
    fn the_bootstrap_leaves_the_oracles_names_in_the_oracles_order() {
        let mut interp = Interp::new();
        interp
            .bootstrap_library()
            .expect("the library bootstrap runs");
        for (dotted, oracle) in [
            (b".ENVIRONMENT".as_slice(), ORACLE_ENVIRONMENT),
            (b".LOCAL".as_slice(), ORACLE_LOCAL),
        ] {
            let directory = interp.dot_variable(dotted).expect("resolves");
            let caller = interp.caller();
            let indexes = interp
                .send_message(directory, b"ALLINDEXES", None, &[], caller)
                .expect("answers")
                .expect("answers a value");
            let names: Vec<String> = interp
                .array_slots_of(indexes)
                .expect("an array")
                .into_iter()
                .flatten()
                .map(|index| String::from_utf8_lossy(&interp.to_text(index)).into_owned())
                .collect();
            assert_eq!(names, oracle, "{}", String::from_utf8_lossy(dotted));
            let owed: Vec<&str> = oracle
                .iter()
                .copied()
                .filter(|name| {
                    matches!(
                        hash::directory_get(&mut interp, directory, name.as_bytes()),
                        Ok(hash::DirectoryEntry::Owed(_))
                    )
                })
                .collect();
            let expected: &[&str] = if oracle == ORACLE_LOCAL {
                &["STDQUE"]
            } else {
                &[]
            };
            assert_eq!(owed, expected, "{}", String::from_utf8_lossy(dotted));
        }
    }

    /// **Every method `Directory`'s own table holds answers on `.environment`
    /// and on `.local`**, enumerated from the registry rather than listed:
    /// each resolves to the method a `.Directory~new` resolves to, and a send
    /// of each is not a not-implemented refusal.
    #[test]
    fn every_directory_method_answers_on_both_directories() {
        let mut interp = Interp::new();
        interp
            .bootstrap_library()
            .expect("the library bootstrap runs");
        let class = interp
            .classes()
            .lookup("Directory")
            .expect("Directory is native");
        let names = interp.classes().own_instance_method_names(class);
        assert!(
            names.contains("HASENTRY") && names.contains("[]="),
            "the registry's Directory table is not the one this test is about: {names:?}"
        );
        let fresh = interp.classes().instance_behaviour_handle(class);
        // `EMPTY` last, since it takes `ENVIRONMENT` and `LOCAL` out.
        let mut ordered: Vec<&String> = names.iter().filter(|name| *name != "EMPTY").collect();
        ordered.extend(names.iter().filter(|name| *name == "EMPTY"));
        let mut calls = String::new();
        for name in ordered {
            let arguments = match name.as_str() {
                "ALLINDEXES" | "ALLITEMS" | "EMPTY" | "INIT" | "ISEMPTY" | "ITEMS"
                | "MAKEARRAY" | "SUPPLIER" => "",
                "AT" | "[]" | "ENTRY" | "HASENTRY" | "HASINDEX" | "HASITEM" | "INDEX"
                | "REMOVE" | "REMOVEENTRY" | "REMOVEITEM" | "UNSETMETHOD" => "'ZZ'",
                "PUT" | "[]=" | "SETENTRY" => "'v', 'ZZ'",
                "SETMETHOD" => "'ZZ', 'return 1'",
                "UNKNOWN" => "'ZZ', .array~new",
                other => panic!("Directory's table holds {other}, which this test has no call for"),
            };
            for (directory, variable) in [("environment", "e"), ("local", "l")] {
                let receiver = interp
                    .dot_variable(format!(".{}", directory.to_uppercase()).as_bytes())
                    .expect("resolves");
                let Some(Body::Instance { behaviour, .. }) =
                    interp.heap.get(receiver).map(|object| &object.body)
                else {
                    panic!(".{directory} is not a Directory instance");
                };
                let behaviour = *behaviour;
                assert_eq!(
                    interp.classes().lookup_at(behaviour, name),
                    interp.classes().lookup_at(fresh, name),
                    ".{directory}~{name} resolves to a different method than a new Directory's"
                );
                calls.push_str(&format!("{variable}~\"{name}\"({arguments})\n"));
            }
        }
        let source = format!("e = .environment\nl = .local\nl['STDQUE'] = 'q'\n{calls}");
        let outcome = crate::run_program("/t.rex", source.into_bytes(), crate::Invocation::none());
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        assert!(
            !stderr.contains("is not implemented"),
            "a Directory method refused on an interpreter directory: {stderr}"
        );
        assert_eq!(outcome.exit_code, 0, "{stderr}");
    }
}
