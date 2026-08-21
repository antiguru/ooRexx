/* Table C wiring row: the .StreamSupplier environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .StreamSupplier~id
say 'class' .StreamSupplier~class
say 'superclass' .StreamSupplier~superClass
say 'superclasses' .StreamSupplier~superClasses~makeString('L', ' ')
say 'metaclass' .StreamSupplier~metaClass
say 'isa-class' .StreamSupplier~isA(.Class)
