/* A class is built by NEW sent to its metaclass, then made a subclass and
   sent INIT again: a metaclass's own NEW runs, and the INIT its instances
   take runs twice per class, for ~subclass and a ::CLASS directive alike. */
say 'start'
k = .Base~subclass('k', .MyMeta)
say 'k:' k~id k~class~id k~superclass~id k~metaclass~id
j = .MyMeta~subclass('j')
say 'j:' j~id j~class~id j~superclass~id j~isMetaclass
m = .object~mixinclass('m', .MyMeta)
say 'm:' m~id m~queryMixinClass m~class~id
t = .T~new('arg1')
say 't:' t~string
d = .directory~new; d~foo = "return 'foo'"
e = .T~enhanced(d, 'earg')
say 'e:' e~string e~foo
c = .object~subclass('dyn', .Counted)
say c~id c~class~id
p = .K~subclass('p')
say p~id p~class~id
::class MyMeta subclass Class
::method new class
  say 'MyMeta NEW class, self=' self~id 'args' arg() arg(1)
  forward class (super)
::method init
  say 'MyMeta instance INIT self=' self~id 'args' arg() 'super' self~superclass~id
  forward class (super)
::class Meta2 subclass Class
::method new
  say 'Meta2 instance NEW self=' self~id 'args' arg() arg(1)
  forward class (super)
::class T metaclass Meta2
::method init
  say 'T INIT' arg() arg(1)
::class Counted subclass Class
::method init
  say 'Counted INIT for' self~id
::class K metaclass Counted
::class Base
