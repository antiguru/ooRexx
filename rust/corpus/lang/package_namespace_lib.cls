/* The library a NAMESPACE qualifier names. Its public class and public
   routine are reachable both qualified and unqualified; its private ones are
   reachable neither way. */
say 'nslib prologue'

::requires 'package_namespace_dep'

::class NsWidget public
::method describe
  return 'a namespace widget'

::class NsMixin public mixinclass Object
::method mixedIn
  return 'mixed in through a namespace'

::class NsMeta public subclass Class

::class NsClash public
::method whose
  return 'the library class'

::class NsHidden
::method describe
  return 'not exported'

::routine nsroutine public
  return 'nsroutine ran'

::routine nsprivate
  return 'nsprivate ran'
