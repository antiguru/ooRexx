-- Class~methods enumeration counts, with and without the .nil argument.
s = .Array~methods(.nil)
n = 0
do while s~available; n += 1; s~next; end
say 'array methods(.nil)' n
s = .Array~methods
n = 0
do while s~available; n += 1; s~next; end
say 'array methods()' n
s = .Array~new~instanceMethods(.Array)
n = 0
do while s~available; n += 1; s~next; end
say 'array instanceMethods(.Array)' n
s = .Array~instanceMethods(.Array~class)
n = 0
do while s~available; n += 1; s~next; end
say 'class-side Array~instanceMethods(.Class)' n
s = .nil~instanceMethods(.Object)
n = 0
do while s~available; n += 1; s~next; end
say 'nil' n
