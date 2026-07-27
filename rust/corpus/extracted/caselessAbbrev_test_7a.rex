/* extracted from caselessAbbrev::test_7a */
::routine main public
    skips = .array~new()
    skips~append('65-97')
    skips~append('66-98')
    skips~append('67-99')
    skips~append('68-100')
    skips~append('69-101')
    skips~append('70-102')
    skips~append('71-103')
    skips~append('72-104')
    skips~append('73-105')
    skips~append('74-106')
    skips~append('75-107')
    skips~append('76-108')
    skips~append('77-109')
    skips~append('78-110')
    skips~append('79-111')
    skips~append('80-112')
    skips~append('81-113')
    skips~append('82-114')
    skips~append('83-115')
    skips~append('84-116')
    skips~append('85-117')
    skips~append('86-118')
    skips~append('87-119')
    skips~append('88-120')
    skips~append('89-121')
    skips~append('90-122')
    skips~append('97-65')
    skips~append('98-66')
    skips~append('99-67')
    skips~append('100-68')
    skips~append('101-69')
    skips~append('102-70')
    skips~append('103-71')
    skips~append('104-72')
    skips~append('105-73')
    skips~append('106-74')
    skips~append('107-75')
    skips~append('108-76')
    skips~append('109-77')
    skips~append('110-78')
    skips~append('111-79')
    skips~append('112-80')
    skips~append('113-81')
    skips~append('114-82')
    skips~append('115-83')
    skips~append('116-84')
    skips~append('117-85')
    skips~append('118-86')
    skips~append('119-87')
    skips~append('120-88')
    skips~append('121-89')
    skips~append('122-90')

    do k = 1 to skips~items
        parse value skips[k] with i'-'j
        s = i~d2c()||j~d2c()
        t = j~d2c()
        self~assertSame((i \= j), s~caselessAbbrev(t, 1))
    end

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
