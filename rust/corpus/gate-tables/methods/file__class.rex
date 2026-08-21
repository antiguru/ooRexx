/* Table C method rows: File, class arm -- .File~hasMethod("M") for
   every method corpus/docs/class-methods.txt documents on this arm,
   one line per row and in the row set's own order. Derived by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'class' .File~hasMethod("isCaseSensitive")
say 'class' .File~hasMethod("listRoots")
say 'class' .File~hasMethod("pathSeparator")
say 'class' .File~hasMethod("readChars")
say 'class' .File~hasMethod("readLines")
say 'class' .File~hasMethod("searchPath")
say 'class' .File~hasMethod("separator")
say 'class' .File~hasMethod("temporaryPath")
say 'class' .File~hasMethod("writeChars")
say 'class' .File~hasMethod("writeLines")
