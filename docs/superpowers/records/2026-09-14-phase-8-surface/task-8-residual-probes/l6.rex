signal on syntax name s
interpret 'do i = 1 to 2; if i = 2 then call bad; end'
exit
s: c = condition('O'); do l over c~traceback; say l; end; exit
bad: x = .nil + 1; return
