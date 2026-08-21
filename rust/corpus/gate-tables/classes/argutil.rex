/* Table C wiring row: the .ArgUtil environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .ArgUtil
say 'class-of-entry' .ArgUtil~class~id
say 'id' .ArgUtil~id
say 'class' .ArgUtil~class
say 'superclass' .ArgUtil~superClass
say 'superclasses' .ArgUtil~superClasses~makeString('L', ' ')
say 'metaclass' .ArgUtil~metaClass
say 'isa-class' .ArgUtil~isA(.Class)
