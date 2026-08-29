/* Phase 5b Task 1: the methods an instance answers that no gate-table row
   sends to one -- ~string, ~defaultName, ~objectName, ~objectName= , ~class
   and ~isA -- with Class~defaultName beside them, which nothing else reaches
   either. The renames are what separate the three names: ~objectName= moves
   ~string and ~objectName and leaves ~defaultName where it was, on a class
   receiver and on an instance alike. */
say 'object' .Object~new~string
o = .K~new
say 'string' o~string
say 'default' o~defaultName
say 'name' o~objectName
say 'class' o~class~id
say 'isa' o~isA(.K) o~isA(.Object) o~isA(.Class)
o~objectName = 'zed'
say 'renamed' o~string o~objectName o~defaultName
say 'classname' .K~defaultName .K~objectName .K~string
.K~objectName = 'kay'
say 'classrenamed' .K~defaultName .K~objectName .K~string
say 'sub' .SUB~new~string .SUB~new~defaultName
say 'article' .EGG~new~string

::CLASS K
::CLASS SUB SUBCLASS K
::CLASS EGG
