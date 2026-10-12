/* .nil's behaviour hides COPY, START and OBJECTNAME= (97.1), and keeps
   the rest of Object's. */
say .nil~hasMethod('COPY') .nil~hasMethod('START') .nil~hasMethod('OBJECTNAME=') .nil~hasMethod('OBJECTNAME') .nil~hasMethod('STARTWITH')
say .nil~instanceMethod('COPY') .nil~instanceMethods(.nil)~available
say .nil~objectName .nil~string
signal on syntax
.nil~copy
syntax: say condition('O')~message
signal on syntax name s2
.nil~start('isNil')
s2: say condition('O')~message
signal on syntax name s3
.nil~objectName = 'x'
s3: say condition('O')~message
say .nil~startWith('isNil', .array~new)~result
