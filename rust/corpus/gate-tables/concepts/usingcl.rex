/* provide.xml `usingcl`: a new class is always a subclass of an existing one,
   and can be created by sending SUBCLASS or MIXINCLASS to a class rather than
   by a directive; without either option the superclass is Object. */
persistence = .object~mixinclass("Persistence")
myarray = .array~subclass("myarray")~~inherit(persistence)
say 'mixin-base' persistence~baseClass~id
say 'subclass-of' myarray~superClass~id
say 'superclasses' myarray~superClasses~makeString('L', ' ')
say 'default-superclass' .object~subclass("plain")~superClass~id
