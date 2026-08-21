/* Table C wiring row: the .TraceObject environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .TraceObject
say 'class-of-entry' .TraceObject~class~id
say 'id' .TraceObject~id
say 'class' .TraceObject~class
say 'superclass' .TraceObject~superClass
say 'superclasses' .TraceObject~superClasses~makeString('L', ' ')
say 'metaclass' .TraceObject~metaClass
say 'isa-class' .TraceObject~isA(.Class)
