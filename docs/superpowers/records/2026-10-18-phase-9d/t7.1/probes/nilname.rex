.Message~new(.nil, .array~of('OBJECTNAME=', .Object), 'I', 'zz')~send
say .nil .nil~string .nil~objectName .nil~defaultName length(.nil) 'a'.nil
a = .array~of(1, .nil, 3)
say a~toString(, ',')
say (.nil == 'zz') (.nil = 'zz') ('zz' == .nil)
x = .nil
say x
say left(.nil, 3) reverse(.nil)
trace r
y = .nil
trace off
signal on syntax name t1
z = .nil + 1
t1: say condition('O')~message
signal on syntax name t2
.nil~frob
t2: say condition('O')~message
c = .Message~new(.nil, .array~of('COPY', .Object))~send
say c c~objectName (c == .nil) c~class~id
.Message~new(.nil, .array~of('OBJECTNAME=', .Object), 'I', 'The NIL object')~send
say .nil c
