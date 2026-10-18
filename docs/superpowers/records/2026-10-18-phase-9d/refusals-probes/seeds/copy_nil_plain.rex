-- COPY sent to .nil directly.
signal on syntax name trapped
c = .nil~copy
say 'copied' (c == .nil)
exit 0
trapped: say 'syntax' condition('O')~code
