/* Table C wiring row: the .OutputStream environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .OutputStream
say 'class-of-entry' .OutputStream~class~id
say 'id' .OutputStream~id
say 'class' .OutputStream~class
say 'superclass' .OutputStream~superClass
say 'superclasses' .OutputStream~superClasses~makeString('L', ' ')
say 'metaclass' .OutputStream~metaClass
say 'isa-class' .OutputStream~isA(.Class)
