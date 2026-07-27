/* extracted from Object::test_uninit */
::routine main public
  -- base test with no uninit method
  ref = .weakReference~new(.UninitTester~new)
  hash = ref~value~identityHash
  self~assertFalse(.UninitTracker~forceUninit(ref, hash))

  -- base test with uninit method
  ref = .weakReference~new(.ClassWithUninitMethod~new)
  hash = ref~value~identityHash
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- base test with uninit method inherited from mixin
  ref = .weakReference~new(.UninitFromMixin~new)
  hash = ref~value~identityHash
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- now create a subclass of two classes above, but
  -- use define to hide the uninit method
  newClass = .ClassWithUninitMethod~subclass('NoUninit')
  newClass~define('UNINIT')

  ref = .weakReference~new(newClass~new)
  hash = ref~value~identityHash
  self~assertFalse(.UninitTracker~forceUninit(ref, hash))

  -- now add an uninit method back
  newClass~define('UNINIT', ".UninitTracker~recordUninit(self)")

  ref = .weakReference~new(newClass~new)
  hash = ref~value~identityHash
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- this will delete the uninit method, exposing the original from the base class
  newClass~delete('UNINIT')

  ref = .weakReference~new(newClass~new)
  hash = ref~value~identityHash
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- base test with no uninit method
  obj = .UninitTester~new
  -- now add an uninit method to the instance
  obj~testSet('UNINIT', ".UninitTracker~recordUninit(self)")

  ref = .weakReference~new(obj)
  hash = ref~value~identityHash
  drop obj
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- base test with no uninit method
  obj = .UninitTester~new
  -- now add an uninit method to the instance
  obj~testSet('UNINIT', ".UninitTracker~recordUninit(self)")
  -- and remove the method again
  obj~testUnSet('UNINIT')

  ref = .weakReference~new(obj)
  hash = ref~value~identityHash
  drop obj
  self~assertFalse(.UninitTracker~forceUninit(ref, hash))

  obj = .ClassWithUninitMethod~new
  -- now hide the unint method on this instance
  obj~testSet('UNINIT')

  ref = .weakReference~new(obj)
  hash = ref~value~identityHash
  drop obj
  self~assertFalse(.UninitTracker~forceUninit(ref, hash))

  -- test using a copied object
  ref = .weakReference~new(.ClassWithUninitMethod~new~copy)
  hash = ref~value~identityHash
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- create an enhanced object that adds an UNINIT method
  methods = .stringtable~of(('UNINIT', ".UninitTracker~recordUninit(self)"))
  obj = .UninitTester~enhanced(methods)

  ref = .weakReference~new(obj)
  hash = ref~value~identityHash
  drop obj
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- create an enhanced object that hides the UNINIT method
  methods = .stringtable~of(('UNINIT', .nil))
  obj = .ClassWithUninitMethod~enhanced(methods)

  ref = .weakReference~new(obj)
  hash = ref~value~identityHash
  drop obj
  self~assertFalse(.UninitTracker~forceUninit(ref, hash))

  -- When a method is defined on a class, changes need to be
  -- reflected by the subclass also.  Create a subclass of our
  -- base test class, then subclass that again.  We add/remove
  -- UNINIT methods to the first subclass and verify that new
  -- instances of the second class function correctly.
  firstClass = .UninitTester~subclass('FirstClass')
  secondClass = firstClass~subclass('SecondClass')
  firstClass~define('UNINIT', ".UninitTracker~recordUninit(self)")

  ref = .weakReference~new(secondClass~new)
  hash = ref~value~identityHash
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- now delete the uninit method to restore to original state
  firstClass~delete('UNINIT')

  ref = .weakReference~new(secondClass~new)
  hash = ref~value~identityHash
  self~assertFalse(.UninitTracker~forceUninit(ref, hash))

  -- this is the reverse of above.  We start with a class that
  -- has an uninit method then hid it and restore it.
  firstClass = .ClassWithUninitMethod~subclass('FirstClass')
  secondClass = firstClass~subclass('SecondClass')
  -- this hides the method
  firstClass~define('UNINIT')

  ref = .weakReference~new(secondClass~new)
  hash = ref~value~identityHash
  self~assertFalse(.UninitTracker~forceUninit(ref, hash))

  -- now delete the uninit method to restore to original state
  firstClass~delete('UNINIT')

  ref = .weakReference~new(secondClass~new)
  hash = ref~value~identityHash
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- now the same, using define methods to do the updates
  firstClass = .UninitTester~subclass('FirstClass')
  secondClass = firstClass~subclass('SecondClass')
  methods = .stringtable~of(('UNINIT', ".UninitTracker~recordUninit(self)"))
  firstClass~defineMethods(methods)

  ref = .weakReference~new(secondClass~new)
  hash = ref~value~identityHash
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))

  -- now delete the uninit method to restore to original state
  firstClass~delete('UNINIT')

  ref = .weakReference~new(secondClass~new)
  hash = ref~value~identityHash
  self~assertFalse(.UninitTracker~forceUninit(ref, hash))

  -- this is the reverse of above.  We start with a class that
  -- has an uninit method then hid it and restore it.
  firstClass = .ClassWithUninitMethod~subclass('FirstClass')
  secondClass = firstClass~subclass('SecondClass')
  methods = .stringtable~of(('UNINIT', .nil))
  -- this hides the method
  firstClass~defineMethods(methods)

  ref = .weakReference~new(secondClass~new)
  hash = ref~value~identityHash
  self~assertFalse(.UninitTracker~forceUninit(ref, hash))

  -- now delete the uninit method to restore to original state
  firstClass~delete('UNINIT')

  ref = .weakReference~new(secondClass~new)
  hash = ref~value~identityHash
  self~assertTrue(.UninitTracker~forceUninit(ref, hash))



-- TODO:  add UNKNOWN method tests
-- TODO:  add hashcode/== override tests for collections.

-- hashCode method

-- if hashCode doesn't return a value, RexxObject::hash() must raise an error
::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
