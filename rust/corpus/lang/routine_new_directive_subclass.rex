/* A ::CLASS directive in a Routine~new source resolves its SUBCLASS through
   the installing package and its parent, not through the caller. */
/* Routine~new source with directives that subclass a parent class */
signal on syntax name s1
say 'routine subclass:' .routine~new('r', .array~of('return .kid~new~name', '::class kid subclass hidden'))~call
s1r:
signal on syntax name s2
p1 = .package~new('p1', .array~of('::class other', '::method name', '  return "other"'))
say 'routine subclass other ctx:' .routine~new('r', .array~of('return .kid~new~name', '::class kid subclass other'), p1)~call
s2r:
signal on syntax name s3
say 'routine subclass hidden via p1:' .routine~new('r', .array~of('return .kid~new~name', '::class kid subclass hidden'), p1)~call
exit
s1: say 'routine subclass: error' condition('O')~code; signal s1r
s2: say 'routine subclass other ctx: error' condition('O')~code; signal s2r
s3: say 'routine subclass hidden via p1: error' condition('O')~code; exit
::class hidden
::method name
  return 'hidden'
