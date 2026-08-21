/* Table C wiring row: the .InputOutputStream environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .InputOutputStream
say 'class-of-entry' .InputOutputStream~class~id
say 'id' .InputOutputStream~id
say 'class' .InputOutputStream~class
say 'superclass' .InputOutputStream~superClass
say 'superclasses' .InputOutputStream~superClasses~makeString('L', ' ')
say 'metaclass' .InputOutputStream~metaClass
say 'isa-class' .InputOutputStream~isA(.Class)
