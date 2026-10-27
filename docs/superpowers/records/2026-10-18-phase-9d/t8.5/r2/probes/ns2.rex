q = .array~of('p','q'); k = .kmake~new; m = .knum~new; n = .nstr~new; d = .directory~new; s = 'plain'
s.1 = 'one'; s. = 'dflt'; drop u
list = 'q k m n d s s. u'
do w over list~makearray(' ')
  interpret 'r = >'w
  rr = >r
  call doit w 'say', 'say r'
  call doit w 'lcat', 'x = r || "|"; say x'
  call doit w 'rcat', 'x = "|" || r; say x'
  call doit w 'labut', 'x = r"|"; say x'
  call doit w 'rabut', 'x = ("|")r; say x'
  call doit w 'lblank', 'x = r "|"; say x'
  call doit w 'rblank', 'x = "|" r; say x'
  call doit w 'len', 'say length(r)'
  call doit w 'pos', 'say pos("q", r)'
  call doit w 'substr', 'say substr(r, 1, 2)'
  call doit w 'd2x', 'say d2x(r)'
  call doit w 'cmpR', 'say ("kmake" = r) ("kmake" == r)'
  call doit w 'cmpL', 'say (r = "kmake") (r == "kmake")'
  call doit w 'addR', 'say (1 + r)'
  call doit w 'addL', 'say (r + 1)'
  call doit w '~string', 'say r~string'
  call doit w '~makestring', 'say r~makestring'
  call doit w 'rr-say', 'say rr'
  call doit w 'rr-lcat', 'say rr || "|"'
  call doit w 'rr-rcat', 'say "|" || rr'
  call doit w 'pval', 'parse value r with v; say v'
  call doit w 'pvar', 'parse var r v; say v'
end
exit
doit:
  signal on nostring name h
  signal on syntax name hs
  parse arg tag, code
  call charout , tag': '; interpret code
  return
h:
  say '  NOSTRING' condition('d') condition('o')~additional~class~id
  return
hs:
  say '  SYNTAX' condition('o')~code condition('o')~message
  return
::class kmake
::method makestring; return 'kmake'
::class knum
::method makestring; return '5'
::class nstr
::method string; return 'nstr'
