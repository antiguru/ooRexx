s = 'abc'
t = >s
signal on syntax
say t + 0
exit
syntax:
say condition('o')~code
do l over condition('o')~traceback; say l; end
call v2
exit
v2:
signal on syntax name s2
say t = 1
exit
s2:
say condition('o')~code
do l over condition('o')~traceback; say l; end
call v3
exit
v3:
signal on syntax name s3
say -t
exit
s3:
say condition('o')~code
do l over condition('o')~traceback; say l; end
exit
