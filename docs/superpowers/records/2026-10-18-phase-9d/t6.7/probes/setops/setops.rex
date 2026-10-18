o1=.object~new; o2=.object~new
do clz over .array~of(.queue, .circularqueue~new(20)~class)
  if clz == .circularqueue then do
    c1 = .circularqueue~of("1v", "2v", "2v", "2v", o1, o1)
    c2 = .circularqueue~of("2v", "2v", o1, o1, o2, o2, "3v")
  end
  else do
    c1=clz~of("1v", "2v", "2v", "2v", o1, o1)
    c2=clz~of("2v", "2v", o1, o1, o2, o2, "3v")
  end
  call show clz~id "diff12", c1~difference(c2)
  call show clz~id "diff21", c2~difference(c1)
  call show clz~id "xor12", c1~xor(c2)
  call show clz~id "xor21", c2~xor(c1)
  call show clz~id "int12", c1~intersection(c2)
  call show clz~id "union12", c1~union(c2)
end
q = .queue~of("a","b","c","b")
q~removeItem("b"); call show "removeItem", q
say q~at(1) q~at(2) q~at(3)
exit
show: procedure expose o1 o2
  use arg label, c
  s = ""
  do i over c~allIndexes; s = s i; end
  t = ""
  do x over c~allItems
    if x == o1 then x = "o1"; else if x == o2 then x = "o2"
    t = t x
  end
  say label c~class~id "items" c~items "idx[" s "] items[" t "]"
  return
