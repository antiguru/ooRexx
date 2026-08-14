trace r
call on user zx name h
do until raiser() > 0
  say 'body'
end
say 'after'
exit 0
raiser:
raise user zx return 5
h:
say 'in h'
return
