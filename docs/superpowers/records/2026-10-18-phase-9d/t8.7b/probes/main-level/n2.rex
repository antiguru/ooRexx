signal on syntax name s1
p = .package~new('pk', .array~of('say 1/0'))
s1: call show
signal on syntax name s2
r = .routine~new('rt', .array~of('::class c', '::constant k (1/0)'))
s2: call show
signal on syntax name s3
r = .routine~new('bad', .array~of('say ('))
s3: call show
signal on syntax name s4
m = .method~new('mm', .array~of('nop', '::routine x', '::routine x'))
s4: call show
exit
show:
  c = condition('o')
  say c~code c~traceback~makearray~makestring('l', '|')
  do f over c~stackframes; say '  ' f~type f~name f~line; end
  return
