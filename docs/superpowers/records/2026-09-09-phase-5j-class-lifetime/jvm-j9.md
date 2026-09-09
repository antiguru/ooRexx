# How the JVM and OpenJ9 make class metadata collectable

Research note for the Rexx-in-Rust collector decision: *built-in classes may keep a static
lifetime; every user-defined class must be collected like an ordinary object; no permgen.*

Sources are cited inline. Claims I could not source are marked `UNSOURCED:` on the sentence.
Claims that are my reading rather than the document's words are marked `INFERENCE:`.
Line and file references to OpenJDK are against `openjdk/jdk` `master` as fetched on
2026-09-09; OpenJ9 references are against `eclipse-openj9/openj9` `master`, same date.

---

## 0. The one-sentence answer

The JVM does **not** have per-class liveness. It has per-*container* liveness, where the
container is the class loader, and it obtains per-class granularity — when it needs it — by
creating **a container of one**. Everything else (Metaspace, CDS, ROMClass/RAMClass,
concurrent unloading) is footprint, startup and pause-time engineering layered on that single
rule. For a design that has no classloaders, the transferable content is the rule and its
soundness argument, not the machinery.

---

## 1. Classloader-rooted liveness: the rule and why it is per-loader

### 1.1 What the specification says

The Java Language Specification, §12.7 *Unloading of Classes and Interfaces*:

> An implementation of the Java programming language may *unload* classes.
>
> A class or interface may be unloaded **if and only if** its defining class loader may be
> reclaimed by the garbage collector [...]
>
> Classes and interfaces loaded by the bootstrap loader may not be unloaded.

(<https://docs.oracle.com/javase/specs/jls/se21/html/jls-12.html>, §12.7; emphasis mine.)

Note it is an *iff*, not an *only if*. **A single unused class in a live loader can never be
collected.** That is not an implementation limitation; it is the specified rule.

### 1.2 The soundness argument — and it is not a GC argument

The JLS spells the reasoning out, and it is worth reading closely because it transfers
directly to Rexx:

> if a class or interface C was unloaded while its defining loader was potentially reachable,
> then C might be reloaded. One could never ensure that this would not happen. [...]
>
> Reloading may not be transparent if, for example, the class has `static` variables (whose
> state would be lost), static initializers (which may have side effects), or `native` methods
> (which may retain static state). Furthermore, the hash value of the `Class` object is
> dependent on its identity. Therefore it is, in general, impossible to reload a class or
> interface in a completely transparent manner.
>
> Since we can never guarantee that unloading a class or interface whose loader is potentially
> reachable will not cause reloading, and reloading is never transparent, but unloading must be
> transparent, it follows that one must not unload a class or interface while its loader is
> potentially reachable.

The invariant is therefore **not** "nothing points at it". It is:

> **A class may be destroyed only when nothing can ever ask for it by name again.**

The loader is chosen as the unit because the loader *is* the thing that answers name queries.
The GC reachability rule is a conservative, cheap approximation of "can no longer be asked".

The same paragraph derives the bootstrap-loader exemption: the boot loader is always reachable,
so its classes are permanently live. **The JVM's "built-in classes are immortal" is not a
separate mechanism — it is a corollary of the single rule.**

### 1.3 The reachability rule as actually implemented in HotSpot

Three edges, all real code:

**instance → its class → its loader.** When a marking closure that cares about metadata visits
an object, the object's `Klass` is followed, and following a `Klass` means scanning its
`ClassLoaderData`:

```cpp
// src/hotspot/share/memory/iterator.inline.hpp:50-56
inline void ClaimMetadataVisitingOopIterateClosure::do_cld(ClassLoaderData* cld) {
  cld->oops_do(this, _claim);
}

inline void ClaimMetadataVisitingOopIterateClosure::do_klass(Klass* k) {
  ClaimMetadataVisitingOopIterateClosure::do_cld(k->class_loader_data());
}
```

The dispatch into it is in `InstanceKlass::oop_oop_iterate`
(`src/hotspot/share/oops/instanceKlass.inline.hpp:175-176`), guarded by
`Devirtualizer::do_metadata(closure)` — collectors that are not doing class unloading pass a
closure whose `do_metadata()` returns `false`
(`BasicOopIterateClosure`, `src/hotspot/share/memory/iterator.hpp:118`), and then the klass edge
is simply not traversed. **The class-liveness edge is an opt-in cost paid only on unloading
cycles.**

**loader-data → the loader object and every class mirror.** `ClassLoaderData::oops_do` walks
one thing: a handle list.

```cpp
// src/hotspot/share/classfile/classLoaderData.cpp:363-370
void ClassLoaderData::oops_do_slow(OopClosure* f, bool clear_mod_oops) {
  if (clear_mod_oops) { clear_modified_oops(); }
  _handles.oops_do(f);
}
```

Two kinds of thing are in that handle list. The loader object itself, added in the constructor
(`classLoaderData.cpp:160`, `_class_loader = _handles.add(h_class_loader());`), and **every
class mirror**:

```cpp
// src/hotspot/share/oops/klass.cpp:64-68
void Klass::set_java_mirror(Handle m) {
  assert(!m.is_null(), "New mirror should never be null.");
  assert(_java_mirror.is_empty(), "should only be used to initialize mirror");
  _java_mirror = class_loader_data()->add_handle(m);
}
```

**class → its static field values.** Statics are not a side table; they are *inside* the mirror
object:

> An InstanceMirrorKlass is a specialized InstanceKlass for `java.lang.Class` instances. These
> instances are special because they contain the static fields of the class in addition to the
> normal fields of Class.
> — `src/hotspot/share/oops/instanceMirrorKlass.hpp:35-40`

So static field values are ordinary heap objects reached from an ordinary heap object (the
mirror), which is reached from the CLD's handle area, which is a root only while the CLD is
alive. **No special handling of statics is needed anywhere in the collector.** This is one of
the cleanest ideas in the design and it transfers.

**Is the CLD alive?** One line:

```cpp
// src/hotspot/share/classfile/classLoaderData.cpp:696-701
bool ClassLoaderData::is_alive() const {
  bool alive = (_keep_alive_ref_count > 0) // null class loader and incomplete non-strong hidden class.
      || (_holder.peek() != nullptr);      // and not cleaned by the GC weak handle processing.
  return alive;
}
```

`_holder` is a **weak** handle, set once, to the loader object (or, for hidden classes, to the
class's own mirror):

```cpp
// src/hotspot/share/classfile/classLoaderData.cpp:544-549
void ClassLoaderData::initialize_holder(Handle loader_or_mirror) {
  if (loader_or_mirror() != nullptr) {
    assert(_holder.is_null(), "never replace holders");
    _holder = WeakHandle(Universe::vm_weak(), loader_or_mirror);
  }
}
```

That is the whole liveness protocol: **strong from the container to its contents; weak from the
container to the thing that decides whether the container lives.**

### 1.4 What the rule buys, and what it forbids

Buys:

* Cycle-immunity for free. Class ↔ superclass ↔ metaclass ↔ subclass list ↔ method dictionary
  are all *inside* one container and can point at each other however they like. Only the
  container's single weak holder edge decides. This is the direct answer to
  "data that is static from within, and not so from the outside": the internal web is made
  strong-but-not-root, and exactly one edge is made external and weak.
* One liveness decision per container rather than per class. HotSpot's `do_unloading` is a
  linear walk of the loader list, not of all classes.
* Bulk deallocation (§3.3): the container owns an arena, and death frees the arena.

Forbids:

* Collecting one dead class out of a live container. Never, by specification.
* Therefore: every class on the application classpath is permanently live in practice, because
  the system class loader is permanently reachable. INFERENCE: the JLS rule plus a permanently reachable
  application loader implies this; I did not find a document stating it in those words.
* **Amplification.** One live instance of one class pins its class, hence its loader, hence
  *every* class that loader defined and every static field value of every one of them. This is
  the mechanism behind every classloader leak in §6.

---

## 2. PermGen → Metaspace (JEP 122)

### 2.1 What actually changed

JEP 122 *Remove the Permanent Generation* (<https://openjdk.org/jeps/122>). Goals:

> * Relocate class metadata, interned Strings, and class static variables from the permanent
>   generation to either the Java heap or native memory
> * Eliminate permanent generation code from the Hotspot JVM
> * Ensure application startup and footprint performance remains stable, with regression not
>   exceeding 1%

Description:

> Class metadata storage would shift from the permanent generation to native memory managed
> explicitly by Hotspot. Rather than a fixed size constraint, "allocation of new class
> meta-data would be limited by the amount of available native memory." Block-based allocation
> strategies would minimize fragmentation while enabling efficient cleanup when associated
> class loaders were no longer referenced.

The motivation stated in the JEP is convergence with JRockit, not GC correctness.

### 2.2 What it did *not* change

The JEP's non-goals are the important part for this reader:

> * Extending Class Data Sharing to application classes
> * **Reducing overall memory consumption for class metadata**
> * **Implementing asynchronous collection of class metadata**

And the trigger for reclamation is unchanged: *"Memory deallocation would occur when class
loaders became unreachable."*

**JEP 122 changed where class metadata lives and how it is sized. It did not change one thing
about when a class becomes collectable.** The rule before and after is JLS §12.7. Anyone
reading "we removed permgen" as "we made classes collectable" has it backwards: classes were
already collectable in permgen (a full GC swept it), and are still only collectable per-loader
now.

What it *did* buy: no fixed `MaxPermSize` cliff, and reclamation by returning whole native
chunks rather than by compacting a Java generation. **INFERENCE:** the practical effect for a
non-moving collector is the interesting one — Metaspace is non-moving and never compacted, and
it survives that by allocating per-container and freeing per-container, accepting internal
fragmentation instead of compaction. That is precisely the trade a non-moving Rexx arena would
have to make.

---

## 3. HotSpot: `ClassLoaderData` / `ClassLoaderDataGraph`

### 3.1 What a CLD is

> ClassLoaderData carries information related to a linkset (e.g., metaspace holding its klass
> definitions). The System Dictionary and related data structures (e.g., placeholder table,
> loader constraints table) as well as the runtime representation of classes only reference
> ClassLoaderData.
> — `src/hotspot/share/classfile/classLoaderData.cpp`, header comment

> A class loader represents a linkset. Conceptually, a linkset identifies the complete
> transitive closure of resolved links that a dynamic linker can produce.
> — `src/hotspot/share/classfile/classLoaderData.hpp`, header comment

Fields that matter (`classLoaderData.hpp:105-160, 231-320`):

| field | role |
|---|---|
| `_holder` (`WeakHandle`) | "The oop that determines lifetime of this class loader" |
| `_class_loader` (`OopHandle`) | the loader instance, itself stored in `_handles` |
| `_keep_alive_ref_count` | pinned liveness; see §5.1 |
| `_metaspace` (`ClassLoaderMetaspace`) | the arena all this loader's metadata comes from |
| `_klasses` | intrusive list of classes defined by this loader |
| `_handles` (`ChunkedHandleList`) | the strong-root set: loader oop, all mirrors, CP arrays, modules |
| `_dictionary`, `_packages`, `_modules` | the name→class tables — **inside** the container |
| `_deallocate_list` | metadata queued for deferred free |
| `_claim` | per-GC "already scanned" bits |
| `_unloading` | this CLD is going away |

The single most transferable structural decision: **the name→class tables are members of the
container, not global tables.** A global `name → class` map would make every class immortal.
HotSpot's per-loader `Dictionary` dies with its loader. (This is the trap flagged in §7.3.)

### 3.2 Where unloading happens in a GC cycle

**Serial full GC** — the closest analogue to a stop-the-world mark-sweep, so worth reading
literally. From `src/hotspot/share/gc/serial/serialFullGC.cpp`:

1. Mark phase roots include only *strongly reachable* loader data:
   `ClassLoaderDataGraph::always_strong_cld_do(&follow_cld_closure);` (line 493), with
   `CLDToOopClosure SerialFullGC::follow_cld_closure(&mark_and_push_closure,
   ClassLoaderData::_claim_stw_fullgc_mark);` (line 92). Everything else reaches a CLD only via
   `do_klass` from a live instance (§1.3).
2. Weak processing: `WeakProcessor::weak_oops_do(&is_alive, &do_nothing_cl);` — this is what
   clears `_holder` for dead loaders, so `is_alive()` starts answering `false`.
3. Class unloading, still inside the mark phase:

```cpp
{
  GCTraceTime(Debug, gc, phases) tm_m("Class Unloading", gc_timer());
  ClassUnloadingContext* ctx = ClassUnloadingContext::context();
  bool unloading_occurred;
  {
    CodeCache::UnlinkingScope scope(&is_alive);
    unloading_occurred = SystemDictionary::do_unloading(gc_timer());
    CodeCache::do_unloading(unloading_occurred);
  }
}
```

4. Compaction phases.
5. `ClassLoaderDataGraph::purge(true /* at_safepoint */);` — the actual free.

The **two-phase shape** (unlink, then much later free) is the load-bearing part.

`ClassLoaderDataGraph::do_unloading()` (`src/hotspot/share/classfile/classLoaderDataGraph.cpp`)
does only the unlink:

```cpp
bool ClassLoaderDataGraph::do_unloading() {
  assert_locked_or_safepoint(ClassLoaderDataGraph_lock);
  ClassLoaderData* prev = nullptr;
  uint loaders_processed = 0;
  uint loaders_removed = 0;
  for (ClassLoaderData* data = _head; data != nullptr; data = data->next()) {
    if (data->is_alive()) {
      prev = data;
      loaders_processed++;
    } else {
      // Found dead CLD.
      loaders_removed++;
      ClassUnloadingContext::context()->register_unloading_class_loader_data(data);
      // Move dead CLD to unloading list.
      if (prev != nullptr) { prev->unlink_next(); }
      else { assert(data == _head, "sanity check");
             // The GC might be walking this concurrently
             AtomicAccess::store(&_head, data->next()); }
    }
  }
  return loaders_removed != 0;
}
```

and `purge()` does the free:

```cpp
void ClassLoaderDataGraph::purge(bool at_safepoint) {
  ClassUnloadingContext::context()->purge_class_loader_data();
  bool classes_unloaded = ClassUnloadingContext::context()->has_unloaded_classes();
  Metaspace::purge(classes_unloaded);
  if (classes_unloaded) { set_metaspace_oom(false); }
  DependencyContext::purge_dependency_contexts();
  if (at_safepoint) {
    _safepoint_cleanup_needed = true;
    if (should_clean_metaspaces_and_reset()) { walk_metadata_and_clean_metaspaces(); }
  } else {
    MutexLocker ml(Service_lock, Mutex::_no_safepoint_check_flag);
    _safepoint_cleanup_needed = true;
    Service_lock->notify_all();
  }
}
```

Two clean-up passes deserve naming:

* `ClassLoaderDataGraph::clean_module_and_package_info()` — *"Walk a ModuleEntry's reads, and a
  PackageEntry's exports lists to determine if there are modules on those lists that are now
  dead"*. This purges dangling references **from healthy loaders into dead loaders' modules**.
  A container's death does not only free its own state; it invalidates *other live containers'*
  cross-references. That is the export-table problem in the Rexx design, and the JVM's answer is
  a scan of the survivors, not a weak reference.
* `ClassLoaderDataGraph::walk_metadata_and_clean_metaspaces()` — *"Mark metadata seen on the
  stack so we can delete unreferenced entries"*, driving `clean_deallocate_lists()`. A
  `Method*` belonging to a dead class may still be *executing*. The JVM does not free it in the
  same cycle; it stack-scans, defers, and frees later.

**G1**: class unloading was moved out of full GC into the concurrent cycle by JDK-8049421,
*"G1 Class Unloading after completing a concurrent mark cycle"*: previously *"G1 treats all
classes as live except for during full gcs"*; the change reuses the CMS infrastructure to
*"follow the object headers to find live classes"* during concurrent marking, and unloads *"at
the remark pause"*. Gated by `ClassUnloadingWithConcurrentMark` (JDK-8048269, *"Add flag to turn
off class unloading after G1 concurrent mark"*).

**ZGC**: JDK-8218905 release note, JDK 12 — *"The Z Garbage Collector now supports class
unloading. By unloading unused classes, data structures related to these classes can be freed,
lowering the overall footprint of the application."* — with *"zero impact on GC pause times"*;
disable with `-XX:-ClassUnloading`.

**Shenandoah**: concurrent class unloading in JDK 14, built on ZGC's JDK 13 work
(<https://developers.redhat.com/blog/2020/03/09/shenandoah-gc-in-jdk-14-part-2-concurrent-roots-and-class-unloading>).
The article names what still must happen in the pause: pre-evacuating non-weak roots, processing
weak roots to prevent resurrection, and arming nmethods.

### 3.3 What has to be true for it to be safe

| requirement | how HotSpot meets it | source |
|---|---|---|
| A class's mirror and statics die exactly with the class | mirror lives in the CLD handle area; statics live *in* the mirror | `klass.cpp:64`, `instanceMirrorKlass.hpp:35` |
| Compiled code holding raw `Klass*`/`Method*` must not outlive them | an nmethod is declared unloading if it holds a dead oop, and is unlinked *before* metadata is freed | `gcBehaviours.cpp:30-37`; `serialFullGC.cpp` ordering |
| Nothing follows metadata of a dying nmethod | explicit guard | `nmethod.cpp:885-889`: *"If the nmethod itself is dying, then it may point at dead metadata. Nobody should follow that metadata; it is strictly unsafe."* |
| Inline caches must not point at dead classes | `CodeCache` cleaning: *"Cleans caches in nmethods that point to either classes that are unloaded or nmethods that are unloaded."* | `nmethod.cpp:786-787` |
| Optimizations assuming a class hierarchy must be invalidated | `DependencyContext::purge_dependency_contexts()` in `purge()` | `classLoaderDataGraph.cpp` |
| A `Method*` on the stack must not be freed | stack scan + deferred deallocate list | `walk_metadata_and_clean_metaspaces` |
| Interned strings must not pin classes | JEP 122 moved interned Strings to the Java heap; they are weak-table entries cleared in `WeakProcessor::weak_oops_do` before class unloading. **INFERENCE** for the second half — I read the ordering in `serialFullGC.cpp`; I did not open `stringTable.cpp`. |
| Live loaders' cross-references to dead loaders' modules/packages must be scrubbed | `clean_module_and_package_info()` | `classLoaderDataGraph.cpp` |
| Symbols (names) refcounted by dead metadata must be released | **UNSOURCED:** I believe `Symbol` refcounts are decremented as metadata is freed and `SymbolTable` reclaims zero-refcount symbols, but I did not verify it in source. |

The nmethod entry-barrier machinery in ZGC/Shenandoah and this beauty in
`ClassLoaderData::demote_strong_roots()` exist **only** because marking runs concurrently:

```cpp
// classLoaderData.cpp:296-313
void ClassLoaderData::demote_strong_roots() {
  // The oop handle area contains strong roots that the GC traces from. We are about
  // to demote them to strong native oops that the GC does *not* trace from. [...]
  // Unless we invoke the right barriers, the GC might not notice that a strong root
  // has been pulled from the system, and is left unprocessed by the GC. There can be
  // several consequences:
  // 1. A concurrently marking snapshot-at-the-beginning GC might assume that the contents
  //    of all strong roots get processed by the GC in order to keep them alive. [...]
  // 2. A concurrently relocating GC might assume that after moving an object, a subsequent
  //    tracing from all roots can fix all the pointers in the system [...]
  // 3. A concurrent GC using colored pointers, might assume that tracing the object graph
  //    from roots results in all pointers getting some particular color [...]
```

All three consequences are conditioned on concurrency or movement. **A stop-the-world
non-moving mark-sweep can add and remove roots freely between cycles and needs none of this.**

---

## 4. OpenJ9: ROMClass / RAMClass, the shared cache, and how it unloads

### 4.1 The split

> In OpenJ9, Java classes are divided into two parts: a read-only part called a ROMClass, which
> contains all of the class's immutable data, and a RAMClass that contains mutable data, such as
> static class variables. A RAMClass points to data in its ROMClass, but these two are
> completely separated.
> — <https://dzone.com/articles/class-sharing-in-eclipse-openj9> (Part 1)

> if the JVM finds a ROMClass in the shared classes cache, it only needs to create the RAMClass
> in its local memory; the RAMClass then references the shared ROMClass. It is quite safe for a
> ROMClass to be shared between JVMs and also between RAMClasses in the same JVM.
> — ibid.

The shared classes cache is a memory-mapped (persistent or non-persistent) region storing
ROMClasses plus AOT code and JIT hints
(<https://www.ibm.com/docs/en/SSYKE2_8.0.0/openj9/shrc/index.html>), enabled by default since
OpenJ9 0.17 (<https://blog.openj9.org/2019/10/15/openj9-class-sharing-is-enabled-by-default/>).

### 4.2 How close is this to "built-ins static, user classes collected"?

**Not close, and the brief's framing should be corrected.** The ROM/RAM split is a *sharing and
footprint* mechanism, not a liveness mechanism. Both halves are unloaded, and both are unloaded
at exactly the same granularity as HotSpot: per class loader. Evidence is in the unloader itself
(`runtime/gc_base/ClassLoaderManager.cpp`):

* `identifyClassLoadersToUnload` walks all class loaders with `GC_ClassLoaderIterator` and marks
  one dead when `((NULL != classLoaderObject) && (!markMap->isBitSet(classLoaderObject)))` —
  i.e. **liveness of a loader is the mark bit of the loader's Java object**, the same rule as
  HotSpot's `_holder.peek()`.
* `cleanUpClassLoadersStart` walks the dead loaders' segments via `addDyingClassesToList`,
  setting `J9AccClassDying` on each class, and — a nice defensive touch — *"For all dying
  classes we poison the classObject field to J9_INVALID_OBJECT to investigate the origin of a
  class object reference whose class has been unloaded."*
* Hooks fire so the JIT and VM can do their own invalidation before memory is released:
  `J9HOOK_VM_CLASSES_UNLOAD`, `J9HOOK_VM_ANON_CLASSES_UNLOAD`, `J9HOOK_VM_CLASS_LOADERS_UNLOAD`.
* `cleanUpSegmentsAlongClassLoaderLink` is where the split shows up **in the free path, not the
  decision path**: ROM class segments are freed immediately with `freeMemorySegment`, while RAM
  class segments are marked `MEMORY_TYPE_UNDEAD_CLASS` for deferred cleanup, and the code
  *"unset[s] the fact that this is a RAM CLASS since some code which walks the segment list is
  looking for still-valid ones."* That is the same unlink-then-free two-phase shape as HotSpot.
* `cleanUpSegmentsInAnonymousClassLoader` special-cases anonymous classes, which are *"expected
  to be allocated one per segment"* — OpenJ9's version of the container-of-one (§5.3).

So the answer to "how close is it": the split says *which bytes are immutable*, not *which
classes may die*. **A ROMClass in the shared cache is not immortal because it is immutable; it
is immortal because it lives in a file mapped by many processes and is refcounted/never freed by
the individual JVM.** UNSOURCED: I did not verify the shared cache's own reclamation policy for
stale ROMClasses.

### 4.3 When OpenJ9 unloads

`-Xclassgc` is the default; class unloading *"occurs only on class loader changes"*, and
`-Xalwaysclassgc` forces it every global collection
(<https://eclipse.dev/openj9/docs/xclassgc/>, <https://eclipse.dev/openj9/docs/xalwaysclassgc/>).
The docs warn that `-Xnoclassgc` *"is not recommended because unlimited native memory growth can
occur"*.

That heuristic is worth stealing: **do not pay the class-liveness edge on every collection; pay
it only when the population of containers has changed since the last time you paid.** In
HotSpot the equivalent is the `do_metadata()` opt-out on the marking closure (§1.3).

UNSOURCED: a search summary reported that OpenJ9's `classunload` GC operation is single-threaded
even under parallel GC policies; I did not open the IBM verbose-GC page that would confirm it.

---

## 5. Immortality made explicit

Three different mechanisms, at three different levels. All three are relevant, because the Rexx
decision keeps built-in classes static.

### 5.1 A pinned refcount, not a region

This is the important one, and it is three tokens of code:

```cpp
// classLoaderData.cpp:141-149
ClassLoaderData::ClassLoaderData(Handle h_class_loader, bool has_class_mirror_holder) :
  ...
  // A non-strong hidden class loader data doesn't have anything to keep
  // it from being unloaded during parsing of the non-strong hidden class.
  // The null-class-loader should always be kept alive.
  _keep_alive_ref_count((has_class_mirror_holder || h_class_loader.is_null()) ? 1 : 0),
```

The bootstrap loader is `h_class_loader.is_null()`, so its CLD is born with
`_keep_alive_ref_count == 1` and `is_alive()` (§1.3) answers `true` forever. And:

```cpp
bool is_permanent_class_loader_data() const {
  return is_builtin_class_loader_data() && !has_class_mirror_holder();
}
```

**After JEP 122 there is no permgen, and built-in classes are still immortal.** The immortality
is a per-container counter, not a memory region. This is a direct, cheap answer to the Rexx
requirement "built-in classes may keep a static lifetime; no permgen" — the two are not in
tension at all, and the JVM is the existence proof.

### 5.2 Shared (CDS/AOT) metadata is never freed

```
// classLoaderData.cpp, add_to_deallocate_list()
// Metadata in shared region isn't deleted.
if (!m->in_aot_cache())
```

Metadata mapped from the CDS/AOT archive never enters the deallocate list, so `purge()` cannot
free it. (`in_aot_cache()` is the current name; it was `is_shared()` in older releases.)
AppCDS itself: JEP 310 (<https://openjdk.org/jeps/310>) — *"AppCDS works by memory-mapping the
contents of the archive at a fixed address."*

### 5.3 Archived heap objects: never collected, never moved, still traced

From the OpenJDK HotSpot wiki, *CDS Archived Heap Improvements*
(<https://wiki.openjdk.org/display/HotSpot/CDS+Archived+Heap+Improvements>):

* **Closed region** — archived Strings and their char arrays; objects here *"can point to only
  other objects in this region"* and are *"alive forever. They are never collected."*
* **Open region** — everything else, including class mirrors and the module graph; these *"can
  reference any objects (including those that are outside of the Archived Heap)"* and are also
  never collected.
* Archived objects are never relocated; they stay pinned at their mapped addresses.
* The wiki records a real bug from this design: archiving the full module graph in JDK 16
  produced archived arrays that were later updated to point at *non*-archived objects; when the
  GC moved those objects, the archived arrays held stale pointers and the JVM crashed
  (JDK-8253081, *"G1 fails on stale objects in archived module graph in Open Archive regions"*).
  The fix was an explicit **Root List** of archived objects that the GC consults so it can
  *"reliably determine whether an archived object is alive or dead"*.

**This is the single most directly applicable cautionary tale in the whole report.** The
distinction between the two regions is exactly the question the Rexx design has to answer about
static built-in classes:

* If a static built-in class can **only** point at other static built-in things, it is a closed
  region: the collector may skip it entirely — not mark it, not sweep it, not trace out of it.
* If a static built-in class can be made to point at a collectable object — a user-defined
  method installed on a built-in class, a class variable holding a user object, a subclass list
  containing a user class — it is an open region: it must be **traced as a root on every
  cycle**, even though it is never itself collected. Skipping it is a use-after-free, and the
  JVM shipped exactly that bug.

The G1 restriction that *"G1 cannot conclude that an archived object is garbage just because
it's unreachable"* is the same rule from the sweep side: **immortal objects must be excluded
from the sweep by identity, not by reachability.**

---

## 6. What goes wrong

### 6.1 The classloader leak, and why it is structural

Amplification (§1.4) means one strong reference from a long-lived structure to *anything* in a
container keeps the whole container alive. The canonical instance is the ThreadLocal pattern
(<https://java.jiderhamn.se/2012/01/29/classloader-leaks-iv-threadlocal-dangers-and-why-threadglobal-may-have-been-a-more-appropriate-name/>):

* Chain: pooled container thread → `Thread.threadLocals` (`ThreadLocalMap`) → map entry *value*
  → an object of a webapp class → its class → its classloader → all webapp classes and statics.
* `ThreadLocalMap` keys are `WeakReference`s to the `ThreadLocal` instances, **but the values are
  strong**, and stale entries are cleaned only opportunistically — the article notes that,
  unlike `WeakHashMap`, there is no `ReferenceQueue`, so *"stale entries are guaranteed to be
  removed only when the table starts running out of space."*
* Result after a few redeploys: `OutOfMemoryError: Metaspace` (formerly `PermGen space`).
  Live example: micrometer issue #7184, *"ClassLoader leak (Metaspace OOM) caused by static
  ThreadLocals in DoubleFormat"*
  (<https://github.com/micrometer-metrics/micrometer/issues/7184>).

Tomcat ships a whole listener whose job is to pre-initialize JRE singletons so they do not
capture the webapp loader as their context classloader: `JreMemoryLeakPreventionListener`, whose
documentation says it *"provides work-arounds for known places where the Java Runtime
environment uses the context class loader to load a singleton as this will cause a memory leak
if a web application class loader happens to be the context class loader at the time"*
— covering `DriverManager`, `SeedGenerator`, `sun.awt.AppContext`, URL/JAR caching, and a
user-extensible `classesToInitialize` list
(<https://tomcat.apache.org/tomcat-10.1-doc/config/listeners.html>).

**The transferable lesson is not "beware ThreadLocal". It is: any global, long-lived,
strongly-referencing memo table defeats container liveness completely and silently.** A
`::REQUIRES` memo table, a global "all loaded packages" registry, a global method cache keyed by
class, or an inline-cache array in compiled IR that holds a class handle strongly — each one of
these, on its own, makes the entire design a no-op while every test still passes.

### 6.2 Where per-loader granularity was documented as the wrong unit

JEP 371 *Hidden Classes* (<https://openjdk.org/jeps/371>) is the JVM admitting it:

> Language runtimes on the JVM generate classes dynamically for features like lambda expressions
> and dynamic proxies. Such generated classes should function as implementation details — not
> discoverable or linkable by other code. Traditional APIs (`ClassLoader::defineClass`,
> `Lookup::defineClass`) create "visible" classes that remain discoverable throughout their
> loader's lifetime, which is inefficient.

and:

> "A normal class cannot be unloaded unless its defining loader can be reclaimed by the garbage
> collector" [...] By default, [hidden classes] use a **weak** relationship with their notional
> defining loader — unloadable when all instances and references disappear, *regardless of
> loader reachability*. [...] the `STRONG` option creates hidden classes with traditional strong
> bindings to their loader.

The predecessor was `sun.misc.Unsafe::defineAnonymousClass`; the case that forced it was
`invokedynamic`/lambda, where one call site mints one class that must die with the call site,
not with the application classloader.

**And here is the mechanism, which is the punchline of this whole report:** HotSpot did *not*
add per-class liveness. It gave each non-strong hidden class **its own ClassLoaderData**, whose
holder is the class's own mirror rather than a loader object:

```cpp
// classLoaderData.cpp:141-149, 165-168 (paraphrased flow)
ClassLoaderData::ClassLoaderData(Handle h_class_loader, bool has_class_mirror_holder)
  ...
  if (!has_class_mirror_holder) {
    // The holder is initialized later for non-strong hidden classes,
    // and before calling anything that call class_loader().
    initialize_holder(h_class_loader);
  }
```

with `bool has_class_mirror_holder() const { return _has_class_mirror_holder; }`
(`classLoaderData.hpp:261`). During parsing, before the mirror exists to hold it, the CLD is
pinned by `_keep_alive_ref_count` — the same counter that makes the boot loader immortal —
and released by `dec_keep_alive_ref_count()`, which calls `demote_strong_roots()` on the way
down (`classLoaderData.cpp:349-360`).

OpenJ9 did the structurally identical thing: anonymous classes get their own segment, one per
class (`cleanUpSegmentsInAnonymousClassLoader`: *"Anonymous classes expected to be allocated one
per segment"*).

**Both VMs solved "per-class granularity" by making the container a singleton, not by adding a
second liveness rule.** That is one mechanism, two granularities, and it is the design I would
copy.

### 6.3 What it costs

* **Pause time.** The remark pause is where G1 pays: JDK-8056240, *"Investigate increased GC
  remark time after class unloading changes in CRM Fuse"*, and the still-open JDK-8326092,
  *"Pause Remark sometimes has extremely long pause times on class unloading"*. Class unloading
  is not free even when amortised into a concurrent cycle.
* **Complexity.** ZGC and Shenandoah needed nmethod entry barriers, concurrent root processing,
  and `demote_strong_roots` barriers to get this off the pause. Roughly: two collectors, several
  releases, one new barrier kind.
* **Space.** Per-container arenas trade internal fragmentation for cheap bulk free (JEP 122's
  "block-based allocation ... minimize fragmentation" is a mitigation, not an elimination).
* **Latency of reclamation.** Even after the decision, freeing is deferred: stack scans,
  service-thread-triggered extra safepoints (`purge(at_safepoint=false)` path above), and
  OpenJ9's `MEMORY_TYPE_UNDEAD_CLASS` segments.

---

## 7. Transfer to a precise, non-moving, non-generational, STW mark-sweep with no barriers and no JIT

### 7.1 Table

| mechanism | why the JVM has it | transfers? |
|---|---|---|
| Container (loader) holds classes strongly; container held by one **weak** external edge | soundness: "can never be asked by name again" | **Yes — this is the whole idea.** It is a graph-shape decision, independent of collector style |
| Class mirror is an **ordinary heap object**, and the collector traces mirrors, not metadata | lets normal tracing decide class liveness | **Yes, and it is a prerequisite.** Blocker #1 in the current Rexx design (tagged handles ≥ `1<<31` that `resolve` returns `None` for) is precisely the absence of this |
| Statics stored **inside** the mirror (`InstanceMirrorKlass`) | statics need no special GC handling at all | **Yes.** Put the class-variable pool *in* the class object's body instead of registering it as a named global root. This alone deletes blocker #2 |
| Name→class tables live **inside** the container, never globally | a global table would pin everything | **Yes, and it is the highest-risk item** — see §7.3 |
| `do_metadata()` opt-out: skip the class edge on non-unloading cycles | avoid paying for class liveness every GC | **Yes.** Cheap and worth doing: a `bool trace_classes` on the mark loop, flipped when the package population changed (OpenJ9's `-Xclassgc` heuristic) |
| Two-phase unlink-then-free, with a stack scan for in-use methods | a `Method*` may be executing | **Yes, conditionally.** If Rexx method bodies are arena objects reachable from activation frames (already roots), tracing gets this for free. If they are native structures hanging off the class, you need HotSpot's explicit "seen on the stack" pass |
| Scrub live containers' references into dead containers (`clean_module_and_package_info`) | exports/merged tables outlive their target | **Yes.** The Rexx public/merged export tables are the same shape. Either scan survivors after unloading, or make those entries weak — the JVM chose the scan |
| Per-container arena, freed wholesale (`Metaspace::purge`) | non-moving reclamation without compaction | **Optional.** Attractive for a non-moving sweep, but only if class-owned data is *not* ordinary arena objects. If classes are ordinary objects, plain sweep already works and this is premature |
| Immortality as a **pinned counter** on the container (`_keep_alive_ref_count`) | built-ins never die, no separate region needed | **Yes.** One flag per built-in class. Directly satisfies "static built-ins, no permgen" |
| Archived-heap closed vs open regions | never-collected objects that may or may not point out | **Yes, as a rule to obey:** never-collected ≠ never-traced. See §7.2 |
| `demote_strong_roots()` SATB/relocation/colour barriers | concurrent marking, moving GC, coloured pointers | **No.** All three stated consequences are conditioned on concurrency or movement |
| nmethod entry barriers, `is_unloading()`, inline-cache cleaning, `DependencyContext` | JIT-compiled code holds raw metadata pointers and speculative assumptions | **No** as written. **But the shape returns** the moment the Rexx IR caches a class handle in a compiled instruction — then you have an nmethod problem in miniature. See §7.4 |
| CLD `_claim` bit-set | de-duplicate scanning across parallel GC workers and incremental phases | **No.** A single-threaded mark's ordinary mark bit is exactly this |
| Metaspace/native-memory split | `Klass` is not an oop; GC cannot move or trace it | **No.** Only exists because class metadata is outside the object model. Do not import it |
| ROMClass/RAMClass split | cross-process sharing and startup, via a mapped file | **No** for liveness. **Maybe** for footprint: if built-ins are static, their immutable half can be `&'static` data in the binary and only the mutable half needs to be an arena object |
| Concurrent/remark-pause placement of unloading (G1/ZGC/Shenandoah) | pause-time engineering | **No.** A STW sweep does it in the sweep |

### 7.2 The one rule that must be enforced in code, not prose

Static built-in classes must be **excluded from the sweep by identity** (a flag, not
reachability), and **either** proven unable to reference any collectable object **or** traced as
roots on every cycle.

The JVM has both halves of this and got it wrong once (JDK-8253081, §5.3) when an "immortal"
object was updated to point at a collectable one. For Rexx the test is concrete:

* Can a user program install a method on a built-in class (`.String~setMethod`, `::EXTEND`, a
  metaclass change)?
* Can a built-in class hold a class variable whose value is a user object?
* Does a built-in class keep a subclass list that a user subclass is appended to?

If any is yes, static built-in classes are "open regions" and **must be roots**. Assert this at
the type level if you can — a `Static` class whose body can only hold `Static` handles — rather
than documenting it. (This is the same shape as the archive's closed-region invariant.)

### 7.3 The failure that will actually happen

Not a collector bug. A table.

The design will be implemented, will pass its gates, and will collect nothing, because some
long-lived structure holds packages or classes strongly:

* the `::REQUIRES` / "already loaded" package memo (**must** be weak, or keyed so entries die
  with the package);
* a global class registry surviving the removal of the named-global-root-per-class;
* a method-lookup cache keyed by class;
* an inline cache in compiled IR;
* the environment/`.local` directory, if class objects land there.

HotSpot's structural defence is that there *is* no global class table — the dictionary is a
member of the container (§3.1). **Do not test this by "run a program and watch memory". Test it
by asserting that a specific package object is unreachable after a specific point**, i.e. assert
the side effect (the sweep freed it), not that a collection returned successfully.

### 7.4 When the JIT mechanisms come back

The report says "no JIT", and that is true today. But the transferable trigger is narrower than
"JIT": it is **any raw, untraced pointer to class-owned data cached outside the class**. If the
Rexx IR memoises a resolved method or class handle in an instruction operand, then:

* that operand is either a traced root (and pins the class — the amplification problem), or
* it is untraced (and is a use-after-free the moment the class is swept).

HotSpot's answer is the third option — declare the *holder* dead when it holds a dead referent
(`IsUnloadingBehaviour::is_unloading` = `has_dead_oop(nm) || nm->is_cold()`,
`gcBehaviours.cpp:30-37`) and unlink it before the metadata is freed. Worth knowing that this
option exists before the first inline cache is written.

---

## 8. Is "liveness delegated to a container" the right unit for a language with no classloaders?

Rexx's plausible analogue is the **package** — a source file with `::CLASS` directives.

### 8.1 The case FOR the container (package) as the unit

1. **The soundness argument is about re-derivation, not reachability, and it applies.** JLS
   §12.7's reasoning is: unloading must be transparent; reloading is not transparent (statics
   lost, initializers re-run with side effects, identity hash changes); therefore do not unload
   anything that could be asked for by name again. In Rexx, installing a `::CLASS` directive
   **runs Rexx code** (`::METHOD` bodies are attached, `::ATTRIBUTE` generates methods, and
   class-scope initialisation happens at install time). So collecting a class that a live
   package can still resolve by name, and re-creating it on the next reference, would re-run
   user-visible code and change class identity. That is exactly the JVM's forbidden case, and
   the package is exactly the thing that answers name queries. **This argument alone decides it
   for directive-defined classes.**
2. **It is cycle-proof by construction.** Superclass, metaclass, subclass list, method
   dictionary, annotation table, class-variable pool all point at each other. Inside one
   container that is free. Per-class liveness would require deciding, for each of those edges,
   whether it is strong or weak — and every weak edge is a resurrection bug waiting to be
   written.
3. **It shrinks the root set immediately.** Today: one named global root per class. Per-package:
   one holder per package. That is a change to the registry, not to the mark loop.
4. **It is one decision per container per GC**, not one per class — a linear walk of a short
   list (HotSpot's `do_unloading` is exactly that).
5. **It matches the unit of source, reload and error reporting**, so the thing a user reasons
   about ("my program stopped needing that file") is the thing the collector reasons about.

### 8.2 The case AGAINST — for per-class liveness

1. **Amplification is real and documented.** One live instance of one class pins every class in
   the package and every class variable of every one of them (§1.4, §6.1). For a Rexx program
   with a few large `::REQUIRES`'d libraries, that is most of the heap's class state.
2. **Not every Rexx class has a package.** A class made at runtime — `.Class~new`, a mixin built
   programmatically, a class defined inside `INTERPRET` — has no directive and no name anyone
   can re-resolve. For those the re-derivation argument in §8.1(1) is simply false: nothing can
   ask for them by name, so nothing prevents collecting them individually. This is precisely the
   case that forced JEP 371 on the JVM after fifteen years.
3. **The JVM's own trajectory is evidence against pure per-container.** It shipped per-loader in
   1.0, hit the dynamic-class-generation wall with `invokedynamic`, and bolted on hidden classes
   in Java 15. Starting from a blank sheet, you get to not repeat that.
4. **With tracing, per-class precision is nearly free.** The JVM's per-loader choice is partly
   historical and partly about the metadata being *outside* the object model. If Rexx class
   objects are ordinary arena objects, the collector already traces them; per-class precision
   costs a mark bit that it is already paying.

### 8.3 Recommendation

Take the container rule **and** the container-of-one, which is one mechanism:

* Every class has a **holder**: one weak edge that decides its life.
* A class defined by a `::CLASS` directive has its **package** as its holder. All such classes
  in a package share a holder; the package's own liveness comes from outside (a variable holding
  one of its classes or instances, an activation executing its code, an active `::REQUIRES`
  chain).
* A class created at runtime with no directive has **itself** as its holder — a container of
  one. This is HotSpot's `_has_class_mirror_holder` verbatim.
* A **built-in** class is born with the holder pinned (`_keep_alive_ref_count = 1`). No permgen,
  no second code path, and §7.2's tracing rule applies to it.

Three cases, one predicate. And it means the answer to "collecting data that is static from
within and not from the outside" is structural rather than clever: **make every internal
reference strong and non-root; make exactly one external reference weak; make the collector's
only question about a class "is my holder still marked?"**

The two things that will decide whether it works are not in the collector:

* whether the package memo table (`::REQUIRES`) and any global class registry are weak (§7.3);
* whether static built-in classes can point at collectable objects, and are therefore roots
  rather than skipped (§7.2).

---

## 9. What I could not establish

* **`Symbol`/name-table reclamation in HotSpot.** I asserted nothing about it beyond the
  `UNSOURCED:` line in §3.3; I did not open `symbolTable.cpp`.
* **StringTable weakness.** I sourced that JEP 122 moved interned Strings to the Java heap. That
  they are held weakly and cleared by `WeakProcessor` before class unloading is an **inference**
  from the call ordering in `serialFullGC.cpp`, not from a document.
* **How an nmethod's embedded `Klass*` keeps its class alive.** I sourced the *negative* half
  (an nmethod holding a dead oop is declared unloading, and dying nmethods' metadata must not be
  followed). I could not find, in the time available, the code that guarantees a *live* nmethod's
  embedded metadata is kept alive — my belief is that the JIT also records the class's mirror in
  the nmethod's oop section so that `has_dead_oop` sees it, but I did not confirm it and grepping
  `ciEnv.cpp` for `mirror` was inconclusive. **UNSOURCED.**
* **OpenJ9 shared-cache reclamation.** Whether and when stale ROMClasses are evicted from the
  shared classes cache. Not established.
* **OpenJ9 `classunload` being single-threaded.** Reported by a search summary attributing it to
  IBM verbose-GC documentation; I did not open the page. **UNSOURCED.**
* **A documented statement that application-classpath classes are effectively never unloaded.**
  This follows from JLS §12.7 plus a permanently reachable system loader, but I found no source
  stating it, so it is marked as an inference in §1.4.
* **Quantified cost of class unloading.** I have the existence of pause-time regressions
  (JDK-8056240, JDK-8326092) but no numbers.
* **`inside.java`'s ZGC concurrent class unloading article** (Österlund, 2019-02-04) would not
  render for me; the ZGC facts here come from the JDK-8218905 release note and the Red Hat
  Shenandoah article instead.

---

## Sources

* JLS SE21 §12.7, §12.2 — <https://docs.oracle.com/javase/specs/jls/se21/html/jls-12.html>
* JEP 122 Remove the Permanent Generation — <https://openjdk.org/jeps/122>
* JEP 310 Application Class-Data Sharing — <https://openjdk.org/jeps/310>
* JEP 371 Hidden Classes — <https://openjdk.org/jeps/371>
* JDK-8049421 G1 Class Unloading after completing a concurrent mark cycle — <https://bugs.java.com/bugdatabase/view_bug?bug_id=8049421>
* JDK-8048269 Add flag to turn off class unloading after G1 concurrent mark (referenced from the above)
* JDK-8056240 Investigate increased GC remark time after class unloading changes — <https://bugs.java.com/bugdatabase/view_bug.do?bug_id=8056240>
* JDK-8326092 Pause Remark sometimes has extremely long pause times on class unloading — <https://bugs.java.com/bugdatabase/view_bug?bug_id=8326092>
* JDK-8218905 Release Note: ZGC: Concurrent Class Unloading — <https://bugs.openjdk.org/browse/JDK-8218905>
* Shenandoah GC in JDK 14, Part 2: Concurrent roots and class unloading — <https://developers.redhat.com/blog/2020/03/09/shenandoah-gc-in-jdk-14-part-2-concurrent-roots-and-class-unloading>
* CDS Archived Heap Improvements (OpenJDK HotSpot wiki) — <https://wiki.openjdk.org/display/HotSpot/CDS+Archived+Heap+Improvements>
* OpenJDK sources (`openjdk/jdk` master, fetched 2026-09-09):
  `src/hotspot/share/classfile/classLoaderData.{hpp,cpp}`,
  `src/hotspot/share/classfile/classLoaderDataGraph.cpp`,
  `src/hotspot/share/memory/iterator.{hpp,inline.hpp}`,
  `src/hotspot/share/oops/klass.cpp`,
  `src/hotspot/share/oops/instanceKlass.inline.hpp`,
  `src/hotspot/share/oops/instanceMirrorKlass.hpp`,
  `src/hotspot/share/gc/serial/serialFullGC.{hpp,cpp}`,
  `src/hotspot/share/gc/shared/gcBehaviours.cpp`,
  `src/hotspot/share/code/nmethod.cpp`
* OpenJ9 `runtime/gc_base/ClassLoaderManager.cpp` — <https://github.com/eclipse-openj9/openj9/blob/master/runtime/gc_base/ClassLoaderManager.cpp>
* OpenJ9 `-Xclassgc` / `-Xalwaysclassgc` — <https://eclipse.dev/openj9/docs/xclassgc/>, <https://eclipse.dev/openj9/docs/xalwaysclassgc/>
* Class Sharing in Eclipse OpenJ9 (Part 1) — <https://dzone.com/articles/class-sharing-in-eclipse-openj9>
* OpenJ9 class sharing enabled by default — <https://blog.openj9.org/2019/10/15/openj9-class-sharing-is-enabled-by-default/>
* IBM: Introduction to class data sharing — <https://www.ibm.com/docs/en/SSYKE2_8.0.0/openj9/shrc/index.html>
* Jiderhamn, Classloader leaks IV — ThreadLocal dangers — <https://java.jiderhamn.se/2012/01/29/classloader-leaks-iv-threadlocal-dangers-and-why-threadglobal-may-have-been-a-more-appropriate-name/>
* Jiderhamn, Classloader leaks V — Common mistakes and known offenders — <https://java.jiderhamn.se/2012/02/26/classloader-leaks-v-common-mistakes-and-known-offenders/>
* Apache Tomcat 10.1 config listeners (JreMemoryLeakPreventionListener) — <https://tomcat.apache.org/tomcat-10.1-doc/config/listeners.html>
* micrometer #7184, ClassLoader leak (Metaspace OOM) caused by static ThreadLocals — <https://github.com/micrometer-metrics/micrometer/issues/7184>
