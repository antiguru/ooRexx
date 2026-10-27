e = .array~new(2); q = .array~of('p','q'); o = .k~new; n = .nostr~new; d = .directory~new; s = 'abc'
names = 'e q o n d s u'
do w over names~makearray(' ')
  call show w
end
exit
show: procedure expose e q o n d s u
  use arg w
  interpret 'ref = >'w
  say w 'L||' (ref || '|')~changestr('0a'x, ',')
  say w 'Lab' (ref'|')~changestr('0a'x, ',')
  say w 'Lbl' (ref '|')~changestr('0a'x, ',')
  say w 'R||' ('|' || ref)~changestr('0a'x, ',')
  say w 'Rab' ('|'ref)~changestr('0a'x, ',')
  say w 'Rbl' ('|' ref)~changestr('0a'x, ',')
  say w 'len' length(ref) left(ref, 1)
  say ref
  return
::class k
::method makestring; return 'kmake'
::class nostr
::method string; return 'nstr'
