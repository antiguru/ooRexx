# `.context~executable` in a method set with `setMethod`

Found by Phase 6 S2-S5 Task 9 through `TRACE_TraceObject.testGroup` `test_object_and_scope`. It
panicked (`class_graph.rs` index on a `.nil` scope); Task 9 made the `.nil`
scope answer the existing loud refusal "a method context whose scope no longer defines it". The
oracle answers the floating `Method` object, `~scope` `.nil` for `'float'`.

```rexx
t = .test~new
t~put('m1', .methods~fm, 'float')
t~m1
::method fm
  say .context~executable~scope
::class test
::method put
  use arg name, meth, scope
  self~setMethod(name, meth, scope)
```

Oracle: `The NIL object`, rc 0. a4bde5677: panic `no entry found for key`, rc 101. Task 9: the
loud refusal, rc 120.
