/* Table C wiring row: the .RexxQueue environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .RexxQueue
say 'class-of-entry' .RexxQueue~class~id
say 'id' .RexxQueue~id
say 'class' .RexxQueue~class
say 'superclass' .RexxQueue~superClass
say 'superclasses' .RexxQueue~superClasses~makeString('L', ' ')
say 'metaclass' .RexxQueue~metaClass
say 'isa-class' .RexxQueue~isA(.Class)
