/* ::REQUIRES ... NAMESPACE, the qualified lookups it makes reachable, and the
   package local environment directory. The required file is .cls so that the
   search's own .cls step resolves the extensionless name and so that neither
   helper is itself a corpus program. */
say 'main body'

say 'qualified class:' w:NsWidget~new~describe
say 'qualified call:' w:nsroutine()
call w:nsroutine
say 'qualified CALL result:' result

say 'transitive class:' w:NsDepClass~new~describe
say 'transitive call:' w:nsdeproutine()

say 'a namespace does not replace the merge:' .NsWidget~new~describe
say 'nor for a routine:' nsroutine()

say 'the REXX namespace:' rexx:Array
say 'a second qualifier for the same file:' v:NsWidget

say 'a bare name is the requiring file own class:' .NsClash~new~whose
say 'a qualifier reaches past it:' w:NsClash~new~whose

say 'subclassed through the qualifier:' .NsSub~new~describe
say 'inherited through it:' .NsSub~new~mixedIn
say 'metaclass through it:' .NsMeta2~new~describe
say 'a qualified SUBCLASS target:' .NsQualified~new~whose
say 'a bare SUBCLASS target:' .NsBare~new~whose

say 'no such namespace:' try("q:NsWidget")
say 'not public in it:' try("w:NsHidden")
say 'no such class in it:' try("w:NsNoSuchClass")
say 'not a public routine:' try("w:nsprivate()")
say 'the REXX namespace holds no routine:' try("rexx:length('abc')")
say 'nor a name that is not a class:' try("rexx:Endofline")

say 'the package local starts empty:' .zzznslocal
.context~package~local~zzznslocal = 'from the package local'
say 'and answers a dot name once written:' .zzznslocal
say 'it beats .local:' packageLocalBeatsLocal()
say 'a REXX class beats both:' rexxClassBeatsLocals()
say 'and the package local is one object:' .context~package~local~zzznslocal

::requires 'package_namespace_lib' namespace w
::requires 'package_namespace_lib' namespace v

::class NsSub subclass w:NsWidget inherit w:NsMixin
::class NsMeta2 subclass w:NsWidget metaclass w:NsMeta

::class NsClash
::method whose
  return 'the requiring file class'

::class NsQualified subclass w:NsClash
::class NsBare subclass NsClash

::routine try
  use arg text
  signal on syntax name raised
  interpret 'zzz = ' || text
  return 'UNEXPECTED, it answered' zzz
raised:
  return 'raised' rc || '.' || condition('e')

::routine packageLocalBeatsLocal
  .local~zzznsboth = 'from .local'
  .context~package~local~zzznsboth = 'from the package local'
  return .zzznsboth

::routine rexxClassBeatsLocals
  .local~array = 'from .local'
  return .Array
