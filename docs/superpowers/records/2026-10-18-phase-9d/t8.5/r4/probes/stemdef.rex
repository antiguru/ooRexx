/* stem whose default is a reference to the stem */
parse arg which
signal on syntax
if which = 1 then do; s. = >s.; say 'made'; say s.; end
if which = 2 then do; r = >s.; s. = r; say 'made'; say length(s.x); end
if which = 3 then do; a = .array~new; a[1] = >a; say 'made'; say a~makestring; say length(a); end
if which = 4 then do; s. = >s.; say 'made'; say 1 + s.; end
if which = 5 then do; r = >r2; r2 = .array~of(>r); say 'made'; say length(r); end
say 'end' which
exit
syntax: say 'SYNTAX' which rc condition('O')~code; exit
