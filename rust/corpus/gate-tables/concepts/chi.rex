/* provide.xml `chi`: the classes Rexx provides belong to the Object class and
   are listed in a hierarchy, an inheriting class indented below its
   superclass or mixin class. */
say 'root' .Object~superClass
say 'collection' .Collection~superClass~id
say 'ordered' .OrderedCollection~superClass~id
say 'array' .Array~superClass~id
say 'array-superclasses' .Array~superClasses~makeString('L', ' ')
say 'circularqueue' .CircularQueue~superClass~id
say 'string' .String~superClasses~makeString('L', ' ')
say 'stream' .Stream~superClass~id
