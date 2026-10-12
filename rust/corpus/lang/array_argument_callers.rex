/* sendWith, startWith, run, Message~new, callWith and both Supplier~new
   arguments convert through requestArray: an object without MAKEARRAY,
   MAKEARRAY answering .nil and a multi-dimensional array raise; MAKEARRAY
   answering an array and a string convert. */
r = .routine~new('r', 'return arg()')
m = .method~new('m', 'return arg()')
o = .object~new
k = .kk~new
n = .kn~new
do v over .array~of(o, k, n, 'ab', .array~new(2,2))
  call t "say 'abc'~sendWith('POS', v)", v
  call t "say 'abc'~startWith('POS', v)~result", v
  call t "say .runner~new~go(m, v)", v
  call t "say .message~new('abc', 'POS', 'A', v)~send", v
  call t "say r~callWith(v)", v
  call t "say .supplier~new(v, .array~of(1))~item", v
  call t "say .supplier~new(.array~of(1), v)~index", v
end
exit
t:
  signal on syntax
  interpret arg(1)
  return
syntax:
  say condition('O')['CODE'] condition('O')['MESSAGE']
  return
::class kk
::method makearray; return .array~of(7)
::class kn
::method makearray; return .nil
::class runner
::method go
  use arg m, v
  return self~run(m, 'A', v)
