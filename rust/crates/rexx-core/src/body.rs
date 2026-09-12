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

use crate::bytes::Bytes;
use crate::{ObjRef, SlotRef};
use rexx_num::{Form, Number};
use std::collections::{HashMap, TryReserveError};

/// Identifies the behaviour (class + method dictionary) an object responds to.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BehaviourId(pub u16);

impl BehaviourId {
    pub const STRING: BehaviourId = BehaviourId(0);
    pub const ARRAY: BehaviourId = BehaviourId(1);
    pub const OBJECT: BehaviourId = BehaviourId(2);
    pub const STEM: BehaviourId = BehaviourId(3);
}

/// Names one flattened method dictionary in the class graph `rexx-classes`
/// builds -- the oracle's `RexxBehaviour` object, which an object points at.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BehaviourHandle(usize);

impl BehaviourHandle {
    pub const fn new(index: usize) -> BehaviourHandle {
        BehaviourHandle(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

/// A byte string that failed `Number::parse_bytes` (D15). One marker and no
/// cause, which is not a simplification paid for later: nothing observable
/// distinguishes one cause from another. A Rexx program that uses a
/// non-numeric value in arithmetic gets error 41.1, "Nonnumeric value
/// ("val") used in arithmetic operation", which substitutes the *value* and
/// never says why it failed to parse. There is no separate `from_utf8` step
/// able to fail on its own either -- a parse over bytes either yields a
/// number or does not.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NotNumeric;

/// The payload of a heap object.
#[derive(Clone, Debug)]
pub enum Body {
    /// A value whose identity is its bytes (D15). `num` is a tri-state cache
    /// of the one parse those bytes ever get: `None` is "not yet asked", and
    /// must keep meaning exactly that -- treating it as "definitely not a
    /// number" after some other value has been filled in answers wrongly for
    /// a value nobody has asked about yet. The two `Result` arms tell "is a
    /// number" from "is not" apart so a non-numeric string is not re-parsed
    /// on every comparison.
    Text {
        bytes: Bytes,
        num: Option<Result<Box<Number>, NotNumeric>>,
    },
    /// A value whose identity is its number (D15). `created_digits` and
    /// `created_form` are the `NUMERIC DIGITS`/`NUMERIC FORM` in force when
    /// this value was produced, and `text` -- formatted under exactly those,
    /// never the settings in force when it is later read -- is fixed for the
    /// object's whole lifetime once filled in.
    Num {
        value: Number,
        created_digits: u32,
        created_form: Form,
        text: Option<Vec<u8>>,
    },
    /// A stem: its own name, an optional default, and its tails keyed by the
    /// tail's value verbatim (D15a). A tail entry present and mapped to
    /// `None` is a tombstone -- an explicitly dropped tail, which must not
    /// fall back to the default -- and is distinct from the key being
    /// absent, which does.
    Stem {
        name: Box<[u8]>,
        default: Option<ObjRef>,
        /// **The `usize` is the tail's insertion ordinal, and it is
        /// load-bearing.** A stem's tails are held in a balanced tree keyed
        /// on (length, bytes) and walked in POST-ORDER
        /// (`classes/support/CompoundVariableTable.cpp:343`, `:405`), so the
        /// order `allIndexes` and `supplier` answer in depends on the shape
        /// the insertions built and not on the tail names alone -- measured,
        /// the same five tails inserted forwards and backwards answer
        /// `Q,MANGO,LONGKEYNAME,ZEBRA,APPLE` and
        /// `APPLE,Q,ZEBRA,LONGKEYNAME,MANGO`. A `HashMap` cannot say which
        /// came first, so the ordinal rides with the value.
        tails: crate::NameMap<Vec<u8>, (usize, Option<ObjRef>)>,
    },
    /// An array's slots, in index order, `None` for a slot that holds no
    /// object at all.
    Array {
        slots: Vec<Option<ObjRef>>,
        dimensions: Option<Box<[usize]>>,
    },
    /// A user-defined object: the class it belongs to, the behaviour it
    /// dispatches against, the name something gave it, and its instance
    /// variables, one pool per scope (D40).
    Instance {
        class: ObjRef,
        behaviour: BehaviourHandle,
        name: Option<Box<[u8]>>,
        pools: ScopePools,
        own: Option<Box<ObjectMethods>>,
        native: Option<Box<NativeState>>,
    },
    /// An object the interpreter builds for itself rather than one a program
    /// constructs: `.environment`, `.local`, a package's `.methods` table and
    /// an activation's `.context`.
    Native(Box<NativeObject>),
    /// A reference that does not keep its target alive. Traces to nothing --
    /// that is the whole point -- and the collector rewrites the target to
    /// `ObjRef::NIL` once it dies.
    WeakRef(ObjRef),
    /// A class object.
    Class { owned: Vec<ObjRef> },
    /// The object a `>name` term answers: a variable named rather than read.
    VarRef(Box<VarRef>),
}

/// The variable one [`Body::VarRef`] names.
#[derive(Clone, Debug)]
pub struct VarRef {
    pub name: Box<[u8]>,
    pub home: VarRefHome,
}

/// Where the variable a [`VarRef`] names keeps its value.
#[derive(Clone, Debug)]
pub enum VarRefHome {
    /// Storage outside every frame, which the variable itself also reads and
    /// writes through for as long as it exists.
    Cell(SlotRef),
    /// A name in one of `owner`'s scope pools -- what an `EXPOSE`d variable
    /// has instead of storage of its own. The object keeps it alive, so
    /// there is nothing to promote.
    Instance { owner: ObjRef, scope: ObjRef },
}

/// The native state an instance carries in its own body, for the classes whose
/// C++ counterparts keep a `CSELF` block. It holds no [`ObjRef`], which is why
/// [`Body::trace`]'s instance arm ignores the slot.
#[derive(Clone, Debug)]
pub enum NativeState {
    /// A `MutableBuffer`'s contents.
    Buffer(BufferState),
    /// A `Stream`'s open file, positions and state. **Not the object's string
    /// value**: `Stream~string` answers the name a Rexx `expose`d variable
    /// holds, so the readers that render a buffer as its contents must answer
    /// for [`NativeState::Buffer`] alone.
    Stream(StreamState),
}

impl NativeState {
    /// The buffer this state holds, or `None` for state of another kind.
    pub fn buffer(&self) -> Option<&BufferState> {
        match self {
            NativeState::Buffer(state) => Some(state),
            NativeState::Stream(_) => None,
        }
    }

    /// [`NativeState::buffer`] for a caller that changes the contents.
    pub fn buffer_mut(&mut self) -> Option<&mut BufferState> {
        match self {
            NativeState::Buffer(state) => Some(state),
            NativeState::Stream(_) => None,
        }
    }

    /// The stream this state holds, or `None` for state of another kind.
    pub fn stream(&self) -> Option<&StreamState> {
        match self {
            NativeState::Stream(state) => Some(state),
            NativeState::Buffer(_) => None,
        }
    }

    /// [`NativeState::stream`] for a caller that opens, reads, writes or
    /// positions it.
    pub fn stream_mut(&mut self) -> Option<&mut StreamState> {
        match self {
            NativeState::Stream(state) => Some(state),
            NativeState::Buffer(_) => None,
        }
    }
}

/// What a stream is, once `~init` has named it. The four states are the
/// oracle's own (`streamLibrary/StreamNative.hpp`), and `Unknown` is what a
/// stream that has never been opened -- or has been closed -- answers.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum StreamStatus {
    /// Never opened, or closed since.
    Unknown,
    /// Open and usable.
    Ready,
    /// A read found the end of the stream.
    NotReady,
    /// The last operation failed, carrying the errno the oracle reports.
    Error(i32),
}

/// Which of the three standard streams an object stands for, when it is one.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum StandardStream {
    In,
    Out,
    Err,
}

/// A `Stream`'s own state (`streamLibrary/StreamNative.hpp:158`). The name is
/// kept as the program wrote it, because `~string` answers it verbatim; the
/// qualified path is what anything touching the file system uses.
#[derive(Clone, Debug)]
pub struct StreamState {
    /// The name as given, which `~qualify` resolves and `~string` does not.
    pub name: Vec<u8>,
    /// The name resolved against the interpreter's current directory, filled
    /// in when the stream is initialised so a later `DIRECTORY()` cannot move
    /// an already-named stream.
    pub qualified: Vec<u8>,
    /// Set by `!std_set` for `STDIN`, `STDOUT` and `STDERR`, with or without a
    /// trailing colon.
    pub standard: Option<StandardStream>,
    /// Set by `!handle_set` for a `HANDLE:` name. This crate refuses to open
    /// one -- reaching an already-open descriptor needs `unsafe` -- so the
    /// marker exists to answer the queries that do not open.
    pub handle: Option<Vec<u8>>,
    pub status: StreamStatus,
}

impl StreamState {
    /// The state `~init` leaves behind: named, resolved, and unopened.
    pub fn new(name: Vec<u8>, qualified: Vec<u8>) -> StreamState {
        StreamState {
            name,
            qualified,
            standard: None,
            handle: None,
            status: StreamStatus::Unknown,
        }
    }

    /// `state`'s own word, which `~state` answers and `~description` opens
    /// with (`StreamNative.cpp`'s `getState`).
    pub fn state_word(&self) -> &'static [u8] {
        match self.status {
            StreamStatus::Unknown => b"UNKNOWN",
            StreamStatus::Ready => b"READY",
            StreamStatus::NotReady => b"NOTREADY",
            StreamStatus::Error(_) => b"ERROR",
        }
    }
}

/// A `MutableBuffer`'s state: its contents, `bufferLength` and `defaultSize`
/// (`classes/MutableBufferClass.hpp`).
#[derive(Clone, Debug)]
pub struct BufferState {
    pub bytes: Vec<u8>,
    pub capacity: usize,
    pub default_size: usize,
}

impl BufferState {
    /// `MutableBuffer::ensureCapacity`: room for `added` more bytes, taking
    /// the larger of what is needed and twice the capacity.
    pub fn ensure_capacity(&mut self, added: usize) -> Result<(), TryReserveError> {
        let needed = self.bytes.len().saturating_add(added);
        if needed > self.capacity {
            self.capacity = needed.max(self.capacity.saturating_mul(2));
            self.bytes
                .try_reserve_exact(self.capacity - self.bytes.len())?;
        }
        Ok(())
    }

    /// `MutableBuffer::setBufferSize`: zero empties the contents and takes
    /// the capacity back to `default_size`; any other size becomes the
    /// capacity and truncates contents longer than it.
    pub fn set_buffer_size(&mut self, size: usize) -> Result<(), TryReserveError> {
        if size == 0 {
            self.bytes.clear();
            if self.capacity > self.default_size {
                self.capacity = self.default_size;
                self.bytes.shrink_to(self.default_size);
            }
        } else if size != self.capacity {
            self.bytes.truncate(size);
            if size > self.bytes.capacity() {
                self.bytes.try_reserve_exact(size - self.bytes.len())?;
            } else {
                self.bytes.shrink_to(size);
            }
            self.capacity = size;
        }
        Ok(())
    }
}

/// Every variable pool one object holds: one per scope that has bound a name
/// in it, found by walking the list (`ObjectClass.cpp:2489`, whose
/// `objectVariables` is a linked list of `VariableDictionary`s chained by
/// `nextDictionary` and searched the same way).
#[derive(Clone, Debug, Default)]
pub struct ScopePools {
    /// The scope's class identity, and the names it has bound. A name with no
    /// value is **absent** rather than present-and-empty: `DROP` removes the
    /// entry, and a reader cannot tell the two apart -- measured, a class
    /// method exposing `v`, assigning it, and dropping it leaves a later
    /// `expose v` reading the derived name `V`, which is what a never-bound
    /// name reads as.
    pools: Vec<(ObjRef, Vec<(Box<[u8]>, ObjRef)>)>,
}

impl ScopePools {
    pub fn new() -> ScopePools {
        ScopePools { pools: Vec::new() }
    }

    /// The value `name` holds in `scope`'s pool, or `None` when the pool does
    /// not hold it -- which is the uninitialised state and not an error.
    pub fn get(&self, scope: ObjRef, name: &[u8]) -> Option<ObjRef> {
        let pool = self.pool(scope)?;
        pool.iter()
            .find(|(bound, _)| **bound == *name)
            .map(|(_, value)| *value)
    }

    /// Binds `name` in `scope`'s pool, creating the pool and the entry if
    /// this is the first write to either.
    pub fn set(&mut self, scope: ObjRef, name: &[u8], value: ObjRef) {
        let pool = match self.pools.iter().position(|(s, _)| *s == scope) {
            Some(index) => &mut self.pools[index].1,
            None => {
                self.pools.push((scope, Vec::new()));
                &mut self.pools.last_mut().expect("the pool just pushed").1
            }
        };
        match pool.iter_mut().find(|(bound, _)| **bound == *name) {
            Some(entry) => entry.1 = value,
            None => pool.push((name.into(), value)),
        }
    }

    /// Returns `name` to the uninitialised state in `scope`'s pool, which is
    /// what `DROP` on an exposed variable does. A name the pool never held is
    /// a no-op, exactly as `DROP` on a never-assigned local is.
    pub fn clear(&mut self, scope: ObjRef, name: &[u8]) {
        let Some(index) = self.pools.iter().position(|(s, _)| *s == scope) else {
            return;
        };
        self.pools[index].1.retain(|(bound, _)| **bound != *name);
    }

    fn pool(&self, scope: ObjRef) -> Option<&Vec<(Box<[u8]>, ObjRef)>> {
        self.pools
            .iter()
            .find(|(s, _)| *s == scope)
            .map(|(_, pool)| pool)
    }

    /// Appends every object these pools reach: each scope's own identity and
    /// every value bound in it.
    fn trace(&self, out: &mut Vec<ObjRef>) {
        for (scope, pool) in &self.pools {
            out.push(*scope);
            out.extend(pool.iter().map(|(_, value)| *value));
        }
    }
}

/// Names one method body in the dictionaries `rexx-classes` builds.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct MethodId(pub u32);

/// One method attached to a single object: the body it runs and the scope
/// its `EXPOSE` binds in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ObjectMethod {
    pub method: MethodId,
    pub scope: ObjRef,
}

/// The methods attached to one object rather than to its class, searched
/// ahead of the class behaviour (`MethodDictionary::addInstanceMethod`'s
/// `addFront`, `behaviour/MethodDictionary.cpp:399`).
#[derive(Clone, Debug, Default)]
pub struct ObjectMethods {
    set: Vec<(Box<[u8]>, Option<ObjectMethod>)>,
    enhanced: Vec<(Box<[u8]>, ObjectMethod)>,
    /// `RexxBehaviour::setEnhanced`, which `RexxClass::enhanced` sets on the
    /// object it returns (`classes/ClassClass.cpp:1478`) and
    /// `RexxObject::defaultName` reads to answer `enhanced <id>` in place of
    /// the article form (`classes/ObjectClass.cpp:1763`-`:1767`).
    enhanced_instance: bool,
}

impl ObjectMethods {
    pub fn new() -> ObjectMethods {
        ObjectMethods {
            set: Vec::new(),
            enhanced: Vec::new(),
            enhanced_instance: false,
        }
    }

    /// Marks the object one `Class~enhanced` built.
    pub fn mark_enhanced_instance(&mut self) {
        self.enhanced_instance = true;
    }

    /// Whether `Class~enhanced` built this object.
    pub fn is_enhanced_instance(&self) -> bool {
        self.enhanced_instance
    }

    /// Every name either level defines, upper case, `setMethod`'s hidden
    /// entries excluded -- what `Object~instanceMethods(.nil)` walks.
    pub fn defined_names(&self) -> Vec<Box<[u8]>> {
        let mut names: Vec<Box<[u8]>> = self
            .set
            .iter()
            .filter(|(_, entry)| entry.is_some())
            .map(|(name, _)| name.clone())
            .collect();
        for (name, _) in &self.enhanced {
            if Self::find(&self.set, name).is_none() {
                names.push(name.clone());
            }
        }
        names
    }

    /// What this object answers for `name`: `None` when neither level holds
    /// an entry under it, `Some(None)` for a hidden name, and
    /// `Some(Some(method))` for one it defines.
    pub fn get(&self, name: &[u8]) -> Option<Option<ObjectMethod>> {
        match Self::find(&self.set, name) {
            Some(entry) => Some(*entry),
            None => Self::find(&self.enhanced, name).map(|method| Some(*method)),
        }
    }

    /// Adds or replaces `setMethod`'s entry for `name`, which
    /// `addInstanceMethod` does in one step: it removes any method it added
    /// before under that name and puts the new one at the front.
    pub fn set(&mut self, name: &[u8], entry: Option<ObjectMethod>) {
        Self::write(&mut self.set, name, entry);
    }

    /// Adds `Class~enhanced`'s entry for `name`, behind `setMethod`'s.
    pub fn enhance(&mut self, name: &[u8], method: ObjectMethod) {
        Self::write(&mut self.enhanced, name, method);
    }

    /// Removes `setMethod`'s entry for `name`, revealing an enhancing method
    /// of the same name.
    pub fn remove(&mut self, name: &[u8]) {
        self.set.retain(|(key, _)| !key.eq_ignore_ascii_case(name));
    }

    /// Appends every object these entries reach, which is each defined
    /// method's scope -- [`ScopePools::trace`]'s position and its reason.
    fn trace(&self, out: &mut Vec<ObjRef>) {
        out.extend(
            self.set
                .iter()
                .filter_map(|(_, entry)| *entry)
                .map(|ObjectMethod { scope, .. }| scope),
        );
        out.extend(
            self.enhanced
                .iter()
                .map(|(_, ObjectMethod { scope, .. })| *scope),
        );
    }

    fn find<'a, T>(level: &'a [(Box<[u8]>, T)], name: &[u8]) -> Option<&'a T> {
        level
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, entry)| entry)
    }

    fn write<T>(level: &mut Vec<(Box<[u8]>, T)>, name: &[u8], entry: T) {
        match level
            .iter_mut()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
        {
            Some(slot) => slot.1 = entry,
            None => level.push((name.to_ascii_uppercase().into(), entry)),
        }
    }
}

/// The payload of a [`Body::Native`]: what class the object answers to, the
/// text it renders as, and the name-to-value table it holds.
#[derive(Clone, Debug)]
pub struct NativeObject {
    class: ObjRef,
    rendered: Box<[u8]>,
    string_value: Option<Box<[u8]>>,
    entries: HashMap<Box<[u8]>, ObjRef>,
    annotations: Option<ObjRef>,
    scope: Option<ObjRef>,
}

impl NativeObject {
    pub fn new(class: ObjRef, rendered: &[u8]) -> NativeObject {
        NativeObject {
            class,
            rendered: rendered.into(),
            string_value: None,
            entries: HashMap::new(),
            annotations: None,
            scope: None,
        }
    }

    /// The class this object answers to.
    pub fn class(&self) -> ObjRef {
        self.class
    }

    /// The bytes `SAY` prints for this object.
    pub fn rendered(&self) -> &[u8] {
        &self.rendered
    }

    /// `RexxObject::stringValue()`: what this object renders as in a
    /// conversion, which falls back to the default name when the class does
    /// not separate the two. See the field.
    pub fn string_value(&self) -> &[u8] {
        self.string_value.as_deref().unwrap_or(&self.rendered)
    }

    /// Gives this object a string value distinct from its default name.
    pub fn set_string_value(&mut self, bytes: &[u8]) {
        self.string_value = Some(bytes.into());
    }

    /// `~objectName=`: replaces the answer to `~objectName`, and with it
    /// every rendering of this object, since `RexxObject::stringValue` is a
    /// `~objectName` send.
    pub fn set_rendered(&mut self, rendered: &[u8]) {
        self.rendered = rendered.into();
    }

    /// The value stored under `key`, which callers hold already uppercased --
    /// the oracle stores every environment entry under
    /// `getUpperGlobalName(name)` and looks one up by `className->upper()`, so
    /// the case folding belongs to the caller that produced the key and not to
    /// each lookup.
    pub fn entry(&self, key: &[u8]) -> Option<ObjRef> {
        self.entries.get(key).copied()
    }

    pub fn set_entry(&mut self, key: &[u8], value: ObjRef) {
        self.entries.insert(key.into(), value);
    }

    /// Every key this object holds, as owned copies.
    pub fn keys(&self) -> Vec<Box<[u8]>> {
        self.entries.keys().cloned().collect()
    }

    /// The `StringTable` this object's `~annotations` answers, or `None` for
    /// an object whose builder gave it none.
    pub fn annotations(&self) -> Option<ObjRef> {
        self.annotations
    }

    pub fn set_annotations(&mut self, annotations: ObjRef) {
        self.annotations = Some(annotations);
    }

    /// `MethodClass::getScope`: the class this method object is installed on,
    /// or `None` for one nothing has installed yet.
    pub fn scope(&self) -> Option<ObjRef> {
        self.scope
    }

    /// `MethodClass::setScope` (`classes/MethodClass.cpp:219`).
    pub fn set_scope(&mut self, scope: ObjRef) {
        self.scope = Some(scope);
    }
}

/// **A `Body` is every object in the heap, and `Slot` is what the arena holds
/// one of per object**, so a variant that widens either widens the footprint of
/// programs that never construct it. Measured at 80 and 96 across a change that
/// took `Number` from 32 bytes to 40 -- `Body::Num`'s payload had headroom
/// against `Body::Stem`'s, which is what set the width then and sets it now.
const _: () = assert!(size_of::<Body>() <= 80);

impl Body {
    /// A single-dimensional array over `slots`.
    pub fn array(slots: Vec<Option<ObjRef>>) -> Body {
        Body::Array {
            slots,
            dimensions: None,
        }
    }

    /// Appends every object this one can reach.
    pub fn trace(&self, out: &mut Vec<ObjRef>) {
        match self {
            // Neither reaches an `ObjRef`: a `Number` and a byte string are
            // both plain data, never a handle into the heap.
            Body::Text { .. } => {}
            Body::Num { .. } => {}
            Body::Stem { default, tails, .. } => {
                out.extend(default.iter().copied());
                // A tombstone (`None`) reaches nothing, same as a weak
                // reference clearing to `.nil` -- it is present but dead.
                out.extend(tails.values().filter_map(|(_, tail)| *tail));
            }
            // An empty slot reaches nothing, the same as a stem's tombstone
            // above.
            Body::Array { slots, .. } => out.extend(slots.iter().filter_map(|item| *item)),
            // Every scope's pool, walked by the storage's own type -- see
            // [`ScopePools::trace`] for why the walk lives there.
            // The class handle is traced for the reason the `Native` arm
            // below traces its own: it names no arena slot today
            // ([`crate::CLASS_SLOT_BASE`]), and the arm stays correct on the
            // day a class object is allocated like anything else.
            Body::Instance {
                class, pools, own, ..
            } => {
                out.push(*class);
                pools.trace(out);
                // A one-off method's scope, in the position the class handle
                // above is in and traced for the same reason.
                if let Some(own) = own {
                    own.trace(out);
                }
            }
            Body::Native(native) => {
                // The class handle travels with the values. It names no arena
                // slot today ([`crate::CLASS_SLOT_BASE`]), so the collector
                // drops it on the floor; tracing it anyway is what keeps this
                // arm correct on the day a class object is allocated like
                // anything else.
                out.push(native.class);
                out.extend(native.entries.values().copied());
                // A method object's scope, in the position the class handle
                // above is in and traced for the same reason.
                out.extend(native.scope);
                // Reachable from the object alone once a program holds the
                // table and drops every other handle on it.
                out.extend(native.annotations);
            }
            // Deliberately reaches nothing: a weak reference must not keep
            // its target alive.
            Body::WeakRef(_) => {}
            Body::Class { owned } => out.extend(owned.iter().copied()),
            // A cell is rooted by `RootSet::iter`, which is what keeps the
            // referenced value alive; the owning object of an instance
            // variable is not, and the reference is the only handle a
            // program may still have on it.
            Body::VarRef(reference) => match reference.home {
                VarRefHome::Cell(_) => {}
                VarRefHome::Instance { owner, scope } => {
                    out.push(owner);
                    out.push(scope);
                }
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    pub behaviour: BehaviourId,
    pub body: Body,
    /// Set when the object defines an `UNINIT` method. Such an object is
    /// resurrected by the collector and reported through
    /// `CollectStats::pending_uninit` rather than swept, and is cleared
    /// through [`crate::Heap::clear_uninit_all`] once the caller reports the
    /// finalizer has run.
    pub(crate) has_uninit: bool,
    /// Set once the collector has reported this object through
    /// `CollectStats::pending_uninit`, so that a later collection resurrects
    /// it again without reporting it twice -- oracle's `setReadyForUninit`
    /// (`classes/ObjectClass.hpp`), read by `MemoryObject::runUninits`.
    pub(crate) ready_for_uninit: bool,
}

impl Object {
    /// Whether this object defines an `UNINIT` method. See the field.
    pub fn has_uninit(&self) -> bool {
        self.has_uninit
    }
}
