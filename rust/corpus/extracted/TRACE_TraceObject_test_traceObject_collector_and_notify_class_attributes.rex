/* extracted from TRACE_TraceObject::test_traceObject_collector_and_notify_class_attributes */
::routine main public

callerName="testTraceObjectCollectorAndNotifyClassAttributes"

r=.routine~new(callerName, .resources~test_traceObject_collector_and_notify_class_attributes)
resArr=r~call         -- note: this is a method call to an object!

   -- collector class attribute values
cRefArray =resArr[1]~refArray
cCpyArray=resArr[1]~atAppendArray   -- values at creation time, must be different

   -- notify class attribute values
nRefArray    =resArr[2]~refArray
nCpyArray   =resArr[2]~atAppendArray   -- values at creation time, must be identical

   -- same number of entries all over?
self~assertEquals(cRefArray~items, cCpyArray~items)
self~assertEquals(nRefArray~items, nCpyArray~items)
self~assertEquals(nRefArray~items, cRefArray~items)

do i=1 to cRefArray~items
   cTrObj1=cRefArray[i]    -- collector traceObject
   cTrObj2=cCpyArray[i]    -- collector traceObject
   nTrObj1=nRefArray[i]    -- notify traceObject
   nTrObj2=nCpyArray[i]    -- notify traceObject

      -- traceObjects in cRefArray, nRefArry, nCpyArray must be identical
   self~assertSame(    cTrObj1, nTrObj1 ) -- traceObjects identical
   self~assertNotSame( cTrObj1, cTrObj2 ) -- traceObjects must not be identical (a copy)
   self~assertNotSame( nTrObj1, nTrObj2 ) -- traceObjects must not be identical (a copy)

      -- check collector traceObjects
   self~assertEquals( 3, cTrObj2~items)    -- only the entries NUMBER, OPTION, TIMESTAMP must be available
   self~assertFalse( cTrObj1~items = cTrObj2~items )
   self~assertEquals( cTrObj1~number   , cTrObj2~number )
   self~assertEquals( cTrObj1~position , cTrObj2~position )
   self~assertEquals( cTrObj1~timeStamp, cTrObj2~timeStamp )

   call check4IdenticalContent cTrObj1, nTrObj1
   call check4IdenticalContent nTrObj1, nTrObj2
end
return

check4IdenticalContent: procedure expose self
  use strict arg trObj1, trObj2
  do idx over trObj1~allIndexes
     self~assertEquals(trObj1[idx], trObj2[idx])
  end
  return

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
