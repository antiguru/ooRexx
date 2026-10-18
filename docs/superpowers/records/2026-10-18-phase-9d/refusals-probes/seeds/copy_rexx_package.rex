-- COPY of the REXX package.
signal on syntax name trapped
p = .Array~package
c = p~copy
say 'copied' c~class~id (c == p) c~name
exit 0
trapped: say 'syntax' condition('O')~code
