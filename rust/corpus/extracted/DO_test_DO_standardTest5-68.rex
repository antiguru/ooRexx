/* extracted from DO::test_DO_standardTest5-68 */
::routine main public
   c=0;cc=1
   i=1
   Do i.1=1 To 1
   Do i.2=1 To 1
   Do i.3=1 To 1
   Do i.4=1 To 1
   Do i.5=1 To 1
   Do i.6=1 To 1
   Do i.7=1 To 1
   Do i.8=1 To 1
   Do i.9=1 To 1
   Do i.10=1 To 1
   Do i.11=1 To 1
   Do i.12=1 To 1
   Do i.13=1 To 1
   Do i.14=1 To 1
   Do i.15=1 To 1
   Do i.16=1 To 1
   Do i.17=1 To 1
   Do i.18=1 To 1
   Do i.19=1 To 1
   Do i.20=1 To 1
   Do i.21=1 To 1
   Do i.22=1 To 1
   Do i.23=1 To 1
   Do i.24=1 To 1
   Do i.25=1 To 1
   Do i.26=1 To 1
   Do i.27=1 To 1
   Do i.28=1 To 1
   Do i.29=1 To 1
   Do i.30=1 To 1
   Do i.31=1 To 1
   Do i.32=1 To 1
   Do i.33=1 To 1
   Do i.34=1 To 1
   Do i.35=1 To 1
   Do i.36=1 To 1
   Do i.37=1 To 1
   Do i.38=1 To 1
   Do i.39=1 To 1
   Do i.40=1 To 1
   Do i.41=1 To 1
   Do i.42=1 To 1
   Do i.43=1 To 1
   Do i.44=1 To 1
   Do i.45=1 To 1
   Do i.46=1 To 1
   Do i.47=1 To 1
   Do i.48=1 To 1
   Do i.49=1 To 1
   Do i.50=1 To 1
   Do i.51=1 To 1
   Do i.52=1 To 1
   Do i.53=1 To 1
   Do i.54=1 To 1
   Do i.55=1 To 1
   Do i.56=1 To 1
   Do i.57=1 To 1
   Do i.58=1 To 1
   Do i.59=1 To 1
   Do i.60=1 To 1
   Do i.61=1 To 1
   Do i.62=1 To 1
   Do i.63=1 To 1
   Do i.64=1 To 1
   Do i.65=1 To 1
   Do i.66=1 To 1
   Do i.67=1 To 1
   Do i.68=1 To 1
   Do i.69=1 To 1
   Do i.70=1 To 1
   Do i.71=1 To 1
   Do i.72=1 To 1
   Do i.73=1 To 1
   Do i.74=1 To 1
   Do i.75=1 To 1
   Do i.76=1 To 1
   Do i.77=1 To 1
   Do i.78=1 To 1
   Do i.79=1 To 1
   Do i.80=1 To 1
   Do i.81=1 To 1
   Do i.82=1 To 1
   Do i.83=1 To 1
   Do i.84=1 To 1
   Do i.85=1 To 1
   Do i.86=1 To 1
   Do i.87=1 To 1
   Do i.88=1 To 1
   Do i.89=1 To 1
   Do i.90=1 To 1
   Do i.91=1 To 1
   Do i.92=1 To 1
   c=c+1
   End i.92
   End i.91
   End i.90
   End i.89
   End i.88
   End i.87
   End i.86
   End i.85
   End i.84
   End i.83
   End i.82
   End i.81
   End i.80
   End i.79
   End i.78
   End i.77
   End i.76
   End i.75
   End i.74
   End i.73
   End i.72
   End i.71
   End i.70
   End i.69
   End i.68
   End i.67
   End i.66
   End i.65
   End i.64
   End i.63
   End i.62
   End i.61
   End i.60
   End i.59
   End i.58
   End i.57
   End i.56
   End i.55
   End i.54
   End i.53
   End i.52
   End i.51
   End i.50
   End i.49
   End i.48
   End i.47
   End i.46
   End i.45
   End i.44
   End i.43
   End i.42
   End i.41
   End i.40
   End i.39
   End i.38
   End i.37
   End i.36
   End i.35
   End i.34
   End i.33
   End i.32
   End i.31
   End i.30
   End i.29
   End i.28
   End i.27
   End i.26
   End i.25
   End i.24
   End i.23
   End i.22
   End i.21
   End i.20
   End i.19
   End i.18
   End i.17
   End i.16
   End i.15
   End i.14
   End i.13
   End i.12
   End i.11
   End i.10
   End i.9
   End i.8
   End i.7
   End i.6
   End i.5
   End i.4
   End i.3
   End i.2
   End i.1
   self~AssertSame(c, cc)

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
