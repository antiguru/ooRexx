/* Table C wiring row: the .MessageNotification environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .MessageNotification~id
say 'class' .MessageNotification~class
say 'superclass' .MessageNotification~superClass
say 'superclasses' .MessageNotification~superClasses~makeString('L', ' ')
say 'metaclass' .MessageNotification~metaClass
say 'isa-class' .MessageNotification~isA(.Class)
