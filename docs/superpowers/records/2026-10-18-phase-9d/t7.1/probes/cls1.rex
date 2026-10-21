signal on syntax name trapped
c = .Message~new(.zz, .array~of('COPY', .Object))~send
say 'a' c~id c~objectName c~string c~defaultName (c = .zz) (c == .zz) (c \== .zz)
say 'b' c~superclass~id c~superclasses~items c~subclasses~items .zz~subclasses~items
say 'c' c~isSubclassOf(.Object) .zz~isSubclassOf(c) c~isSubclassOf(.zz)
o = c~new
say 'd' o~class~id (o~class == .zz) o~isA(c) o~isA(.zz) o~isInstanceOf(c) c~cm
say 'e' c~methods~allIndexes~items c~instanceMethods~allIndexes~items c~method('M')~class~id
say 'f' (c~method('M') == .zz~method('M')) c~queryMixinClass c~metaClass~id c~package~name~length
c~define('N', 'return "n"')
say 'g' c~new~n
say 'h' .zz~new~hasMethod('N') c~new~hasMethod('N')
s = c~subclass('SUB')
say 'i' s~id s~superclass~id (s~superclass == c) s~new~m s~new~class~id
say 'j' c~hashCode == .zz~hashCode, c~identityHash == .zz~identityHash
.zz~define('P', 'return "p"')
say 'k' c~new~hasMethod('P') .zz~new~hasMethod('P')
c~objectName = 'named'; say 'l' c~objectName .zz~objectName c~id
say 'm' c~isMetaclass
d = .Message~new(c, .array~of('COPY', .Object))~send
say 'n' d~id (d == c)
say 'o' c~class~id c~class == .Class
say 'p' c~inherit(.mx) 
say 'q' c~new~mm .zz~new~hasMethod('MM')
say 'r' c~annotations~items
exit 0
trapped: say 'syntax' condition('O')~code condition('O')~message condition('O')~position
::class zz
::method m
  return 'm'
::method cm class
  return 'cm'
::class mx mixinclass object
::method mm
  return 'mm'
