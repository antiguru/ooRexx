-- COPY of a RexxContext.
signal on syntax name trapped
c = .context~copy
say 'copied' c~class~id
exit 0
trapped: say 'syntax' condition('O')~code
