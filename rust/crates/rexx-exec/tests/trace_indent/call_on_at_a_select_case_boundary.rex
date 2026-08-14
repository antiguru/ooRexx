trace r
call on user zx name h
zv = 'unset'
select case raiser()
  when 1 then say 'one'
  otherwise say 'other'
end
say 'after' zv
exit
raiser:
raise user zx return 2
h:
zv = 'set'
return
