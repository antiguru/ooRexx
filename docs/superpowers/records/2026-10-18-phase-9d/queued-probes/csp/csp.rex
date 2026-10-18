signal on syntax
r = .Routine~new('Testing', 'return arg(1')
exit
syntax: do l over condition("O")~traceback; say l; end
