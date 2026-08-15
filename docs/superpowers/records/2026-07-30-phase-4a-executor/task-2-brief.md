### Task 2: `rexx-core` gains the value bodies and root-set slot frames

**Spec:** D15 "Where these variants live, and what that costs", D15a, and D16's `RootSet` bullet.

**Files:**
- Modify: `rust/crates/rexx-core/src/body.rs`, `src/roots.rs`, `src/heap.rs` (tests), `Cargo.toml`
- Test: `rust/crates/rexx-core/tests/collect.rs`

**Interfaces:**
- Produces:
  ```rust
  pub enum Body {
      Text { bytes: Vec<u8>, num: Option<Result<Box<Number>, NotNumeric>> },
      Num { value: Number, created_digits: u32, created_form: Form, text: Option<Vec<u8>> },
      Stem { name: Box<[u8]>, default: Option<ObjRef>, tails: HashMap<Vec<u8>, Option<ObjRef>> },
      Array(Vec<ObjRef>),
      Instance(Vec<(String, ObjRef)>),
      WeakRef(ObjRef),
  }
  ```
  `Body::String` is deleted. `BehaviourId::STEM` is added.
- Produces on `RootSet`: `push_slots(initial_len: usize) -> SlotFrame`, `pop_slots(SlotFrame)`, `slot(SlotFrame, usize) -> Option<ObjRef>`, `set_slot(&mut self, SlotFrame, usize, ObjRef)`, `grow_slots(&mut self, SlotFrame) -> usize` returning the new index.

**Why:** the values 4a manipulates are heap objects, and `Body` lives here. And an activation's variables must be reachable from the collector: `RootSet` is globals plus temps, so as it stands the first collection sweeps every local.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn a_stems_tails_and_default_are_traced() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let tail = heap.alloc(Body::Text { bytes: b"kept".to_vec(), num: None });
    let default = heap.alloc(Body::Text { bytes: b"dflt".to_vec(), num: None });
    let mut tails = HashMap::new();
    tails.insert(b"1".to_vec(), Some(tail));
    // A tombstone: present, and reaching nothing.
    tails.insert(b"2".to_vec(), None);
    let stem = heap.alloc_with(BehaviourId::STEM, Body::Stem {
        name: b"A.".to_vec().into_boxed_slice(),
        default: Some(default),
        tails,
    });
    roots.add_global("a.", stem);
    heap.collect(&roots);
    assert_eq!(heap.get(tail).is_some(), true, "a live tail was swept");
    assert_eq!(heap.get(default).is_some(), true, "the stem default was swept");
}

#[test]
fn slot_frames_keep_locals_alive_and_release_them_on_pop() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let frame = roots.push_slots(2);
    let v = heap.alloc(Body::Text { bytes: b"local".to_vec(), num: None });
    roots.set_slot(frame, 0, v);
    heap.collect(&roots);
    assert!(heap.get(v).is_some(), "a live local was swept");
    roots.pop_slots(frame);
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1, "the local outlived its frame");
}

#[test]
fn a_slot_frame_grows_for_a_name_the_plan_never_saw() {
    let mut roots = RootSet::new();
    let frame = roots.push_slots(1);
    let index = roots.grow_slots(frame);
    assert_eq!(index, 1);
}
```

- [ ] **Step 2: Run them to watch them fail**

Run: `cd rust && cargo test -p rexx-core`
Expected: compile errors — `Body::Text`, `Body::Stem`, `push_slots` do not exist.

- [ ] **Step 3: Add the variants and extend `trace`**

`Body::trace` must gain arms for `Text` (reaches nothing) and `Stem` (reaches `default` and every `Some` tail). **It has no wildcard arm and must not gain one**: that exhaustive match is the whole of Phase 1's GC-safety argument, so a new variant has to be a compile error here rather than a use-after-free later.

`rexx-core/Cargo.toml` gains `rexx-num`, because `Body::Text` holds a `Number`. That edge is new and points the object model at the arithmetic core; it is declared in the spec and is not an accident to be quietly avoided.

Delete `Body::String` and update `heap.rs`'s `retire_tests`, which construct it.

- [ ] **Step 4: Implement slot frames**

Only the **top** frame ever grows **in 4a**, which has one frame, and for `INTERPRET`, which runs inside the activation that created it. Assert it: `grow_slots` on a frame that is not the top one is a panic with a message saying so, because a silent wrong answer here is a variable that lands in another routine's pool.

Write in the doc comment that this is a **4a invariant and not a general one**, and why: measured, `sub: procedure expose zzz` makes a callee write into its caller's pool while the callee's frame is on top, so 4b either grows a non-top frame or binds exposed names to caller slots at call time. A panic that a later sub-phase must remove is the right shape here; a silent allowance it would inherit is not.

`iter()` must yield globals, temps **and** every assigned slot, so `collect`'s signature does not change.

- [ ] **Step 5: Verify**

Run: `cd rust && cargo test -p rexx-core && cargo clippy -p rexx-core --all-targets -- -D warnings`
Expected: all pass, including the three new tests.

- [ ] **Step 6: Commit**

```bash
git add rust/crates/rexx-core
git commit -m "Add the executor's value bodies and root-set slot frames"
```

---

