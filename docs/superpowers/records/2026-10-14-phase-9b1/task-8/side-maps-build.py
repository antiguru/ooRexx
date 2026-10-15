import sys
rows = [l.rstrip('\n').split('\t') for l in open(sys.argv[1]) if l.strip()]
NP = "not per-object"
ACT = (NP, "-", "the activity's execution state, popped with its frames or its activity")
C = {
 # Interp
 ("Interp","env"): (NP,"-","the interpreter's environment variables"),
 ("Interp","library_search"): (NP,"-","directories LD_LIBRARY_PATH named at start"),
 ("Interp","plans"): ("Program","BodyKey (ProgramId, directive)","3: prune_program_rows"),
 ("Interp","chunks"): ("Program","(BodyKey, ChunkTrace)","3: prune_program_rows, which releases the chunk's interned literals (Heap::release_immortal)"),
 ("Interp","rexx_class_cache"): (NP,"-","a .NAME lookup cache, cleared at every collection (invalidate_rexx_class_cache in collect_now)"),
 ("Interp","class_variables"): ("Class","class ObjRef","2: prune_freed_classes"),
 ("Interp","class_packages"): ("Class","class ObjRef","2: prune_freed_classes"),
 ("Interp","routine_objects"): ("Routine of a ::ROUTINE","InstalledRoutine (ProgramId, directive)","3: prune_program_rows"),
 ("Interp","library_routine_objects"): (NP,"-","one Routine per library procedure row, a global root for the run as the oracle keeps a library's routine objects; bounded by the rows"),
 ("Interp","internal_routine_objects"): (NP,"-","one slot per static INTERNAL_ROUTINES row"),
 ("Interp","annotations"): ("Class, member, package, routine","environment::Annotated","2 for Class/Member: prune_freed_classes; 3 for Package/Unattached/Routine: prune_program_rows; LibraryRoutine/InternalRoutine rows are one per static or library row; Compiled sites are never inserted (annotation_table)"),
 ("Interp","method_objects"): ("Method a class entry answers","(class ObjRef, side, name)","2: prune_freed_classes"),
 ("Interp","method_bodies"): ("Method identity","MethodId","3: prune_program_rows; 2 for an identity a Method object owns (ExecutableRecord::owns_installed): forget_method_identity from prune_dead_object_rows"),
 ("Interp","library_programs"): (NP,"-","the library bootstrap's programs, held for the run"),
 ("Interp","compiled_method_names"): ("Program","ProgramId","3: prune_program_rows"),
 ("Interp","executable_objects"): ("Method identity","MethodId, value a Method ObjRef","2: prune_dead_object_rows drops a row whose object died, forget_method_identity an owned identity's; 3: prune_program_rows"),
 ("Interp","executable_sources"): ("Method or Routine object","ObjRef","2: prune_dead_object_rows; 3: prune_program_rows"),
 ("Interp","stem_exposers"): ("Stem","stem ObjRef, values weak cells","2 by its cells: a row goes once every exposer it names is swept, checked when a list or the table doubles (Interp::record_exposer)"),
 ("Interp","method_flag_writes"): ("Method object","ObjRef","2: prune_dead_object_rows"),
 ("Interp","message_outcomes"): ("Message","ObjRef","2: prune_dead_object_rows"),
 ("Interp","started_messages"): ("Message","ObjRef","2: prune_dead_object_rows"),
 ("Interp","unnotified_messages"): ("Message","ObjRef","2: prune_dead_object_rows"),
 ("Interp","generated_methods"): ("Method identity","MethodId","3: prune_program_rows; 2 for an owned identity: forget_method_identity"),
 ("Interp","native_externals"): ("Method identity","MethodId","3: prune_program_rows; 2 for an owned identity: forget_method_identity"),
 ("Interp","guarded_externals"): ("Method identity","MethodId","3: prune_program_rows; 2 for an owned identity: forget_method_identity"),
 ("Interp","kept_strings"): ("any object a native call copied","ObjRef","2: collect_now's retain"),
 ("Interp","kept_names"): (NP,"-","one terminated copy per distinct message or routine name a native call asked for"),
 ("Interp","kept_holders"): (NP,"-","counts of native calls in flight, dropped as each call ends"),
 ("Interp","command_handlers"): (NP,"-","per ADDRESS environment name an extension registered"),
 ("Interp","library_externals"): ("Method identity","MethodId","3: prune_program_rows; 2 for an owned identity: forget_method_identity"),
 ("Interp","external_packages"): ("Method identity","MethodId","3: prune_program_rows; 2 for an owned identity: forget_method_identity"),
 ("Interp","library_codes"): (NP,"-","one row per library procedure spelling, as the oracle caches a code object per spelling (LibraryPackage::resolveMethod); its package is rooted for the run"),
 ("Interp","library_code_rows"): (NP,"-","the row of each library procedure spelling"),
 ("Interp","library_code_keys"): (NP,"-","the procedure of each library code row"),
 ("Interp","library_code_routines"): (NP,"-","the routine row of each library code row"),
 ("Interp","library_routine_codes"): ("Routine of a ::ROUTINE","InstalledRoutine","3: prune_program_rows"),
 ("Interp","rexx_routine_rows"): ("Routine of a ::ROUTINE","InstalledRoutine","3: prune_program_rows"),
 ("Interp","defined_library_codes"): ("Method identity","MethodId","2: the identity a loadExternalMethod answer owns, forget_method_identity"),
 ("Interp","package_routines"): (NP,"-","PackageManager::packageRoutines, by routine name"),
 ("Interp","package_routine_codes"): (NP,"-","the code row of each package_routines slot"),
 ("Interp","special_methods"): ("Method identity","MethodId index","3: prune_program_rows; 2 for an owned identity: forget_method_identity. The vector's length follows the highest identity with a row (reported)"),
 ("Interp","uninit_ready"): (NP,"-","the queue of objects awaiting UNINIT, drained by run_ready_uninits"),
 ("Interp","required_paths"): ("Program","ProgramId","3: prune_program_rows"),
 ("Interp","required_packages"): ("Program","::REQUIRES name, value ProgramId","3: prune_program_rows, and collect_now drops a row whose package was swept"),
 ("Interp","untranslated"): ("Program","ProgramId","3: prune_program_rows"),
 # Programs
 ("Programs","slots"): ("Program","ProgramId slot","3: Programs::free_unheld frees a slot whose package was swept; the slot is reused under a new generation"),
 ("Programs","packages"): ("Program","ProgramId slot","3: freed with the slot (Programs::free_unheld)"),
 ("Programs","free"): (NP,"-","the free list of program slots"),
 ("Programs","issued"): (NP,"-","test-only record of issued ids"),
 ("Programs","freed"): (NP,"-","test-only record of freed ids"),
 ("ObjectModel","natives"): ("Method identity","MethodId index","bootstrap rows for the primitive classes; 2 for an owned copy's row: forget_method_identity. The vector's length follows the highest identity with a row (reported)"),
 ("ObjectModel","store_holders"): (NP,"-","the bootstrap's store-holder classes"),
 ("Libraries","held"): (NP,"-","PackageManager::packages: every native library a name loaded"),
 ("PackageTable","names"): (NP,"-","a loaded library's routine name table"),
 ("PackageTable","next"): (NP,"-","a loaded library's routine name table"),
 ("Activities","idle"): (NP,"-","one slot per activity, reused through free"),
 ("Activities","free"): (NP,"-","the free list of activity slots"),
 ("Activities","owners"): ACT,
 ("Activities","set_aside"): ACT,
 ("Activities","timers"): (NP,"-","one per running Alarm or Ticker timer, removed when it ends"),
 ("Activities","message_ids"): ("Message","ObjRef","2: Activities::prune_messages, and removed at completion"),
 ("Activities","senders"): ("Message","ObjRef","2: Activities::prune_messages"),
 ("Activities","waiters"): ("Message","MessageId","removed when the message completes; a waiting activity holds the message"),
 ("Activities","retired"): (NP,"-","the thread context of each ended activity that made a native call, kept until termination because an extension may hold it"),
 ("Activities","late_failures"): (NP,"-","failures of started activities awaiting report"),
 ("Activities","abandoned"): (NP,"-","native frames of calls whose waiter left, freed as each call ends"),
 ("Activities","abandoned_blocks"): (NP,"-","blocking calls whose waiter left, removed as each completes"),
 ("GuardTable","locks"): ("object and scope","GuardKey (ObjRef, scope)","removed at the last release (GuardTable::release); a holder runs a method on the object"),
 ("GuardTable","waiting"): (NP,"-","per waiting activity"),
 ("GuardTable","pools"): ("object and scope","GuardKey (ObjRef, scope)","2: GuardTable::prune_pools"),
 ("GuardTable","watches"): ("object","ObjRef","2: GuardTable::prune_pools"),
 ("Semaphores","table"): ("MutexSemaphore or EventSemaphore","SemaphoreKey","2 for an object key: Semaphores::prune; a Sys*Sem handle is the program's until it closes it"),
 ("Semaphores","parked"): (NP,"-","per parked activity"),
 ("Semaphores","held"): (NP,"-","per holding activity"),
 ("Semaphores","answers"): (NP,"-","per resumed activity"),
}
for f in ["routines","public_routines","merged_public_routines","classes","public_classes","merged_public_classes","namespaces","imports","constants"]:
    C[("PackageData",f)] = ("Package","-","1: a field of the package object's body")
C.update({
 ("Heap","slots"): (NP,"-","the arena itself"),
 ("Heap","marks"): (NP,"-","the collector's mark bits"),
 ("Heap","immortal"): (NP,"-","roots: the bootstrap's classes and the literals live programs' chunks interned; a chunk's literals are released with it (Heap::release_immortal)"),
 ("Heap","uninit"): ("object with UNINIT","ObjRef","2: dropped by the collection that finds the object gone or cleared"),
 ("Heap","names"): ("any object without a name field","ObjRef","2: pruned in Heap::collect's sweep"),
 ("RootSet","globals"): (NP,"-","named global roots; an annotation table's root goes with its program (unroot_annotation_table from prune_program_rows)"),
 ("RootSet","cells"): ("variable a >name reference was taken to","cell index","none: never freed, and each cell's value is a root (reported)"),
 ("ActivityRoots","temps"): ACT, ("ActivityRoots","slots"): ACT, ("ActivityRoots","aliases"): ACT,
 ("ActivityRoots","segments"): ACT, ("ActivityRoots","parked"): (NP,"-","values an off-stack activation owns, released with it and reused through parked_free"),
 ("ActivityRoots","parked_free"): (NP,"-","the free list of parked"),
 ("ClassRegistry","names"): ("Class","class ObjRef","2: ClassRegistry::expunge"),
 ("ClassRegistry","default_names"): ("Class","class ObjRef","2: ClassRegistry::expunge"),
 ("ClassRegistry","object_names"): ("Class","class ObjRef","2: ClassRegistry::expunge"),
 ("ClassRegistry","by_name"): ("Class","name, value class ObjRef","2: ClassRegistry::expunge"),
 ("ClassRegistry","by_system_name"): ("Class","name, value class ObjRef","2: ClassRegistry::expunge"),
 ("ClassRegistry","private_native_methods"): (NP,"-","Setup.cpp's private native methods"),
 ("ClassDef","superclasses"): ("Class","-","1: a field of the class's graph entry"),
 ("ClassDef","subclasses"): ("Class","-","1: a field of the class's graph entry, scrubbed of swept classes in ClassGraph::expunge"),
 ("ClassDef","retired_behaviours"): ("Class","-","1: a field of the class's graph entry; its behaviours are emptied and freed in ClassGraph::expunge. A live class keeps every behaviour define retired (reported)"),
 ("ClassGraph","classes"): ("Class","class ObjRef","2: ClassGraph::expunge"),
 ("ClassGraph","behaviours"): ("Class","BehaviourHandle index","2: ClassGraph::expunge empties a swept class's behaviours, and free_behaviours hands their slots out again"),
 ("ClassGraph","free_behaviours"): (NP,"-","the free list of behaviour slots"),
 ("ClassGraph","uninit_classes"): ("Class","class ObjRef","2: ClassGraph::expunge"),
 ("MethodDict","entries"): ("Behaviour","-","1: a field of the behaviour or class dictionary"),
 ("MethodDict","scope_list"): ("Behaviour","-","1: a field of the behaviour"),
 ("MethodDict","scope_orders"): ("Behaviour","-","1: a field of the behaviour"),
 ("Table","entries"): (NP,"-","the handles an extension holds: a native call's locals, gone with its frame, or global references until the extension releases them"),
 ("Released","handles"): ("object a released handle named","handle","2 by limit: Table::release drops the handles whose object is gone when the table doubles"),
})
for f in ["value_buffer","suspended","spare_activations","spare_exposed","native_handles","native_spares","internal_levels","flat_loops","loop_objects","frames","call_tails","parked_calls","parked_levels","native_tails","flat_spares","failure_sites","failure_frames","fragments","requires_installing","failed_sends","guard_waits"]:
    C[("Activity",f)] = ACT
out=["site\tstruct\tfield\ttype\tdescribes\tkey\tstate and pruning site"]
missing=[]
for site,s,f,t in rows:
    k=(s,f)
    if k not in C: missing.append(k); continue
    d,key,st=C[k]
    if d==NP: d,key,st="-","-",NP+": "+st
    out.append("\t".join([site,s,f,t,d,key,st]))
extra=set(C)-{(s,f) for _,s,f,_ in rows}
if missing or extra:
    print("missing",missing,"extra",extra); sys.exit(1)
open(sys.argv[2],"w").write("\n".join(out)+"\n")
print(len(out)-1,"rows")
