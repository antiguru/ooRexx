k = .km~new
r = >k
say 'p' r
say 'arr' .array~of(r)~makestring
say 'arrk' .array~of(k)~makestring
say 'str' r~string
say 'cat' r || 'x'
say 'len' length(r)
trace r
z = r
trace off
say 'done'
exit
::class km
::method makestring; say '  (mk)'; return 'KMAKE'
::method defaultname; say '  (dn sent)'; return 'DNK'
