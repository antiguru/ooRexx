/* Table C wiring row: the .RexxContext environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .RexxContext
say 'class-of-entry' .RexxContext~class~id
say 'id' .RexxContext~id
say 'class' .RexxContext~class
say 'superclass' .RexxContext~superClass
say 'superclasses' .RexxContext~superClasses~makeString('L', ' ')
say 'metaclass' .RexxContext~metaClass
say 'isa-class' .RexxContext~isA(.Class)
