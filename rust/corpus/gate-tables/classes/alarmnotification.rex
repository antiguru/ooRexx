/* Table C wiring row: the .AlarmNotification environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .AlarmNotification~id
say 'class' .AlarmNotification~class
say 'superclass' .AlarmNotification~superClass
say 'superclasses' .AlarmNotification~superClasses~makeString('L', ' ')
say 'metaclass' .AlarmNotification~metaClass
say 'isa-class' .AlarmNotification~isA(.Class)
