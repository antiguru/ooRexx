trace ?a
do label lbl
  nop
end lbl
do
  nop
end
do label rl i = 1 to 2
  nop
end rl
loop label fl
  leave fl
end fl
do label w while 0
end w
select label s
  when 1 then nop
end s
say 'end'
