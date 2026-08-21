/* Table C wiring row: the .InputStream environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .InputStream
say 'class-of-entry' .InputStream~class~id
say 'id' .InputStream~id
say 'class' .InputStream~class
say 'superclass' .InputStream~superClass
say 'superclasses' .InputStream~superClasses~makeString('L', ' ')
say 'metaclass' .InputStream~metaClass
say 'isa-class' .InputStream~isA(.Class)
