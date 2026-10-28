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
use crate::{ArraySlots, ObjRef, SlotRef};
use rexx_num::{Form, Number};
use rustc_hash::FxHashMap;
use std::collections::TryReserveError;

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

/// The bytes one `Body::Array` slot holds.
pub const SLOT_BYTES: usize = size_of::<Option<ObjRef>>();

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
        /// The tails an `EXPOSE` or `PROCEDURE EXPOSE` made another stem's,
        /// each naming that stem: `CompoundTableElement::expose`'s
        /// `realElement`. Such a tail is also in `tails`, as a tombstone, so
        /// that it keeps its place and holds no value of its own.
        exposed: Option<Box<crate::NameMap<Vec<u8>, ObjRef>>>,
    },
    /// An array's slots, in index order, `None` for a slot that holds no
    /// object at all.
    Array {
        slots: ArraySlots,
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
    /// What keeps the storage of a variable a `>name` reference was taken
    /// to: the index of that storage in [`crate::RootSet`]'s cells, which the
    /// collector frees with this object, and the name of the variable that
    /// owned it.
    Cell(u32, Box<[u8]>),
}

// A heap value names no borrowed frame (spec 2026-09-29 section 5, D3).
const _: () = {
    const fn require_static<T: 'static>() {}
    require_static::<Body>();
};

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
/// C++ counterparts keep a `CSELF` block. [`NativeState::Package`] and
/// [`NativeState::Executable`] hold [`ObjRef`]s.
#[derive(Clone, Debug)]
pub enum NativeState {
    /// A `MutableBuffer`'s contents.
    Buffer(BufferState),
    /// A `Stream`'s open file, positions and state. **Not the object's string
    /// value**: `Stream~string` answers the name a Rexx `expose`d variable
    /// holds, so the readers that render a buffer as its contents must answer
    /// for [`NativeState::Buffer`] alone.
    Stream(StreamState),
    /// A `.Pointer`'s address, the `pointerData` of `PointerClass`
    /// (`interpreter/classes/PointerClass.hpp:77`). Its string value is the
    /// address rendered, so it answers for the reader
    /// [`NativeState::Buffer`] answers for.
    Pointer(*mut std::ffi::c_void),
    /// A `Buffer`'s bytes (`interpreter/classes/BufferClass.hpp`), which an
    /// extension writes through the address `BufferData` handed it for as long
    /// as the object lives, so the storage is never resized.
    Data(AlignedBytes),
    /// A `String` subclass instance's bytes, the `RexxString` body that
    /// `RexxString::newRexx` (`interpreter/classes/StringClass.cpp:2352`)
    /// gives the subclass's behaviour.
    Text(Box<[u8]>),
    /// A `Package` subclass instance's package state.
    Package(PackageState),
    /// The package a `Method` or `Routine` subclass instance's code belongs
    /// to, which the instance keeps alive.
    Executable(ObjRef),
}

/// The state a `Package` object holds. Its fields are the interpreter's own
/// types, so the heap reaches them through this trait alone.
pub trait PackageFields: std::any::Any + std::fmt::Debug {
    /// Appends every object these fields hold.
    fn trace(&self, out: &mut Vec<ObjRef>);
    /// A copy of these fields, holding the same objects.
    fn clone_fields(&self) -> Box<dyn PackageFields>;
}

/// A `Package` object's [`PackageFields`]. Cloning it clones the fields.
#[derive(Debug)]
pub struct PackageState(Box<dyn PackageFields>);

impl Clone for PackageState {
    fn clone(&self) -> PackageState {
        PackageState(self.0.clone_fields())
    }
}

impl PackageState {
    pub fn new(fields: Box<dyn PackageFields>) -> PackageState {
        PackageState(fields)
    }

    /// The fields as `T`, or `None` for fields of another type.
    #[inline]
    pub fn downcast_ref<T: PackageFields>(&self) -> Option<&T> {
        let fields: &dyn std::any::Any = &*self.0;
        fields.downcast_ref()
    }

    /// [`PackageState::downcast_ref`] for a caller that changes them.
    #[inline]
    pub fn downcast_mut<T: PackageFields>(&mut self) -> Option<&mut T> {
        let fields: &mut dyn std::any::Any = &mut *self.0;
        fields.downcast_mut()
    }

    /// Appends every object the fields hold.
    pub fn trace(&self, out: &mut Vec<ObjRef>) {
        self.0.trace(out);
    }
}

/// One 16-byte unit of [`AlignedBytes`], aligned as `malloc` aligns on
/// x86-64, which is what an extension may store any C object in.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C, align(16))]
pub struct Word(u128);

/// Bytes whose first one is 16-byte aligned, with a length of their own.
#[derive(Clone, Debug)]
pub struct AlignedBytes {
    words: Vec<Word>,
    len: usize,
}

impl AlignedBytes {
    /// `len` zero bytes, in at least one word, so that even an empty run has
    /// an address no other shares.
    pub fn zeroed(len: usize) -> AlignedBytes {
        AlignedBytes {
            words: vec![Word::default(); len.div_ceil(16).max(1)],
            len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// A copy of the bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self
            .words
            .iter()
            .flat_map(|word| word.0.to_ne_bytes())
            .collect();
        bytes.truncate(self.len);
        bytes
    }

    /// Writes `bytes` from `at`, as far as the length reaches.
    pub fn write(&mut self, at: usize, bytes: &[u8]) {
        for (offset, byte) in bytes.iter().enumerate() {
            let index = at + offset;
            if index >= self.len {
                break;
            }
            let word = &mut self.words[index / 16];
            let mut raw = word.0.to_ne_bytes();
            raw[index % 16] = *byte;
            word.0 = u128::from_ne_bytes(raw);
        }
    }

    /// The address of the first byte.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.words.as_mut_ptr().cast()
    }
}

impl NativeState {
    /// A `Buffer`'s `length` zero bytes, at an address no other buffer
    /// shares, which an empty `Vec` would not give: the oracle's empty buffer
    /// is an object of its own.
    pub fn zeroed(length: usize) -> NativeState {
        NativeState::Data(AlignedBytes::zeroed(length))
    }

    /// The buffer this state holds, or `None` for state of another kind.
    pub fn buffer(&self) -> Option<&BufferState> {
        match self {
            NativeState::Buffer(state) => Some(state),
            NativeState::Stream(_)
            | NativeState::Pointer(_)
            | NativeState::Data(_)
            | NativeState::Text(_)
            | NativeState::Package(_)
            | NativeState::Executable(_) => None,
        }
    }

    /// [`NativeState::buffer`] for a caller that changes the contents.
    pub fn buffer_mut(&mut self) -> Option<&mut BufferState> {
        match self {
            NativeState::Buffer(state) => Some(state),
            NativeState::Stream(_)
            | NativeState::Pointer(_)
            | NativeState::Data(_)
            | NativeState::Text(_)
            | NativeState::Package(_)
            | NativeState::Executable(_) => None,
        }
    }

    /// The stream this state holds, or `None` for state of another kind.
    pub fn stream(&self) -> Option<&StreamState> {
        match self {
            NativeState::Stream(state) => Some(state),
            NativeState::Buffer(_)
            | NativeState::Pointer(_)
            | NativeState::Data(_)
            | NativeState::Text(_)
            | NativeState::Package(_)
            | NativeState::Executable(_) => None,
        }
    }

    /// [`NativeState::stream`] for a caller that opens, reads, writes or
    /// positions it.
    pub fn stream_mut(&mut self) -> Option<&mut StreamState> {
        match self {
            NativeState::Stream(state) => Some(state),
            NativeState::Buffer(_)
            | NativeState::Pointer(_)
            | NativeState::Data(_)
            | NativeState::Text(_)
            | NativeState::Package(_)
            | NativeState::Executable(_) => None,
        }
    }

    /// The address this state holds, or `None` for state of another kind.
    ///
    /// A null address is a `.Pointer` that answers `1` to `~isNull`, not the
    /// absence of one, so the two `None`s never mean the same thing.
    pub fn pointer(&self) -> Option<*mut std::ffi::c_void> {
        match self {
            NativeState::Pointer(address) => Some(*address),
            NativeState::Buffer(_)
            | NativeState::Stream(_)
            | NativeState::Data(_)
            | NativeState::Text(_)
            | NativeState::Package(_)
            | NativeState::Executable(_) => None,
        }
    }

    /// A `Buffer`'s bytes, or `None` for state of another kind.
    pub fn data(&self) -> Option<&AlignedBytes> {
        match self {
            NativeState::Data(bytes) => Some(bytes),
            NativeState::Buffer(_)
            | NativeState::Stream(_)
            | NativeState::Pointer(_)
            | NativeState::Text(_)
            | NativeState::Package(_)
            | NativeState::Executable(_) => None,
        }
    }

    /// The address of a `Buffer`'s first byte, which stays valid while the
    /// object does, or `None` for state of another kind.
    pub fn data_address(&mut self) -> Option<*mut std::ffi::c_void> {
        match self {
            NativeState::Data(bytes) => Some(bytes.as_mut_ptr().cast()),
            NativeState::Buffer(_)
            | NativeState::Stream(_)
            | NativeState::Pointer(_)
            | NativeState::Text(_)
            | NativeState::Package(_)
            | NativeState::Executable(_) => None,
        }
    }

    /// A `String` subclass instance's bytes, or `None` for state of another
    /// kind.
    pub fn text(&self) -> Option<&[u8]> {
        match self {
            NativeState::Text(bytes) => Some(bytes),
            NativeState::Buffer(_)
            | NativeState::Stream(_)
            | NativeState::Pointer(_)
            | NativeState::Data(_)
            | NativeState::Package(_)
            | NativeState::Executable(_) => None,
        }
    }

    /// A `Package` subclass instance's package state, or `None` for state of
    /// another kind.
    pub fn package(&self) -> Option<&PackageState> {
        match self {
            NativeState::Package(state) => Some(state),
            _ => None,
        }
    }

    /// [`NativeState::package`] for a caller that changes it.
    pub fn package_mut(&mut self) -> Option<&mut PackageState> {
        match self {
            NativeState::Package(state) => Some(state),
            _ => None,
        }
    }

    /// Appends every object this state holds.
    pub fn trace(&self, out: &mut Vec<ObjRef>) {
        match self {
            NativeState::Package(state) => state.trace(out),
            NativeState::Executable(package) => out.push(*package),
            NativeState::Buffer(_)
            | NativeState::Stream(_)
            | NativeState::Pointer(_)
            | NativeState::Data(_)
            | NativeState::Text(_) => {}
        }
    }

    /// Whether the object holding this state renders its string value from the
    /// state itself, as `MutableBuffer::stringValue`
    /// (`interpreter/classes/MutableBufferClass.cpp:740`) and
    /// `PointerClass::stringValue` (`interpreter/classes/PointerClass.cpp:152`)
    /// do, rather than from the name every other instance answers with.
    ///
    /// Matched over the variants rather than asked of one of them, so that a
    /// state added here has to decide.
    pub fn renders_its_own_string_value(&self) -> bool {
        match self {
            NativeState::Buffer(_) | NativeState::Pointer(_) | NativeState::Text(_) => true,
            NativeState::Stream(_)
            | NativeState::Data(_)
            | NativeState::Package(_)
            | NativeState::Executable(_) => false,
        }
    }
}

/// `Numerics::pointerToString` (`interpreter/runtime/Numerics.cpp:882`): the
/// string value of a `.Pointer`, which is `0x0` for a null address and the
/// platform's `%p` rendering otherwise.
pub fn pointer_to_string(address: *mut std::ffi::c_void) -> Vec<u8> {
    if address.is_null() {
        return b"0x0".to_vec();
    }
    format!("{address:p}").into_bytes()
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

/// Which way a stream was opened, from the options the open parsed.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenMode {
    pub read_only: bool,
    pub write_only: bool,
    pub read_write: bool,
    pub append: bool,
    pub nobuffer: bool,
    pub record_based: bool,
    /// `RECLENGTH`, or the file's own size for a `BINARY` open that gave none.
    pub record_length: u64,
}

/// How many written bytes a buffered stream holds before it writes them: the
/// oracle's `DEFAULT_BUFFER_SIZE` (`common/platform/unix/SysFile.hpp`).
pub const STREAM_BUFFER: usize = 4096;

/// The descriptor an open stream holds, with the logical positions one
/// descriptor serves. The character positions are 1-based "next byte", as the
/// oracle's are. A SEEK that would move one below 1 fails, so a write
/// position is 0 only on a stream opened READ, which has none.
///
/// A stream is buffered unless it was opened `NOBUFFER` or is a terminal
/// (rexxref streamclasses.xml, `NOBUFFER`): its writes reach the file when
/// [`STREAM_BUFFER`] bytes have gathered, when the next write does not follow
/// on, and before anything reads, measures or positions the descriptor
/// through [`OpenFile::flushed`], so another stream or process sees them only
/// from then on. A drop writes what is left.
#[derive(Debug)]
pub struct OpenFile {
    file: std::fs::File,
    /// Written bytes the file does not hold yet, which land at byte offset
    /// `held_at` (where the file stands, on a transient stream).
    held: Vec<u8>,
    held_at: u64,
    buffered: bool,
    pub read_position: i64,
    pub write_position: i64,
    /// A byte taken from a transient stream to learn whether data waits, which
    /// the next read returns first. Only a device that cannot say how much
    /// waits without being read leaves one.
    pub pending: Option<u8>,
    pub transient: bool,
    /// A character position and the line it lies in, from the last line
    /// operation. Any write landing before the position drops it, so it is
    /// always what a count from the start would answer.
    pub line_mark: Option<(i64, i64)>,
    /// The stream's line count, dropped by any write.
    pub line_total: Option<u64>,
    /// Debug builds only: digests of the bytes [`OpenFile::line_mark`] and
    /// [`OpenFile::line_total`] were counted over, so a check of either runs
    /// only while those bytes are unchanged. Another stream object or process
    /// writing the file may leave the line answers stale (rexxref
    /// iostrms.xml:495-500); the checks then stand aside.
    pub line_mark_digest: Option<u64>,
    pub line_total_digest: Option<u64>,
}

impl OpenFile {
    /// `file` opened, read from position 1 and written at `write_position`.
    pub fn new(
        file: std::fs::File,
        write_position: i64,
        transient: bool,
        buffered: bool,
    ) -> OpenFile {
        OpenFile {
            file,
            held: Vec::new(),
            held_at: 0,
            buffered,
            read_position: 1,
            write_position,
            pending: None,
            transient,
            line_mark: None,
            line_total: None,
            line_mark_digest: None,
            line_total_digest: None,
        }
    }

    /// The descriptor as it stands, without the writes still held: for its
    /// type, its number and its size where held writes cannot matter.
    pub fn file(&self) -> &std::fs::File {
        &self.file
    }

    /// The descriptor once the held writes are in it, for anything that
    /// reads, measures or positions it. A failed write is dropped here, as
    /// the oracle's `SysFile::flush` from a read drops it.
    pub fn flushed(&mut self) -> &mut std::fs::File {
        let _ = self.flush();
        &mut self.file
    }

    /// The size the file has once the held writes are in it, without
    /// writing them.
    pub fn size(&self) -> u64 {
        let size = self.file.metadata().map_or(0, |meta| meta.len());
        match self.held.is_empty() || self.transient {
            true => size,
            false => size.max(self.held_at + self.held.len() as u64),
        }
    }

    /// The descriptor's byte offset as the stream sees it: past the held
    /// writes, which have not moved the descriptor yet.
    pub fn offset(&mut self) -> u64 {
        use std::io::Seek;
        match self.held.is_empty() || self.transient {
            true => self.file.stream_position().unwrap_or(0),
            false => self.held_at + self.held.len() as u64,
        }
    }

    /// Writes the held bytes.
    pub fn flush(&mut self) -> std::io::Result<()> {
        if self.held.is_empty() {
            return Ok(());
        }
        let held = std::mem::take(&mut self.held);
        self.write_through(self.held_at, &held)
    }

    /// Writes `bytes` at byte offset `offset` (where the file stands, on a
    /// transient stream), as `SysFile::write` does: held while they follow
    /// on from the held bytes and fit, written in buffer-sized pieces as the
    /// buffer fills, and written at once, after the held bytes, when they
    /// are more than a buffer holds.
    pub fn write(&mut self, offset: u64, bytes: &[u8]) -> std::io::Result<()> {
        if !self.buffered {
            return self.write_through(offset, bytes);
        }
        let follows = self.transient || self.held_at + self.held.len() as u64 == offset;
        if !follows || bytes.len() > STREAM_BUFFER {
            self.flush()?;
        }
        if bytes.len() > STREAM_BUFFER {
            return self.write_through(offset, bytes);
        }
        if self.held.is_empty() {
            self.held_at = offset;
        }
        let mut rest = bytes;
        while !rest.is_empty() {
            if self.held.len() == STREAM_BUFFER {
                let at = self.held_at + STREAM_BUFFER as u64;
                self.flush()?;
                self.held_at = at;
            }
            let take = rest.len().min(STREAM_BUFFER - self.held.len());
            self.held.extend_from_slice(&rest[..take]);
            rest = &rest[take..];
        }
        Ok(())
    }

    fn write_through(&mut self, offset: u64, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::{Seek, SeekFrom, Write};
        if !self.transient {
            self.file.seek(SeekFrom::Start(offset))?;
        }
        self.file.write_all(bytes)
    }
}

/// Writes what is held, except while a panic unwinds: a crash loses held
/// writes, as the oracle's does.
impl Drop for OpenFile {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let _ = self.flush();
        }
    }
}

/// A `Stream`'s own state (`streamLibrary/StreamNative.hpp:158`). The name is
/// kept as the program wrote it, because `~string` answers it verbatim; the
/// qualified path is what anything touching the file system uses.
#[derive(Debug)]
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
    /// Set by `!handle_set` for `HANDLE:0`, `HANDLE:1` and `HANDLE:2`, which
    /// read and write through the standard stream of that number
    /// (Deviation 35). `standard` is set with it.
    pub handle: Option<StandardStream>,
    pub status: StreamStatus,
    /// `None` until something opens the stream, and again after a close.
    pub open: Option<OpenFile>,
    pub mode: OpenMode,
    /// What a standard output or error stream opened without `NOBUFFER` has
    /// written and not yet handed to its descriptor, as [`OpenFile`] holds a
    /// file's writes.
    pub held: Vec<u8>,
}

/// A copy is an **unopened** stream carrying the same name: a descriptor
/// cannot be cloned without either sharing a file position or a `dup(2)` that
/// can fail where `Clone` cannot. The oracle has no answer to match --
/// `~copy` of a stream aborts it (`corpus/oracle-crashes.txt` entry 13) --
/// so this is a licensed divergence rather than a shape being reproduced.
impl Clone for StreamState {
    fn clone(&self) -> StreamState {
        StreamState {
            name: self.name.clone(),
            qualified: self.qualified.clone(),
            standard: self.standard,
            handle: self.handle,
            status: StreamStatus::Unknown,
            open: None,
            mode: OpenMode::default(),
            held: Vec::new(),
        }
    }
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
            open: None,
            mode: OpenMode::default(),
            held: Vec::new(),
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
    pub bytes: BufferBytes,
    pub capacity: usize,
    pub default_size: usize,
}

/// A `MutableBuffer`'s bytes, whose address native calls in flight may hold
/// (spec 2026-09-29 2.5). While any does ([`BufferBytes::lend`]), growth
/// moves the bytes to new storage and keeps the old until the last of those
/// calls ends ([`BufferBytes::release`]); a shrink keeps the storage.
///
/// Only the growth methods here may reallocate: the slice it dereferences to
/// cannot, and [`BufferBytes::edit`] refuses to.
#[derive(Debug)]
pub struct BufferBytes {
    bytes: Vec<u8>,
    lent: usize,
    retired: Vec<Vec<u8>>,
}

impl BufferBytes {
    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    /// Room for `additional` bytes past the length.
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        if self.bytes.capacity() - self.bytes.len() >= additional {
            return Ok(());
        }
        if self.lent == 0 {
            return self.bytes.try_reserve_exact(additional);
        }
        let mut moved = Vec::new();
        moved.try_reserve_exact(self.bytes.len().saturating_add(additional))?;
        moved.extend_from_slice(&self.bytes);
        self.retired.push(std::mem::replace(&mut self.bytes, moved));
        Ok(())
    }

    /// [`BufferBytes::try_reserve_exact`], aborting where `Vec`'s own growth
    /// would.
    fn reserve_exact(&mut self, additional: usize) {
        if self.try_reserve_exact(additional).is_err() {
            std::alloc::handle_alloc_error(std::alloc::Layout::new::<u8>());
        }
    }

    pub fn extend_from_slice(&mut self, bytes: &[u8]) {
        self.reserve_exact(bytes.len());
        self.bytes.extend_from_slice(bytes);
    }

    pub fn resize(&mut self, length: usize, value: u8) {
        self.reserve_exact(length.saturating_sub(self.bytes.len()));
        self.bytes.resize(length, value);
    }

    pub fn truncate(&mut self, length: usize) {
        self.bytes.truncate(length);
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
    }

    /// `Vec::shrink_to`, which keeps lent storage where it is.
    pub fn shrink_to(&mut self, capacity: usize) {
        if self.lent == 0 {
            self.bytes.shrink_to(capacity);
        }
    }

    /// Runs `edit` over the bytes as a `Vec` that it may shorten or rewrite
    /// but not grow past its capacity.
    ///
    /// # Panics
    /// Where `edit` reallocated lent storage.
    pub fn edit<R>(&mut self, edit: impl FnOnce(&mut Vec<u8>) -> R) -> R {
        let address = self.bytes.as_ptr();
        let answer = edit(&mut self.bytes);
        assert!(
            self.lent == 0 || std::ptr::eq(address, self.bytes.as_ptr()),
            "a buffer edit reallocated lent storage"
        );
        answer
    }

    /// One more native call holds the bytes' address.
    pub fn lend(&mut self) {
        self.lent += 1;
    }

    /// One call that held the bytes' address has ended; the last frees the
    /// storage growth moved them out of.
    pub fn release(&mut self) {
        self.lent = self.lent.saturating_sub(1);
        if self.lent == 0 {
            self.retired.clear();
        }
    }

    /// How many storages growth has moved the bytes out of while lent.
    pub fn retired(&self) -> usize {
        self.retired.len()
    }
}

impl Clone for BufferBytes {
    /// A copy's bytes, which no call holds.
    fn clone(&self) -> BufferBytes {
        BufferBytes::from(self.bytes.clone())
    }
}

impl From<Vec<u8>> for BufferBytes {
    fn from(bytes: Vec<u8>) -> BufferBytes {
        BufferBytes {
            bytes,
            lent: 0,
            retired: Vec::new(),
        }
    }
}

impl std::ops::Deref for BufferBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.bytes
    }
}

impl std::ops::DerefMut for BufferBytes {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl BufferState {
    /// `MutableBuffer::ensureCapacity`: room for `added` more bytes, taking
    /// the larger of what is needed and twice the capacity. Answers how far
    /// the capacity grew.
    pub fn ensure_capacity(&mut self, added: usize) -> Result<usize, TryReserveError> {
        let needed = self.bytes.len().saturating_add(added);
        if needed > self.capacity {
            let capacity = needed.max(self.capacity.saturating_mul(2));
            self.bytes.try_reserve_exact(capacity - self.bytes.len())?;
            let grown = capacity - self.capacity;
            self.capacity = capacity;
            return Ok(grown);
        }
        Ok(0)
    }

    /// The bytes' address, writable up to the capacity answered with it.
    ///
    /// A copy's `Vec` holds only its length, so the allocation is grown to
    /// the capacity here; where that fails the capacity drops to what the
    /// allocation holds.
    pub fn writable(&mut self) -> (*mut u8, usize) {
        if self.bytes.capacity() < self.capacity
            && self
                .bytes
                .try_reserve_exact(self.capacity.saturating_sub(self.bytes.len()))
                .is_err()
        {
            self.capacity = self.bytes.capacity();
        }
        (self.bytes.as_mut_ptr(), self.capacity)
    }

    /// `MutableBuffer::setBufferSize`: zero empties the contents and takes
    /// the capacity back to `default_size`; any other size becomes the
    /// capacity and truncates contents longer than it. Answers how far the
    /// capacity grew.
    pub fn set_buffer_size(&mut self, size: usize) -> Result<usize, TryReserveError> {
        let before = self.capacity;
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
        Ok(self.capacity.saturating_sub(before))
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

    /// The value `name` holds in the pool created last that binds it, which
    /// is `RexxObject::getObjectVariable`'s walk: the oracle chains each new
    /// dictionary at the head of the list (`ObjectClass.cpp:2504`).
    pub fn find(&self, name: &[u8]) -> Option<ObjRef> {
        self.pools.iter().rev().find_map(|(_, pool)| {
            pool.iter()
                .find(|(bound, _)| **bound == *name)
                .map(|(_, value)| *value)
        })
    }

    /// Every name `scope`'s pool binds, with its value, in binding order.
    pub fn entries(&self, scope: ObjRef) -> &[(Box<[u8]>, ObjRef)] {
        self.pool(scope).map_or(&[], Vec::as_slice)
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

/// One method attached to a single object: the body it runs, the scope its
/// `EXPOSE` binds in, and the `Method` object it is, which the entry keeps
/// alive as the oracle's behaviour keeps the object it holds.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ObjectMethod {
    pub method: MethodId,
    pub scope: ObjRef,
    pub executable: ObjRef,
}

/// The methods attached to one object rather than to its class, searched
/// ahead of the class behaviour (`MethodDictionary::addInstanceMethod`'s
/// `addFront`, `behaviour/MethodDictionary.cpp:399`).
#[derive(Clone, Debug, Default)]
pub struct ObjectMethods {
    set: Vec<(Box<[u8]>, Option<ObjectMethod>)>,
    /// The "Enhanced Subclass" `Class~enhanced` built this object from, whose
    /// instance behaviour the object dispatches against while its `class` is
    /// the receiver: `setOwningClass` and `setEnhanced`
    /// (`classes/ClassClass.cpp:1474`-`:1476`). Held so the object keeps it
    /// alive; `RexxObject::defaultName` answers `enhanced <id>` for such an
    /// object (`classes/ObjectClass.cpp:1763`-`:1767`).
    enhanced_by: Option<ObjRef>,
}

impl ObjectMethods {
    pub fn new() -> ObjectMethods {
        ObjectMethods {
            set: Vec::new(),
            enhanced_by: None,
        }
    }

    /// Marks the object one `Class~enhanced` built from `dummy`.
    pub fn mark_enhanced_by(&mut self, dummy: ObjRef) {
        self.enhanced_by = Some(dummy);
    }

    /// Whether `Class~enhanced` built this object.
    pub fn is_enhanced_instance(&self) -> bool {
        self.enhanced_by.is_some()
    }

    /// Every name `setMethod` defines, upper case, hidden entries excluded.
    pub fn defined_names(&self) -> Vec<Box<[u8]>> {
        self.set
            .iter()
            .filter(|(_, entry)| entry.is_some())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// What this object answers for `name`: `None` when it holds no entry
    /// under it, `Some(None)` for a hidden name, and `Some(Some(method))` for
    /// one it defines.
    pub fn get(&self, name: &[u8]) -> Option<Option<ObjectMethod>> {
        Self::find(&self.set, name).copied()
    }

    /// Adds or replaces `setMethod`'s entry for `name`, which
    /// `addInstanceMethod` does in one step: it removes any method it added
    /// before under that name and puts the new one at the front.
    pub fn set(&mut self, name: &[u8], entry: Option<ObjectMethod>) {
        Self::write(&mut self.set, name, entry);
    }

    /// Removes `setMethod`'s entry for `name`.
    pub fn remove(&mut self, name: &[u8]) {
        self.set.retain(|(key, _)| !key.eq_ignore_ascii_case(name));
    }

    /// Appends every object these entries reach, which is each defined
    /// method's scope and `Method` object, and the enhancing class, for
    /// [`ScopePools::trace`]'s reason.
    fn trace(&self, out: &mut Vec<ObjRef>) {
        for ObjectMethod {
            method: _,
            scope,
            executable,
        } in self.set.iter().filter_map(|(_, entry)| *entry)
        {
            out.push(scope);
            out.push(executable);
        }
        out.extend(self.enhanced_by);
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
    entries: FxHashMap<Box<[u8]>, ObjRef>,
    /// `ObjRef::NIL` for an object whose builder gave it no table.
    annotations: ObjRef,
    bound: Bound,
}

/// What a `Method`, `Routine` or `Package` object is bound to, sharing one
/// slot so that every other native object stays its size.
#[derive(Clone, Debug)]
enum Bound {
    Nothing,
    /// A method object's scope.
    Scope(ObjRef),
    /// The package a `Method` or `Routine` object's code belongs to.
    Code(ObjRef),
    /// A method object's scope and the package its code belongs to.
    ScopedCode {
        scope: ObjRef,
        package: ObjRef,
    },
    /// The `INTERNAL_ROUTINES` row a routine object runs.
    RoutineRow(usize),
    /// A package object's state, boxed so that every other native object
    /// stays its size.
    Package(Box<PackageState>),
}

/// Measured: a field beside `Bound` moves every native object's box to the
/// next allocator size class, +0.19% of instructions on `hashnew.rex`.
const _: () = assert!(size_of::<NativeObject>() <= 104);

impl NativeObject {
    pub fn new(class: ObjRef, rendered: &[u8]) -> NativeObject {
        NativeObject {
            class,
            rendered: rendered.into(),
            string_value: None,
            entries: FxHashMap::default(),
            annotations: ObjRef::NIL,
            bound: Bound::Nothing,
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

    /// Room for `additional` more entries, for a builder that knows how many
    /// it sets.
    pub fn reserve_entries(&mut self, additional: usize) {
        self.entries.reserve(additional);
    }

    pub fn remove_entry(&mut self, key: &[u8]) {
        self.entries.remove(key);
    }

    /// Every key this object holds, as owned copies.
    pub fn keys(&self) -> Vec<Box<[u8]>> {
        self.entries.keys().cloned().collect()
    }

    /// The `StringTable` this object's `~annotations` answers, or `None` for
    /// an object whose builder gave it none.
    pub fn annotations(&self) -> Option<ObjRef> {
        (self.annotations != ObjRef::NIL).then_some(self.annotations)
    }

    pub fn set_annotations(&mut self, annotations: ObjRef) {
        self.annotations = annotations;
    }

    /// `MethodClass::getScope`: the class this method object is installed on,
    /// or `None` for one nothing has installed yet.
    pub fn scope(&self) -> Option<ObjRef> {
        match self.bound {
            Bound::Scope(scope) | Bound::ScopedCode { scope, .. } => Some(scope),
            _ => None,
        }
    }

    /// `MethodClass::setScope` (`classes/MethodClass.cpp:219`).
    pub fn set_scope(&mut self, scope: ObjRef) {
        self.bound = match self.bound {
            Bound::Code(package) | Bound::ScopedCode { package, .. } => {
                Bound::ScopedCode { scope, package }
            }
            _ => Bound::Scope(scope),
        };
    }

    /// The package a `Method` or `Routine` object's code belongs to, which
    /// the object keeps alive, or `None` for one that names none.
    pub fn executable_package(&self) -> Option<ObjRef> {
        match self.bound {
            Bound::Code(package) | Bound::ScopedCode { package, .. } => Some(package),
            _ => None,
        }
    }

    /// Records the package this `Method` or `Routine` object's code belongs
    /// to, keeping any scope.
    pub fn set_executable_package(&mut self, package: ObjRef) {
        self.bound = match self.bound {
            Bound::Scope(scope) | Bound::ScopedCode { scope, .. } => {
                Bound::ScopedCode { scope, package }
            }
            _ => Bound::Code(package),
        };
    }

    /// The `INTERNAL_ROUTINES` row this `Routine` object runs, which a copy
    /// carries too, or `None` for one that runs none.
    pub fn routine_row(&self) -> Option<usize> {
        match self.bound {
            Bound::RoutineRow(row) => Some(row),
            _ => None,
        }
    }

    /// Records the row this `Routine` object runs, in place of any scope.
    pub fn set_routine_row(&mut self, row: usize) {
        self.bound = Bound::RoutineRow(row);
    }

    /// The state this `Package` object holds, or `None` for another object.
    #[inline]
    pub fn package(&self) -> Option<&PackageState> {
        match &self.bound {
            Bound::Package(state) => Some(state),
            _ => None,
        }
    }

    /// [`NativeObject::package`] for a caller that changes it.
    #[inline]
    pub fn package_mut(&mut self) -> Option<&mut PackageState> {
        match &mut self.bound {
            Bound::Package(state) => Some(state),
            _ => None,
        }
    }

    /// Makes this object a package holding `state`.
    pub fn set_package(&mut self, state: PackageState) {
        self.bound = Bound::Package(Box::new(state));
    }

    /// Takes this object's package state, leaving it bound to nothing.
    pub fn take_package(&mut self) -> Option<PackageState> {
        match std::mem::replace(&mut self.bound, Bound::Nothing) {
            Bound::Package(state) => Some(*state),
            other => {
                self.bound = other;
                None
            }
        }
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
            slots: ArraySlots::new(slots),
            dimensions: None,
        }
    }

    /// A `.Pointer` over `address`, an instance of `class` with no variables
    /// and no methods of its own (`PointerClass::operator new`,
    /// `interpreter/classes/PointerClass.cpp:122`, which marks the object as
    /// having no references).
    pub fn pointer(
        class: ObjRef,
        behaviour: BehaviourHandle,
        address: *mut std::ffi::c_void,
    ) -> Body {
        Body::Instance {
            class,
            behaviour,
            name: None,
            pools: ScopePools::new(),
            own: None,
            native: Some(Box::new(NativeState::Pointer(address))),
        }
    }

    /// The bytes this body holds in an allocation of its own: a `Text`'s
    /// bytes when they do not fit inline, an `Array`'s slot capacity, and a
    /// `MutableBuffer`'s capacity. Nothing for every other body.
    pub fn held_bytes(&self) -> usize {
        match self {
            Body::Text { bytes, .. } => bytes.heap_len(),
            Body::Array { slots, .. } => slots.capacity() * SLOT_BYTES,
            Body::Instance {
                native: Some(native),
                ..
            } => match &**native {
                NativeState::Buffer(state) => state.capacity,
                _ => 0,
            },
            _ => 0,
        }
    }

    /// The state a `Package` object holds, whether it is the class's own
    /// instance or a subclass's, or `None` for any other body.
    #[inline]
    pub fn package(&self) -> Option<&PackageState> {
        match self {
            Body::Native(native) => native.package(),
            Body::Instance {
                native: Some(state),
                ..
            } => state.package(),
            _ => None,
        }
    }

    /// [`Body::package`] for a caller that changes it.
    #[inline]
    pub fn package_mut(&mut self) -> Option<&mut PackageState> {
        match self {
            Body::Native(native) => native.package_mut(),
            Body::Instance {
                native: Some(state),
                ..
            } => state.package_mut(),
            _ => None,
        }
    }

    /// Appends every object this one can reach.
    pub fn trace(&self, out: &mut Vec<ObjRef>) {
        match self {
            // Neither reaches an `ObjRef`: a `Number` and a byte string are
            // both plain data, never a handle into the heap.
            // `Heap::collect`'s mark loop relies on this: it reads a `Text`
            // survivor's bytes and never calls `trace` on it.
            Body::Text { .. } => {}
            Body::Num { .. } => {}
            Body::Stem {
                default,
                tails,
                exposed,
                ..
            } => {
                out.extend(default.iter().copied());
                // A tombstone (`None`) reaches nothing, same as a weak
                // reference clearing to `.nil` -- it is present but dead.
                out.extend(tails.values().filter_map(|(_, tail)| *tail));
                out.extend(exposed.iter().flat_map(|exposed| exposed.values().copied()));
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
                class,
                pools,
                own,
                native,
                ..
            } => {
                out.push(*class);
                pools.trace(out);
                if let Some(native) = native {
                    native.trace(out);
                }
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
                out.extend(native.scope());
                out.extend(native.executable_package());
                // Reachable from the object alone once a program holds the
                // table and drops every other handle on it.
                out.extend(native.annotations());
                if let Some(state) = native.package() {
                    state.trace(out);
                }
            }
            // Deliberately reaches nothing: a weak reference must not keep
            // its target alive.
            Body::WeakRef(_) => {}
            Body::Class { owned } => out.extend(owned.iter().copied()),
            // The owning object of an instance variable is traced because the
            // reference may be the only handle a program still has on it.
            Body::VarRef(reference) => match reference.home {
                VarRefHome::Cell(slot) => out.extend(slot.cell()),
                VarRefHome::Instance { owner, scope } => {
                    out.push(owner);
                    out.push(scope);
                }
            },
            // The value is in the root set's cell, which `Heap::collect`
            // marks when it marks this object.
            Body::Cell(..) => {}
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
    /// Set once a `GUARD WHEN` has watched one of this object's variables,
    /// and never cleared, as `RexxVariable::dependents` stays once created
    /// (`execution/RexxVariable.cpp:137-150`): the one test a variable store
    /// or drop makes before looking for watchers.
    pub(crate) watched: bool,
}

impl Object {
    /// Whether a `GUARD WHEN` has ever watched one of this object's
    /// variables. See the field.
    #[inline(always)]
    pub fn watched(&self) -> bool {
        self.watched
    }

    /// Marks this object as one whose variables a `GUARD WHEN` has watched.
    pub fn mark_watched(&mut self) {
        self.watched = true;
    }

    /// Whether this object defines an `UNINIT` method. See the field.
    pub fn has_uninit(&self) -> bool {
        self.has_uninit
    }
}

#[cfg(test)]
mod aligned_tests {
    use super::{AlignedBytes, Word};

    /// **A buffer's bytes start 16-byte aligned**, and read back as written.
    #[test]
    fn aligned_bytes_are_aligned_and_read_back() {
        assert_eq!(std::mem::align_of::<Word>(), 16);
        let mut bytes = AlignedBytes::zeroed(20);
        assert_eq!(bytes.as_mut_ptr() as usize % 16, 0);
        bytes.write(15, b"abcdefgh");
        let read = bytes.to_vec();
        assert_eq!((read.len(), &read[15..20]), (20, &b"abcde"[..]));
    }
}
