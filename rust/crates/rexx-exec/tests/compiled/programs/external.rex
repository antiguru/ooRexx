signal on syntax
call 'fail.rxo'
syntax: do l over condition('o')~traceback; say l; end
signal on syntax name s3
x = fail.rxo()
s3: do l over condition('o')~traceback; say l; end
