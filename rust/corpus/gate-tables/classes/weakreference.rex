/* Table C wiring row: the .WeakReference environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .WeakReference~id
say 'class' .WeakReference~class
say 'superclass' .WeakReference~superClass
say 'superclasses' .WeakReference~superClasses~makeString('L', ' ')
say 'metaclass' .WeakReference~metaClass
say 'isa-class' .WeakReference~isA(.Class)
