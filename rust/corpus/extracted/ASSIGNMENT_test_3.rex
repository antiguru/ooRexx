/* extracted from ASSIGNMENT::test_3 */
::routine main public
   /* most ways to assign '123' to compound variables */
   cnt.=0
   g_.=''
   c.=':-)'
   three=3
   aha=123
   c.1=123
   c.2='123'
   c.3=-' - 123 '
   c.4=abs('  -  123.000e+0000   ') /1 /* get rid of .000                */
   c.5='12'three
   c.6=100+23
   c.7=1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+,
       1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+,
       2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+2+,
       1+1+1+0
   Numeric Digits 3
   c.8=+122.5
   c.9=+122.5+0
   Numeric Digits 9
   c.10=left(+123.49999999999999999999,3)
   c.11=left(123456789,3)
   c.12=left(12,3,3)
   c.13=right(987654123,3)
   c.14=right(23,3,1)
   c.15=random(123,123)     /* 910922 PA removed trailing comma */
   c.16=reverse(321)
   c.17=substr(999123999,4,3)
   c.18=aha
   c.19=c.1
   ii=20;c.ii=c.2
   c.21=substr(substr(012345,2),1,3)
   c.22=123
   aerr=1
   c.23=aerr*123
   c.24=120+3
   c.25=130-7
   c.26=25*5-2
   c.27=246/2
   c.28=2**7-5
   c.29=abbrev(1234,123)+122
   c.30=abs(-00123.00) / 1             /* get rid of .00                 */
   address ''
   c.31='12'address()'3'
   c.33=123
   c.34=123
   c.35=bitand(123,'333333'x)
   c.36=bitor('010203'x,'303030'x)
   c.37=bitxor('010203'x,'303030'x)
   c.38=center(' 123  ',3)
   c.39=centre('  123  ',3)
   c.40=compare(copies('a',122),copies('a',122)'b')
   c.41=right(cd,3)
   c.42=copies('123',1)
   c.43=c2d('7B'x)
   c.44=right(c2x('0123'x),3)
   c.45=left(c2x('1230'x),3)
   c.46=123*datatype(123,'w')
   c.47=120+length(word(date(),2))
   c.48=delstr(delstr(012345,1,1),4)
   c.49=strip(delword(delword(0 123 4 5,1,1),2))
   Numeric Digits 123
   c.50=123
   c.51=d2c(3224115)
   c.52=d2x(291)
   c.53=errortext(0)123
   c.54=123
   c.55=format(000123.39999,3,0)
   Numeric Digits 124
   Numeric Fuzz   123
   c.56=123
   Numeric Fuzz   0
   Numeric Digits 9
   c.57=insert(2,13,1)
   c.58=insert('',13,1,1,2)
   c.59=lastpos('a',copies('a',123))
   c.60=length(copies('.',120+3))
   c.61=max(111,123,-123)
   c.62=min(999,123,100000)
   c.63=overlay(2,'153',2)
   c.64=pos('aa',copies('b',122)'aa')
   Signal Off Error
   Do While queued()>0; Pull .; End
   Do     123; Queue ' '; End
   c.65=queued()
   Do While queued()>0; Pull .; End
   c.66=random(1,1,1)+122
   c.67=reverse(21)3
   c.68=right(0099123,3)
   c.69=sign(123)+122
   c.70=sign(123)23
   c.71=-sign(-1)23
   c.72=123
   c.73=space('  1   2   3   ',0)
   c.74=strip('$$123$$$',,'$')
   c.75=substr('abc123123',4,3)
   c.76=subword('11 22 33 123 55',4,1)
   c.77=(symbol('C.1')='VAR')+122
   c.78=Length(Time())+115
   Trace   'O'
   c.79=(trace()='O')23
   c.80=translate('cba','132','cab')
   c.81=trunc(123.49999999999)
   ccc='aaa'; aaa=123
   c.82=value(ccc)
   c.83=12''verify('ABXC','ABC')
   c.84=word(copies('12 ',122)' 123',123)
   c.85=wordindex(copies('1',121)' 2',2)
   c.86=wordlength(copies('1',122)' 'copies('2',123)' **',2)
   c.87=wordpos('abc',copies('1 ',122)' abc def ghi')
   c.88=12''words(' a  b   c     ')
   c.89=xrange('1','3')
   c.90=x2c(strip(strip("'313233'x",,'x'),,"'"))
   c.91=x2d('7B')
   Parse Value '123' With c.92 1 c.93 1 c.94 1 c.95 1 c.96
   Queue '123'
   Pull c.97
   c.98=result
   c.99=result
   c.100=246%2
   c.101=1122//999
   c.102=-(-1*123)
   c.103=+.123e+3
   c.104=1''2''3
   c.105=1||2||3
   c.106=123.0000000000000000000005/(122<123)
   Do id=1 By 1 While c.id<>':-)'
     If c.id<>123 Then Do
       Say 'id='id '->' c.id
       End
     self~assertSame(c.id, 123)
     End

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
