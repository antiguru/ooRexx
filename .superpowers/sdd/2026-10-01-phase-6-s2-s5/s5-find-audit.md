# S5 findings: audit (Task 23 Steps 1-2, criteria 4 and 5)

HEAD 540e6a72e. Evidence: `s5-evidence/audit/`.

## What was measured

1. The brief's command, run from the repo root:
   `/bin/grep -a -rn 'unsafe impl Send\|unsafe impl Sync\|thread::spawn\|Arc<\|Mutex<\|Atomic' rust/crates`
   -> 108 lines, `s5-evidence/audit/grep-hits.txt`.
2. What that pattern does not match: `/bin/grep -a -rn 'unsafe impl' rust/crates` and
   `/bin/grep -a -rn -E 'thread::(Builder|scope)|Condvar|thread_local!|OnceLock|LazyLock|lazy_static' --include=*.rs rust/crates`
   -> `s5-evidence/audit/pattern-misses.txt`.
3. A proposed replacement pattern (below) -> 183 lines, `s5-evidence/audit/proposed-pattern.txt`.
4. Existing type-level facts and tests, run in my own target dir:
   `memcap 8G cargo test -p rexx-core --doc` (3 compile_fail + 2 controls, all ok) and
   `memcap 8G cargo test -p rexx-exec --lib -- a_context_reads_an_activation_of_another_live_activity a_context_follows_its_activation_to_a_reply_continuation island::`
   (4 passed) -> `s5-evidence/audit/tests-run.txt`.
5. Probes of proposed static assertions:
   - scratch crate on rexx-core: RegFrame into `Box<dyn Any>` fails E0597; RegFrame into a `T: Send`
     bound fails E0277; `require_static::<Body>()` compiles -> `probe-regframe.txt`.
   - scratch `git archive 540e6a72e` tree: `require_send::<Posted>()` in scheduler.rs compiles;
     the not-Send assertion for `Interp` and `RegFrame<'static>` in island.rs compiles (control:
     the same assertion on `u64` fails E0283); `require_send::<Box<OffBaton>>()` FAILS because
     `CStringPool` holds `Vec<(ObjRef, Vec<u8>)>` -> `probe-offbaton-send.txt`, `probe-island-tail.rs.txt`.

## Results

### R1. The committed pattern misses the central grant

The brief's pattern `unsafe impl Send` does not match the generic form
`unsafe impl<T> Send for Islanded<T> {}` (`rexx-exec/src/island.rs:38`), the D-U2 grant itself. It
also misses every production thread start that is not `thread::spawn`: the pool
(`scheduler/pool.rs:123`, `thread::Builder`), the timer thread (`timer.rs:205`), the interpreter
thread (`lib.rs:3124`), the pipe drain (`command.rs:618`, `thread::scope`), and the non-test
`thread_local!`s (`rexx-api/src/ffi.rs:700` CALLING, `layout.rs:421` REFUSED, `load.rs:1518`
HOOK_THREW). An inventory from the brief's command alone is not complete.

Proposed committed command (183 hits, a superset of the brief's 108):

    /bin/grep -a -rn 'unsafe impl\|thread::spawn\|thread::Builder\|thread::scope\|Arc<\|Mutex<\|Condvar\|Atomic\|thread_local!' rust/crates

Plain `static` items need no row: a `static` must be `Sync`, which the compiler checks.

### R2. Islanded carries ObjRefs across threads

`Islanded<T>` is instantiated twice (`grep -rn 'Islanded' rust/crates`): `Island =
Islanded<NonNull<Interp>>` (island.rs:57) and `PooledCall::work: Islanded<(Box<OffBaton>,
ThreadContext)>` (`dispatch/library.rs:1077`). The probe shows `OffBaton` is not `Send` because of
`ObjRef`s in `CStringPool` (`rexx-api/src/values.rs:390`). So spec 5's "the compiler refuses an
object handle in the inbox, a completion or another interpreter" holds for the inbox and completions
(`Posted: Send` verified) but object handles DO reach a pool thread, inside the `Islanded` wrapper,
used there only under a lend (`PooledCall::run` takes it after `Lent::wait`). The SAFETY note covers
this ("a value made here is made on the baton ... taken out only by `Islanded::take`"), and the two
`should_panic` tests in island.rs test the runtime checks. What is missing is a type-level bound on
which payloads `Islanded` may carry: `unsafe impl<T> Send` is for every `T`.

### R3. Non-test thread_locals are three, not two

Spec 2.5 says `REFUSED` and `HOOK_THREW` are the only non-test `thread_local!`s. At HEAD a third
exists: `CALLING` (`rexx-api/src/ffi.rs:700`, a raw pointer to the innermost `Entered` on this OS
thread's stack). Command: `/bin/grep -a -rn -B3 'thread_local!' --include=*.rs rust/crates` (every
other site is `#[cfg(test)]` or in a test file). All three are per native call on one OS thread's
stack, hold no `ObjRef`, and are sound when the activity moves threads because a native call does not
move threads mid-call.

### R4. Existing static-assertion mechanisms

- compile_fail doctests: `ObjRef` not Send, not Sync, with a positive control
  (`rexx-core/src/handle.rs:52`, `:57`, `:65`); a frame cannot outlive its arena, E0597, with a
  control (`rexx-core/src/frame.rs:139`, `:150`). All pass (tests-run.txt).
- `const _` positive Send assertion: `require_send::<NativeCall>()`,
  `require_send::<Completion>()` (`rexx-api/src/invoke.rs:250-254`).
- No crate (`static_assertions`, `trybuild`) is a dependency. A negative assertion on a crate-private
  type (`Interp`) cannot be a doctest; the ambiguity trick (R5 item A2) works on stable and was
  probed.

### R5. Criterion 5 (D3)

Facts at HEAD:
- `Body` (`rexx-core/src/body.rs:57`) has no lifetime parameter, so it cannot name a `RegFrame<'a>`
  for a non-`'static` `'a`.
- `RegFrame<'a>` (`rexx-core/src/frame.rs:88`) borrows `&'a FrameArena` by `PhantomData`; arenas are
  owned by `Rc<FrameArena>` in `ActivityRoots` (`rexx-core/src/roots.rs:81`); no `Box::leak` and no
  `static` FrameArena in `rust/crates` (`/bin/grep -rn 'Box::leak\|static.*FrameArena'`: no hits), so
  no `RegFrame<'static>` is made.
- Parking holds `ParkedFrame { len, opened }` (frame.rs:127), no pointer.
- Caveat: `NativeState::Pointer(*mut c_void)` (`body.rs:171`) is a raw pointer in a heap body; raw
  pointers are outside the lifetime fact. It holds C-supplied addresses only.

Task 7's test, at HEAD in `rust/crates/rexx-exec/src/scheduler/tests.rs`:
- `a_context_reads_an_activation_of_another_live_activity` (`:981`): live activation of another
  activity read (line, name, variables, digits, thread 2 vs 1, stackFrames, args), then 98.981 once
  it ended (the finished case). Modes unswitched, collect at every alloc, EveryOpportunity.
- `a_context_follows_its_activation_to_a_reply_continuation` (`:1013`): the moved (REPLY) case,
  `~invocation` kept, before the continuation runs and while it waits.
- Also: corpus witnesses `context_of_another_activity`, `context_moved_by_reply`
  (`rust/corpus/phase-8.txt`); `tests/outer_context.rs:178`, `:247` (kept call and thread contexts
  from another activity).
- Lookup code: `dispatch/context.rs:159` `at_context`, `scheduler.rs:713` `idle_context_owner`.

Together these cover the spec's three cases (live, finished, moved). What is missing for the
type-level half is a committed check that pins the fact (today it is implied by `Body`'s
declaration, and the only compile_fail shows a frame cannot outlive its arena, not that it cannot
enter a `'static` heap value).

## What the implementer must build or change

A. Static assertions (all probed to compile or fail as stated):
1. `rexx-core/src/frame.rs`, on `RegFrame`'s doc, two compile_fail doctests:
   - `compile_fail,E0597`: `fn keep(_: Box<dyn std::any::Any>) {} let arena = FrameArena::new(FrameBlock::DEFAULT); keep(Box::new(arena.reserve(1)));`
     ("a frame cannot be held by a `'static` value, which every heap `Body` is").
   - `compile_fail,E0277`: `fn require_send<T: Send>(_: T) {} let arena = FrameArena::new(FrameBlock::DEFAULT); require_send(arena.reserve(1));`
   - positive control: the same `keep` with `frame.get(0)` (an `ObjRef`, `'static`) compiles.
2. `rexx-core/src/body.rs`: `const _: () = { const fn require_static<T: 'static>() {} require_static::<Body>(); };`
3. `rexx-exec/src/island.rs`: a not-Send assertion (static_assertions' `assert_not_impl_any`
   technique) for `Interp` and `RegFrame<'static>`:
   `trait AmbiguousIfSend<A> { fn some_item() {} } impl<T: ?Sized> AmbiguousIfSend<()> for T {} impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {} const _: fn() = || { let _ = <Interp as AmbiguousIfSend<_>>::some_item; let _ = <rexx_core::RegFrame<'static> as AmbiguousIfSend<_>>::some_item; };`
   Check clippy `-D warnings` on it (not run by me).
4. `rexx-exec/src/scheduler.rs`: `const _: () = { const fn require_send<T: Send>() {} require_send::<Posted>(); };`
   (today implied only through `timer.rs`'s `static REGISTRY: LazyLock<Registry>`).
5. `rexx-exec/src/island.rs`: bound `Islanded`'s payloads, e.g. `pub(crate) trait IslandPayload {}`
   implemented for `NonNull<Interp>` and `(Box<OffBaton>, rexx_api::ffi::ThreadContext)`, and
   `unsafe impl<T: IslandPayload> Send for Islanded<T> {}`, so a third payload is a compile error until
   listed (and its SAFETY argument read). If the lead prefers no code change, the gate row must name
   the two instantiations and the grep that finds them.

B. Gate record: replace the brief's command with R1's, commit its output, and paste the table below.
Fix spec 5's "refuses an object handle ... in another interpreter" sentence only if the lead rules it
(R2: true for inbox and completions; ObjRefs ride `Islanded` to a pool thread of the same
interpreter). Fix spec 2.5's "only non-test `thread_local!`s" (R3) the same way.

C. Ready-to-paste table for `phase-6-gate.md` (S5, criterion 4). Line numbers at 540e6a72e.

| Site | Cross-thread type | Why sound | Fact or test |
|---|---|---|---|
| `rexx-exec/src/island.rs:38` | `Islanded<T>`: `Island` (root pointer), `PooledCall::work` (`Box<OffBaton>` with ObjRefs, `ThreadContext` Rc) | made and taken only on the baton; moved untouched; `Lent<'b>` bounds the `&mut Interp` | runtime asserts tested by `island::tests::an_island_value_is_made_only_on_the_baton`, `..._taken_only_on_the_baton`; payload bound A5 (to add); TSan run (criterion 3) |
| `rexx-exec/src/baton.rs:21,26` | `Baton<L>`: `Mutex<State>`, `Condvar`, test `AtomicU64` | the holder/lend state is the lock's data; `lent()` answers only to the lendee | `tests/loom.rs` (baton models); TSan |
| `rexx-exec/src/timer.rs:64,85,178,245,287,321`; `lib.rs:1223`; `dispatch/library.rs:1081-1211` | `Arc<Inbox<Posted>>`, `Arc<InterpBaton>`, `Requests(AtomicU32)` | `Posted: Send` (holds no ObjRef); request bits publish no data | `require_send::<Posted>()` (A4, to add); `ObjRef` compile_fail (`handle.rs:52,57`); `tests/loom.rs` inbox/timer models; `scheduler::tests::a_post_from_another_thread_ends_an_idle` |
| `rexx-exec/src/scheduler/pool.rs:52-83,123,177` | `Arc<Shared>`, `Arc<Mailbox>`, `Job = Box<dyn FnOnce() + Send>` | a job is `Send` by its type; the three job kinds (`scheduler.rs:627` native call, `:658` blocking op, `input.rs:377` stdin) capture only `Send` values plus `Islanded` | type of `Job`; `scheduler/tests/pool.rs` |
| `rexx-exec/src/timer.rs:205` | timer thread | touches only the registry (`Mutex<State>`) and inboxes | `static REGISTRY` (Sync by compiler); loom timer models |
| `rexx-exec/src/lib.rs:3124` | interpreter thread | body is `FnOnce + Send + 'static`; `Interp` is built on it | closure bound; `Interp` not Send (A3, to add) |
| `rexx-exec/src/command.rs:618` | pipe drain `thread::scope` | captures pipes and `&(dyn Fn + Sync)` only | the `Sync` bound |
| `rexx-exec/src/signal.rs:36,40` | `static PENDING: AtomicBool`, `WAKE: AtomicI32` | async-signal-safe flags; carry no data | `signal.rs` unit tests (`:237`, `:269` raisers); loom `timer::signal` models |
| `rexx-exec/src/sync.rs` | std or loom re-exports, `Wake` | the shim the loom models compile | `tests/loom.rs` |
| `rexx-api/src/ffi.rs:109,112` | `unsafe impl Send for ValueDescriptor`, `Value` | a word and two ints; pointer members are addresses, dereferenced `unsafe` at their sites | `require_send::<NativeCall>()`, `<Completion>()` (`invoke.rs:250-254`); `rexx-api/tests/invoke.rs:754-760` moves both across threads |
| `rexx-api/src/ffi.rs:619` | `Requester = Arc<dyn Baton + Send + Sync>` | a foreign thread reaches the interpreter only by taking the baton | `HostRef` debug asserts (`ffi.rs:192`, `:208`, `:1320`) tested by `ffi.rs:6330`; `scheduler/tests/callbacks.rs` (`a_callback_from_another_thread_takes_the_baton_and_raises_98_983` and siblings) |
| `rexx-api/src/ffi.rs:700`, `layout.rs:421`, `load.rs:1518` | non-test `thread_local!` CALLING, REFUSED, HOOK_THREW | per native call on one OS thread's stack; no ObjRef | `thread_local!` is not Send by construction |
| `rexx-api/src/load.rs:113-205,1157-2018` | `Arc<Mapping>` (`Mutex<Option<Library>>`, `AtomicBool`, `AtomicUsize`) | a close refuses while a call is counted in flight; the row outlives its mapping by the Arc | compile_fail `load.rs:191`, `:1681`; `load.rs` tests |
| `rexx-api/src/load.rs:746-900` (doc-hidden stubs) | foreign-thread and buffer-handshake test stubs | plays an extension's foreign thread | `scheduler/tests/callbacks.rs:68`, `scheduler/tests/lent.rs` |
| `rexx-parse/src/selector.rs:19,44` | `Selector(Arc<[u8]>)` | immutable bytes; never crosses a thread today | `Arc<[u8]>: Send + Sync` |
| `rexx-api/src/ffi.rs:6283` (test), `rexx-exec/src/scheduler/tests*.rs`, `rexx-exec/tests/*` (loom, signals, stdin_contention, prompt_before_read, program_end, support/oracle, support/group_runner, watchdog), `rexx-classes/tests/*`, `rexx-api/tests/invoke.rs`, `dispatch/library/tests.rs:735`, `rexx-parse/tests/deep.rs`, `rexx-parse/examples/depth_probe.rs`, `rexx-exec/src/ir/corpus_shape_tests.rs:540`, `rexx-exec/src/bin/rexx-ir.rs:43` | test harness, tool and probe threads | outside the interpreter or test fixtures | not shipped in the interpreter library |

Criterion 5 row:

| Claim | Fact or test |
|---|---|
| no heap object holds a frame | `Body` has no lifetime (`body.rs:57`); `RegFrame<'a>` borrows its arena (`frame.rs:88`, compile_fail `frame.rs:139`); A1 and A2 to add |
| contexts resolve across all activities | `dispatch/context.rs:159` `at_context`, `scheduler.rs:713` `idle_context_owner` |
| live, finished, moved | `scheduler::tests::a_context_reads_an_activation_of_another_live_activity` (live, finished 98.981), `scheduler::tests::a_context_follows_its_activation_to_a_reply_continuation` (moved); corpus `context_of_another_activity`, `context_moved_by_reply` |

## Open questions for the lead

1. Adopt R1's wider pattern as the committed command, or keep the brief's and add a second command
   for the misses?
2. A5 (seal `Islanded`'s payloads) is a code change in an `unsafe` module: rule yes, or record the two
   instantiations only?
3. R2 and R3 contradict spec sentences (5 "Isolation at the type level", 2.5 "only non-test
   `thread_local!`s"). Amend the spec, or record the deviation in the gate?
4. The `Body::Pointer(*mut c_void)` caveat: state it in the criterion 5 row, or leave it out as C-only?
