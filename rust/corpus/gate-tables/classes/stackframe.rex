/* Table C wiring row: the .StackFrame environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .StackFrame
say 'class-of-entry' .StackFrame~class~id
say 'id' .StackFrame~id
say 'class' .StackFrame~class
say 'superclass' .StackFrame~superClass
say 'superclasses' .StackFrame~superClasses~makeString('L', ' ')
say 'metaclass' .StackFrame~metaClass
say 'isa-class' .StackFrame~isA(.Class)
