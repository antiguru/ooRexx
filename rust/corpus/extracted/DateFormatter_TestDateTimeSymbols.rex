/* extracted from DateFormatter::TestDateTimeSymbols */
::routine main public

  formats = .DateFormats~new

  formats~dayNames = ("lunes", "martes", "miercoles", "jueves", "viernes", "sabado", "domingo")
  self~assertSame("2019 1 martes", .DateFormatter~format(.DateTime~fromIsoDate("2019-01-01T00:00:00.000000"), "yyyy w DDD", formats))

  formats~dayAbbreviations = ("lu", "ma", "mi", "ju", "vi", "sa", "do")
  self~assertSame("2019 1 ma", .DateFormatter~format(.DateTime~fromIsoDate("2019-01-01T00:00:00.000000"), "yyyy w DD", formats))

  formats~monthAbbreviations = ("ene", "feb", "mar", "abr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dic")
  self~assertSame("abr 11, 2019", .DateFormatter~format(.DateTime~fromStandardDate(20190411), "MMM dd, yyyy", formats))

  formats~monthNames = ("enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agusto", "septiembre", "octubre", "noviembre", "diciembre")
  self~assertSame("abril 11, 2019", .DateFormatter~format(.DateTime~fromStandardDate(20190411), "MMMM dd, yyyy", formats))

  formats~civilLabels = ("ax", "px")
  self~assertSame("2019/08/02 12:30px", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T12:30:00.000000"), "yyyy/MM/dd h:mmtt", formats))

  formats~civilShortLabels = ("m", "n")
  self~assertSame("2019/08/02 12:30m", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:30:00.000000"), "yyyy/MM/dd h:mmt", formats))

  formats~ordinalSuffixes = ("er", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e", "e")
  self~assertSame("1er", .DateFormatter~format(.DateTime~fromStandardDate(20190101), "ddddd", formats))
  self~assertSame("2e", .DateFormatter~format(.DateTime~fromStandardDate(20190102), "ddddd", formats))

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
