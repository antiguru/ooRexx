parse arg which
signal on syntax
s. = >s.
say 'made'
if which = 1 then say s. = 'a'
if which = 2 then say 'a' = s.
if which = 3 then say s.~request('STRING')
if which = 4 then say s.~string
if which = 5 then do; trace r; x = s.; trace o; say 'traced'; end
if which = 6 then say -s.
if which = 7 then say \s.
if which = 8 then do; r = >s.; say r; end
if which = 9 then do; r = >s.; say r~foo; end
if which = 10 then say s.~makestring
if which = 11 then say s.[]
if which = 12 then say s.~length
say 'end' which
exit
syntax: say 'SYNTAX' which rc condition('O')~code; exit
