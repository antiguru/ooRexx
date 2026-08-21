/* Table C wiring row: the .CircularQueue environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .CircularQueue
say 'class-of-entry' .CircularQueue~class~id
say 'id' .CircularQueue~id
say 'class' .CircularQueue~class
say 'superclass' .CircularQueue~superClass
say 'superclasses' .CircularQueue~superClasses~makeString('L', ' ')
say 'metaclass' .CircularQueue~metaClass
say 'isa-class' .CircularQueue~isA(.Class)
