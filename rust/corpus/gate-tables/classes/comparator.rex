/* Table C wiring row: the .Comparator environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Comparator
say 'class-of-entry' .Comparator~class~id
say 'id' .Comparator~id
say 'class' .Comparator~class
say 'superclass' .Comparator~superClass
say 'superclasses' .Comparator~superClasses~makeString('L', ' ')
say 'metaclass' .Comparator~metaClass
say 'isa-class' .Comparator~isA(.Class)
