/* extracted from CONCATENATION::test_3 */
::routine main public
   a="abcdefg"'01'x
   b="abcdefgh"
   nb="abcdefgh"
   c="abcdefg "
   d=" abcdefg"
   e='01'x"abcdefg"
   f=" abcdefg "
   g='01'x"abcdefg"'01'x
   self~assertSame(b b, 'abcdefgh abcdefgh')
   self~assertSame(b || b, 'abcdefghabcdefgh')
   self~assertSame(b "" b, 'abcdefgh  abcdefgh')
   self~assertSame(b""nb, 'abcdefghabcdefgh')
   self~assertSame('Test the concatenation of "'a'" with "'a'" : "'a || a'"', 'Test the concatenation of "abcdefg" with "abcdefg" : "abcdefgabcdefg"')
   self~assertSame('Test the concatenation of "'a'" with "'nb'" : "'a || b'"', 'Test the concatenation of "abcdefg" with "abcdefgh" : "abcdefgabcdefgh"')
   self~assertSame('Test the concatenation of "'a'" with "'c'" : "'a || c'"', 'Test the concatenation of "abcdefg" with "abcdefg " : "abcdefgabcdefg "')
   self~assertSame('Test the concatenation of "'a'" with "'d'" : "'a || d'"', 'Test the concatenation of "abcdefg" with " abcdefg" : "abcdefg abcdefg"')
   self~assertSame('Test the concatenation of "'a'" with "'e'" : "'a || e'"', 'Test the concatenation of "abcdefg" with "abcdefg" : "abcdefgabcdefg"')
   self~assertSame('Test the concatenation of "'a'" with "'f'" : "'a || f'"', 'Test the concatenation of "abcdefg" with " abcdefg " : "abcdefg abcdefg "')
   self~assertSame('Test the concatenation of "'a'" with "'g'" : "'a || g'"', 'Test the concatenation of "abcdefg" with "abcdefg" : "abcdefgabcdefg"')
   self~assertSame(a b c d e f g, 'abcdefg abcdefgh abcdefg   abcdefg abcdefg  abcdefg  abcdefg')
   self~assertSame(a || b || c || d || e || f || g, 'abcdefgabcdefghabcdefg  abcdefgabcdefg abcdefg abcdefg')
   self~assertSame(a || b c || d e || f g, 'abcdefgabcdefgh abcdefg  abcdefg abcdefg abcdefg  abcdefg')
   self~assertSame(a "" b "" c "" d "" e "" f "" g, 'abcdefg  abcdefgh  abcdefg    abcdefg  abcdefg   abcdefg   abcdefg')
   self~assertSame(a""nb""c""d""e""f""g, 'abcdefgabcdefghabcdefg  abcdefgabcdefg abcdefg abcdefg')
   self~assertSame(a""nb||c "" d""e||f  g, 'abcdefgabcdefghabcdefg    abcdefgabcdefg abcdefg  abcdefg')
   self~assertSame((a) (b) (c) (d) (e) (f) (g), 'abcdefg abcdefgh abcdefg   abcdefg abcdefg  abcdefg  abcdefg')
   self~assertSame((a)(b)(c)(d)(e)(f)(g), 'abcdefgabcdefghabcdefg  abcdefgabcdefg abcdefg abcdefg')
   self~assertSame(( a )( b )( c )( d )( e )( f )( g ), 'abcdefgabcdefghabcdefg  abcdefgabcdefg abcdefg abcdefg')
   self~assertSame(a 1+1 b 1+2 c 1+3 d 1+4 e 1+5 f 1+6 g, 'abcdefg 2 abcdefgh 3 abcdefg  4  abcdefg 5 abcdefg 6  abcdefg  7 abcdefg')
   self~assertSame(a (1+1) b (1+2) c (1+3) d (1+4) e (1+5) f (1+6) g, 'abcdefg 2 abcdefgh 3 abcdefg  4  abcdefg 5 abcdefg 6  abcdefg  7 abcdefg')
   self~assertSame((a=a) (a=b) (a=c) (a=d) (a=e) (a=f) (a=g), '1 0 0 0 0 0 0')
   self~assertSame((a=a)(a=b)(a=c)(a=d)(a=e)(a=f)(a=g), '1000000')


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
