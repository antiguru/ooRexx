/* Table C wiring row: the .Alarm environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Alarm
say 'class-of-entry' .Alarm~class~id
say 'id' .Alarm~id
say 'class' .Alarm~class
say 'superclass' .Alarm~superClass
say 'superclasses' .Alarm~superClasses~makeString('L', ' ')
say 'metaclass' .Alarm~metaClass
say 'isa-class' .Alarm~isA(.Class)
