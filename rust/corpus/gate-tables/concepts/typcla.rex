/* provide.xml `typcla`: the kinds of class the section names -- object,
   mixin, abstract and metaclass. One of each is declared here and asked a
   class-side question that distinguishes it. */
say 'object' .oc~id .oc~class~id
say 'mixin' .mx~id .mx~baseClass~id
say 'abstract' .ab~id .ab~class~id
say 'metaclass' .mc~id .k~class~id

::class oc
::class mx mixinclass object
::class ab abstract
::class mc subclass class
::class k metaclass mc
