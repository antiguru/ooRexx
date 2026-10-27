/* operators on a stem with a default and on a stem subclass instance */
parse arg w
signal on syntax
s. = 5
t = .stem~subclass('m')~new
t[] = 5
if w = 1 then say (s. = 5) (s. == 5) ('5' = s.)
if w = 2 then say (t = 5) (t == 5) (t = t) ('5' = t)
if w = 3 then say s. + 1
if w = 4 then say t + 1
if w = 5 then say 1 + t
if w = 6 then say (s. < 6)
if w = 7 then say (t < 6)
if w = 8 then say -t
say 'end' w
exit
syntax: say 'SYNTAX' w condition('O')~code; exit
