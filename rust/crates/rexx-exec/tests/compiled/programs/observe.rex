say 'sl()' sourceline()
parse source s
say 'ps' s
say 'pkgsrc' .context~package~source~items
say 'ctxsrc' .context~package~sourceSize
r = .context~package~findroutine('R')
say 'rsrc' r~source~items
call r
signal on syntax
say 'sl1' sourceline(1)
exit
syntax:
say 'cond' condition('o')~code
say 'msg' condition('o')~message
say 'tb' condition('o')~traceback~items
do l over condition('o')~traceback; say '[' l ']'; end
call t
exit
t:
trace r
x = 1 + 2
say x
trace i
y = x * 2
trace o
say 'raise'
z = 1 + 'a'
::routine r
say 'in r'
return
