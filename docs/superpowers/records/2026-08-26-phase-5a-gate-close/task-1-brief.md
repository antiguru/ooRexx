## Task 1: `::ATTRIBUTE EXTERNAL`

**Goal.** `corpus/gate-tables/directives/attribute__external__subkeyword.rex` agrees.

**Measured.** Oracle **rc 166**, stderr echoing the directive's own line 3, then
`Error 90 running <path> line 3:  External name not found.` and
`Error 90.998:  Unable to find external method "GETzzz_no_entry".` Crate: rc 120,
`::ATTRIBUTE EXTERNAL is not implemented (Phase 7)`.

**Build.** `GET` and `SET` are **prepended to the procedure name**, not appended to the method name --
`concatToCstring` appends its receiver to its argument (`classes/StringClass.cpp:1405`-`:1416`), and
the previous plan's Task 22 recorded the measurement. Resolve `GET`+procedure against the
`LIBRARY REXX` entry-point registry Task 22 built, raise 90.998 naming the composed name on the miss,
and the `SET` half after it. `::METHOD ... ATTRIBUTE EXTERNAL` is the same mechanism
(`parser/DirectiveParser.cpp:867`, `:1678`) and both spellings must move together or neither.

**The registry exports no `GET*`/`SET*` entry at all** -- measured,
`/bin/grep -n "INTERNAL_METHOD(GET" interpreter/runtime/NativeMethods.h` matches nothing, and the
`SET` form likewise -- so **every** `::ATTRIBUTE ... EXTERNAL 'LIBRARY REXX x'` raises 90.998 on both
sides. That makes the work small and complete rather than partial.

**Done when** the row agrees on both engines, `::METHOD ... ATTRIBUTE EXTERNAL` agrees too, and a
control is recorded: appending `GET` instead of prepending it names `zzz_no_entryGET` and the row
reddens.

---

