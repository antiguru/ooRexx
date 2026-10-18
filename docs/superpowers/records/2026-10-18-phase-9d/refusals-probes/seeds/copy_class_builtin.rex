-- COPY of a built-in class object.
signal on syntax name trapped
c = .Array~copy
say 'copied' c~id (c == .Array) c~class~id
exit 0
trapped: say 'syntax' condition('O')~code
