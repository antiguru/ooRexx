/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

/* This crate's numeric semantics as an executable model.

   Numbers are digit strings: a value is "sign digits exponent", the digits
   with no leading zero, zero as "+ 0 0". Every digit of an operand or a
   result is computed here one decimal digit at a time, so the host
   interpreter's own arithmetic decides nothing but counters, lengths and
   exponents, which the model keeps at NUMERIC DIGITS 20.

   The rules are rexxref's ("Numbers and Arithmetic", the numeric built-in
   functions, DO), with ANSI X3.274 sections 7.4, 8.3.6, 9.2.1 and 9.4 cited
   where they decide a point. Where the documentation leaves a point open,
   the model takes this crate's answer, and says so at the rule.

   numeric_eval answers one operation as "V[value]", "S code" for a SYNTAX
   condition, or "L[description]" for a LOSTDIGITS condition taken by a
   SIGNAL ON trap, which `trapped` says is set. */

::routine numeric_eval public
  use strict arg trapped, digits, fuzz, form, kind, name, count, args
  numeric digits 20
  m. = ''
  m.0trapped = trapped
  m.0digits = digits
  m.0fuzz = fuzz
  m.0form = translate(left(form, 1))
  m.0event = ''
  m.0count = count
  do i = 1 to max(count, 5)
    m.0given.i = args~hasIndex(i) & i <= count
    if m.0given.i then m.0arg.i = args[i]
  end
  select
    when kind == 'infix' then r = infix(name, m.0arg.1, m.0arg.2)
    when kind == 'prefix' then r = prefix(name, m.0arg.1)
    when kind == 'bif' then r = bif(name, 1)
    when kind == 'msg' then r = bif(name, 0)
    when kind == 'do' then r = do_count(name, m.0arg.1)
    otherwise return 'S model-kind' kind
  end
  if m.0event \== '' then return m.0event
  return 'V['r']'

/* ---- events ---- */

/* A SYNTAX condition ends the operation. */
fail: procedure expose m.
  if m.0event == '' then m.0event = 'S' arg(1)
  return ''

/* LOSTDIGITS: an operand of an arithmetic operation has more digits than
   the precision it is carried out at (rexxref "LOSTDIGITS"; leading zeros
   are not counted and trailing zeros are). Only a SIGNAL ON trap takes it;
   otherwise the operation goes on. */
lost: procedure expose m.
  parse arg n, k, text
  parse var n . d .
  if length(d) > k & m.0trapped then
    if m.0event == '' then m.0event = 'L['text']'
  return

/* ---- numbers as digit strings ---- */

/* The value of `t`, or '' when it is not a number (rexxref "Numbers"):
   blanks or tabs around, an optional sign followed by optional blanks,
   digits with at most one period, and an optional exponent. An exponent
   written or implied outside -999999999..999999999 makes no number. */
num: procedure expose m.
  t = strip(translate(arg(1), ' ', '09'x))
  s = '+'
  c = left(t, 1)
  if c == '+' | c == '-' then do
    if c == '-' then s = '-'
    t = strip(substr(t, 2), 'L')
  end
  p = pos('E', translate(t))
  ex = 0
  if p > 0 then do
    mant = left(t, p - 1)
    et = substr(t, p + 1)
    es = ''
    if left(et, 1) == '+' | left(et, 1) == '-' then do
      es = left(et, 1)
      et = substr(et, 2)
    end
    if et == '' | verify(et, '0123456789') > 0 then return ''
    et = strip(et, 'L', '0')
    if et == '' then et = 0
    if length(et) > 9 then return ''
    ex = et
    if es == '-' then ex = -ex
  end
  else mant = t
  if verify(mant, '0123456789.') > 0 | countstr('.', mant) > 1 then return ''
  parse var mant int '.' frac
  d = int || frac
  if d == '' then return ''
  e = ex - length(frac)
  d = strip(d, 'L', '0')
  if d == '' then return '+ 0 0'
  if e + length(d) - 1 > 999999999 | e < -999999999 then return ''
  return s d e

iszero: procedure
  parse arg . d .
  return d == '0'

negate: procedure
  parse arg s d e
  if d == '0' then return s d e
  if s == '+' then return '-' d e
  return '+' d e

/* `n` rounded to `k` significant digits, half up (rexxref "Precision"). */
round: procedure expose m.
  parse arg s d e, k
  if length(d) <= k then return s d e
  e = e + length(d) - k
  keep = left(d, k)
  if substr(d, k + 1, 1) >= 5 then do
    keep = inc(keep)
    if length(keep) > k then do
      keep = left(keep, k)
      e = e + 1
    end
  end
  return s keep e

/* `n` cut to its first `k` digits, the DIGITS + 1 every operand is
   truncated to (rexxref "Arithmetic Operators"). */
truncto: procedure
  parse arg s d e, k
  if length(d) <= k then return s d e
  return s left(d, k) (e + length(d) - k)

/* The digit string `d` plus one. */
inc: procedure
  parse arg d
  out = ''
  carry = 1
  do i = length(d) to 1 by -1
    v = substr(d, i, 1) + carry
    if v == 10 then do
      out = '0' || out
      carry = 1
    end
    else do
      out = v || out
      carry = 0
    end
  end
  if carry then out = '1' || out
  return out

/* Magnitude order of two digit strings: -1, 0 or 1. */
cmpmag: procedure
  parse arg x, y
  x = strip(x, 'L', '0')
  y = strip(y, 'L', '0')
  if length(x) \== length(y) then return sign(length(x) - length(y))
  if x == y then return 0
  if x >> y then return 1
  return -1

/* x + y, digit by digit from the right. */
addmag: procedure
  parse arg x, y
  n = max(length(x), length(y))
  x = right(x, n, '0')
  y = right(y, n, '0')
  out = ''
  carry = 0
  do i = n to 1 by -1
    v = substr(x, i, 1) + substr(y, i, 1) + carry
    carry = v % 10
    out = (v // 10) || out
  end
  if carry > 0 then out = carry || out
  return out

/* x - y for x not smaller than y, keeping x's width: a borrow leaves its
   leading zeros in place, where rounding counts them. */
submag: procedure
  parse arg x, y
  n = length(x)
  y = right(y, n, '0')
  out = ''
  borrow = 0
  do i = n to 1 by -1
    v = substr(x, i, 1) - substr(y, i, 1) - borrow
    if v < 0 then do
      v = v + 10
      borrow = 1
    end
    else borrow = 0
    out = v || out
  end
  return out

/* x * y, long multiplication, with no leading zero. */
mulmag: procedure
  parse arg x, y
  lx = length(x)
  ly = length(y)
  col. = 0
  do i = 1 to lx
    a = substr(x, lx - i + 1, 1)
    if a == 0 then iterate
    do j = 1 to ly
      k = i + j - 1
      col.k = col.k + a * substr(y, ly - j + 1, 1)
    end
  end
  out = ''
  carry = 0
  do k = 1 to lx + ly
    v = col.k + carry
    out = (v // 10) || out
    carry = v % 10
  end
  out = strip(out, 'L', '0')
  if out == '' then out = '0'
  return out

/* An arithmetic result outside the exponent range is error 42.901 or
   42.902 (rexxref "Arithmetic Errors"). */
inrange: procedure expose m.
  parse arg s d e
  if d == '0' then return 1
  if e + length(d) - 1 > 999999999 then do
    call fail '42.901'
    return 0
  end
  if e < -999999999 then do
    call fail '42.902'
    return 0
  end
  return 1

/* ---- rendering ---- */

/* `n` as a result is written at precision `k` (rexxref "Numbers and
   Arithmetic"): exponential when the places before the point exceed `k`
   or the places after it exceed twice `k`, SCIENTIFIC with one digit before
   the point, ENGINEERING with an exponent that is a multiple of three. */
render: procedure expose m.
  parse arg n, k
  n = round(n, k)
  parse var n s d e
  if d == '0' then return '0'
  a = e + length(d) - 1
  if a >= k | (a < 0 & -e > 2 * k) then do
    x = group(a)
    body = plain(s (d) (e - x), '')
    if x == 0 then return body
    if x < 0 then return body || 'E-' || -x
    return body || 'E+' || x
  end
  return plain(n, '')

/* The exponent the form groups an adjusted exponent `a` into. */
group: procedure expose m.
  a = arg(1)
  if m.0form == 'S' then return a
  q = a % 3
  if q * 3 \= a & a < 0 then q = q - 1
  return q * 3

/* `n` in plain notation; with `after`, exactly that many decimals (`n` is
   already rounded to them), and with 0 no point at all. */
plain: procedure
  parse arg s d e, after
  len = length(d)
  point = len + e
  if e >= 0 then do
    int = d || copies('0', e)
    dec = ''
    has = 0
  end
  else if point > 0 then do
    int = left(d, point)
    dec = substr(d, point + 1)
    has = 1
  end
  else do
    int = '0'
    dec = copies('0', -point) || d
    has = 1
  end
  if after \== '' then do
    if after = 0 then has = 0
    else do
      dec = dec || copies('0', after - length(dec))
      has = 1
    end
  end
  out = int
  if has then out = out'.'dec
  if s == '-' & d \== '0' then out = '-'out
  return out

/* `n` with every digit it holds, the description LOSTDIGITS gives a MAX or
   MIN argument and a DO count: exponential when the places before the point
   reach `k` or the exponent is beyond twice `k`. The documentation does not
   fix this rendering, and the model takes this crate's. */
unrounded: procedure expose m.
  parse arg s d e, k
  if d == '0' then return '0'
  sg = ''
  if s == '-' then sg = '-'
  if e = 0 then return sg || d
  len = length(d)
  a = e + len - 1
  shown = e
  suffix = ''
  if a >= k | abs(e) > 2 * k then do
    if m.0form == 'E' then do
      if a < 0 then a = a - 2
      a = (a % 3) * 3
    end
    shown = e - a
    if a < 0 then suffix = 'E-' || -a
    if a > 0 then suffix = 'E+' || a
  end
  width = shown + len
  if width <= 0 then body = '0.' || copies('0', -width) || d
  else if width >= len then body = d || copies('0', width - len)
  else body = left(d, width)'.'substr(d, width + 1)
  return sg || body || suffix

/* ---- the operators (rexxref "Arithmetic Operators") ---- */

/* a + b, or a - b with `neg`, at precision `k`. Each operand is truncated to
   k + 1 digits. Aligned, the two must span no more than k + 1 places, so the
   less significant one loses digits from its low end; then the sum is
   rounded to `k` places counted from the leftmost place the aligned
   operands occupy. That is how rexxref's own FUZZ example comes out:
   4.9999 = 5 at DIGITS 5, FUZZ 1 subtracts at 4 digits and the difference
   rounds to zero. */
add: procedure expose m.
  parse arg a, b, neg, k
  if neg then b = negate(b)
  a = truncto(a, k + 1)
  b = truncto(b, k + 1)
  parse var a sa da ea
  parse var b sb db eb
  minexp = min(ea, eb)
  adja = ea - minexp
  adjb = eb - minexp
  la = length(da)
  lb = length(db)
  fast = ''
  if da == '0' then fast = b
  else if db == '0' then fast = a
  else if adja + la > lb + k then fast = a
  else if adjb + lb > la + k then fast = b
  if fast \== '' then do
    fast = round(fast, k)
    if \inrange(fast) then return ''
    return fast
  end
  over_a = la + adja - (k + 1)
  over_b = lb + adjb - (k + 1)
  if over_a > 0 | over_b > 0 then do
    adjust = max(over_a, over_b)
    if adja \== 0 then do
      taken = min(adjust, adja)
      db = droplow(db, taken)
      eb = eb + taken
      adjust = adjust - taken
    end
    else if adjb \== 0 then do
      taken = min(adjust, adjb)
      da = droplow(da, taken)
      ea = ea + taken
      adjust = adjust - taken
    end
    if adjust \== 0 then do
      da = droplow(da, adjust)
      ea = ea + adjust
      db = droplow(db, adjust)
      eb = eb + adjust
    end
  end
  minexp = min(ea, eb)
  xa = da || copies('0', ea - minexp)
  xb = db || copies('0', eb - minexp)
  if sa == sb then do
    raw = addmag(xa, xb)
    s = sa
  end
  else do
    order = cmpmag(xa, xb)
    if order == 0 then return '+ 0 0'
    if order > 0 then do
      raw = submag(xa, xb)
      s = sa
    end
    else do
      raw = submag(xb, xa)
      s = sb
    end
  end
  r = round(s raw minexp, k)
  parse var r s d e
  d = strip(d, 'L', '0')
  if d == '' then return '+ 0 0'
  if \inrange(s d e) then return ''
  return s d e

/* The low `count` digits of `d` dropped, never all of them. */
droplow: procedure
  parse arg d, count
  keep = max(length(d) - max(count, 0), 1)
  return left(d, keep)

/* a * b at precision `k`: long multiplication of the operands truncated to
   k + 1 digits, cut to k + 1 digits, then rounded. */
mul: procedure expose m.
  parse arg a, b, k
  a = truncto(a, k + 1)
  b = truncto(b, k + 1)
  parse var a sa da ea
  parse var b sb db eb
  if da == '0' | db == '0' then return '+ 0 0'
  p = mulmag(da, db)
  extra = 0
  if length(p) > k then do
    extra = length(p) - (k + 1)
    p = left(p, k + 1)
  end
  s = '+'
  if sa \== sb then s = '-'
  r = round(s p (ea + eb + extra), k)
  if \inrange(r) then return ''
  return r

/* The quotient digits of n / d, one long-division step at a time: the
   digits of floor(n * 10**shift / d) for the smallest shift that gives
   `want` digits or divides exactly. Answers "digits shift". */
longdiv: procedure
  parse arg n, d, want
  q = ''
  rem = ''
  shift = 0
  i = 0
  do while length(q) < want
    i = i + 1
    if i <= length(n) then next = substr(n, i, 1)
    else do
      next = 0
      shift = shift + 1
    end
    rem = strip(rem || next, 'L', '0')
    count = 0
    do while cmpmag(rem, d) >= 0
      rem = strip(submag(right(rem, max(length(rem), length(d)), '0'), d), 'L', '0')
      count = count + 1
    end
    if q == '' & count == 0 then iterate
    q = q || count
    if rem == '' & i >= length(n) then leave
  end
  return q shift

/* a / b, a % b or a // b at precision `k` (rexxref "Arithmetic Operators",
   "Integer Division", "Remainder"). */
div: procedure expose m.
  parse arg a, b, k, op
  parse var b . db .
  if db == '0' then return fail('42.3')
  parse var a sa da ea
  if da == '0' then return '+ 0 0'
  a = truncto(a, k + 1)
  b = truncto(b, k + 1)
  parse var a sa da ea
  parse var b sb db eb
  s = '+'
  if sa \== sb then s = '-'
  calc = ea - eb + length(da) - length(db)
  if calc < 0 & op \== '/' then do
    if op == '%' then return '+ 0 0'
    return a
  end
  parse value longdiv(da, db, k + 1) with q shift
  qe = ea - eb - shift
  if op == '/' then do
    r = round(s q qe, k)
    if \inrange(r) then return ''
    parse var r s d e
    do while length(d) > 1 & right(d, 1) == '0'
      d = left(d, length(d) - 1)
      e = e + 1
    end
    return s d e
  end
  if qe < 0 then do
    if -qe >= length(q) then q = '0'
    else q = left(q, length(q) + qe)
  end
  q = strip(q, 'L', '0')
  if q == '' then q = '0'
  qe = max(qe, 0)
  if q \== '0' & length(q) + qe > k then do
    if op == '%' then return fail('26.11')
    return fail('26.12')
  end
  if op == '%' then do
    if q == '0' then return '+ 0 0'
    return s q qe
  end
  /* The remainder, exactly: a less the integer quotient times b, signed as
     the dividend. */
  minexp = min(ea, eb + qe)
  xa = da || copies('0', ea - minexp)
  xp = mulmag(q, db) || copies('0', eb + qe - minexp)
  if cmpmag(xa, xp) == 0 then return '+ 0 0'
  r = strip(submag(xa, right(xp, length(xa), '0')), 'L', '0')
  return round(sa r minexp, k)

/* `n` rounded to `k` digits as a whole number of at most `k` digits, and
   never more than 18, or '' when it is not one (rexxref "Power", "Whole
   Numbers"). */
whole: procedure expose m.
  parse arg n, k
  n = round(n, k)
  parse var n s d e
  if d == '0' then return '0'
  if length(d) + e > min(k, 18) then return ''
  if e < 0 then do
    if -e >= length(d) then return ''
    if verify(right(d, -e), '0') > 0 then return ''
    d = left(d, length(d) + e)
  end
  else d = d || copies('0', e)
  if s == '-' then return '-'d
  return d

/* a ** b at precision `k`: exponentiation by squaring at k plus the power's
   own width plus one digits, inverted for a negative power, rounded to `k`
   and stripped of trailing zeros (rexxref "Power"). */
pow: procedure expose m.
  parse arg a, b, k
  p = whole(b, k)
  if p == '' then return fail('26.8')
  negp = left(p, 1) == '-'
  if negp then p = substr(p, 2)
  a = truncto(a, k + 1)
  parse var a sa da ea
  if da == '0' then do
    if negp then return fail('42.903')
    if p == '0' then return '+ 1 0'
    return '+ 0 0'
  end
  if abs(ea + length(da) - 1) * p > 999999999 then return fail('42.1')
  if p == '0' then return '+ 1 0'
  work = k + length(p) + 1
  bits = ''
  v = p
  do while v > 0
    bits = (v // 2) || bits
    v = v % 2
  end
  acc = a
  do i = 2 to length(bits)
    acc = mul(acc, acc, work)
    if acc == '' then return ''
    if substr(bits, i, 1) == 1 then do
      acc = mul(acc, a, work)
      if acc == '' then return ''
    end
  end
  if negp then do
    parse var acc sc dc ec
    parse value longdiv('1', dc, work + 1) with q shift
    acc = sc q (-ec - shift)
  end
  r = round(acc, k)
  if \inrange(r) then return ''
  parse var r s d e
  do while length(d) > 1 & right(d, 1) == '0'
    d = left(d, length(d) - 1)
    e = e + 1
  end
  return s d e

/* -1, 0 or 1: a against b as a numeric comparison, which is the sign of
   a - b subtracted at DIGITS minus FUZZ (rexxref "Numeric Comparisons"). */
compare: procedure expose m.
  parse arg a, b
  parse var a sa da .
  parse var b sb db .
  if da == '0' & db == '0' then return 0
  diff = add(a, b, 1, m.0digits - m.0fuzz)
  if diff == '' then return 0
  parse var diff s d .
  if d == '0' then return 0
  if s == '-' then return -1
  return 1

/* The LOSTDIGITS checks of a numeric comparison: each operand against
   DIGITS, then, under FUZZ, against DIGITS minus FUZZ, the precision the
   subtraction runs at, whatever the signs. */
compare_lost: procedure expose m.
  parse arg a, ta, b, tb
  call lost a, m.0digits, ta
  call lost b, m.0digits, tb
  if m.0fuzz > 0 then do
    call lost a, m.0digits - m.0fuzz, ta
    call lost b, m.0digits - m.0fuzz, tb
  end
  return

infix: procedure expose m.
  parse arg op, ta, tb
  k = m.0digits
  if wordpos(op, '== \== >> << >>= <<= \>> \<<') > 0 then
    return strictcmp(op, ta, tb)
  if wordpos(op, '= \= <> >< > < >= <= \> \<') > 0 then do
    a = num(ta)
    b = num(tb)
    if a == '' | b == '' then return stringcmp(op, ta, tb)
    call compare_lost a, ta, b, tb
    if m.0event \== '' then return ''
    return holds(op, compare(a, b))
  end
  a = num(ta)
  if a == '' then return fail('41.1')
  if op == '**' then do
    b = num(tb)
    if b == '' then return fail('26.8')
    call lost a, k, ta
    if m.0event \== '' then return ''
    r = pow(a, b, k)
  end
  else do
    b = num(tb)
    if b == '' then return fail('41.1')
    call lost a, k, ta
    call lost b, k, tb
    if m.0event \== '' then return ''
    select
      when op == '+' then r = add(a, b, 0, k)
      when op == '-' then r = add(a, b, 1, k)
      when op == '*' then r = mul(a, b, k)
      otherwise r = div(a, b, k, op)
    end
  end
  if m.0event \== '' then return ''
  return render(r, k)

/* Prefix + and - are 0 + a and 0 - a. */
prefix: procedure expose m.
  parse arg op, t
  a = num(t)
  if a == '' then return fail('41.1')
  call lost a, m.0digits, t
  if m.0event \== '' then return ''
  r = add('+ 0 0', a, op == '-', m.0digits)
  if m.0event \== '' then return ''
  return render(r, m.0digits)

holds: procedure
  parse arg op, order
  select
    when op == '=' then return order == 0
    when op == '\=' | op == '<>' | op == '><' then return order \== 0
    when op == '>' then return order > 0
    when op == '<' then return order < 0
    when op == '>=' | op == '\<' then return order >= 0
    otherwise return order <= 0
  end

/* A non-strict comparison of strings: blanks stripped, the shorter padded
   with blanks. */
stringcmp: procedure
  parse arg op, x, y
  x = strip(x)
  y = strip(y)
  n = max(length(x), length(y))
  x = left(x, n)
  y = left(y, n)
  order = 0
  if x >> y then order = 1
  if x << y then order = -1
  return holds(op, order)

/* A strict comparison of the strings as written. */
strictcmp: procedure
  parse arg op, x, y
  order = 0
  if x >> y then order = 1
  if x << y then order = -1
  select
    when op == '==' then return order == 0
    when op == '\==' then return order \== 0
    when op == '>>' then return order > 0
    when op == '<<' then return order < 0
    when op == '>>=' | op == '\<<' then return order >= 0
    otherwise return order <= 0
  end

/* ---- the numeric built-in functions and methods ---- */

/* A builtin's whole-number argument, read at the 18 digits every builtin
   argument is read at, or '' when it is not one. */
argwhole: procedure expose m.
  n = num(arg(1))
  if n == '' then return ''
  return whole(n, 18)

/* Argument `i` as a non-negative whole number, '' when omitted, after the
   40.12 its type raises. The 93.906 for a negative one is the caller's,
   since the target's own 93.943 comes between them. */
count_arg: procedure expose m.
  parse arg name, i
  if \m.0given.i then return ''
  w = argwhole(m.0arg.i)
  if w == '' then return fail('40.12')
  return w

bif: procedure expose m.
  parse arg name, function
  k = m.0digits
  select
    when name == 'MAX' | name == 'MIN' then return maxmin(name)
    when name == 'ABS' | name == 'SIGN' then do
      n = num(m.0arg.1)
      if n == '' then return fail('93.943')
      parse var n s d e
      if name == 'SIGN' then do
        if d == '0' then return 0
        if s == '-' then return -1
        return 1
      end
      return render('+' d e, k)
    end
    when name == 'TRUNC' then do
      places = count_arg(name, 2)
      if m.0event \== '' then return ''
      n = num(m.0arg.1)
      if n == '' then return fail('93.943')
      if left(places, 1) == '-' then return fail('93.906')
      if places == '' then places = 0
      return trunc(round(n, k), places)
    end
    when name == 'FORMAT' then do
      do i = 2 to 5
        w.i = count_arg(name, i)
        if m.0event \== '' then return ''
      end
      n = num(m.0arg.1)
      if n == '' then return fail('93.943')
      do i = 2 to 5
        if left(w.i, 1) == '-' then return fail('93.906')
      end
      return format(n, w.2, w.3, w.4, w.5)
    end
    when name == 'DATATYPE' then return datatype_w(m.0arg.1)
    otherwise return fail('model-bif' name)
  end

/* TRUNC: the rounded number cut, never rounded, to `places` decimals, in
   plain notation, with no sign on a zero (rexxref "TRUNC"). */
trunc: procedure expose m.
  parse arg n, places
  parse var n s d e
  if e < -places then do
    drop = -places - e
    if drop >= length(d) then return plain('+ 0' (-places), places)
    d = strip(left(d, length(d) - drop), 'L', '0')
    if d == '' then return plain('+ 0 0', places)
    e = -places
  end
  return plain(s d e, places)

/* `n` rounded to `places` decimals, half up. */
round_places: procedure
  parse arg s d e, places
  if e >= -places then return s d e
  drop = -places - e
  len = length(d)
  if drop > len then return '+ 0' (-places)
  keep = len - drop
  kept = left(d, keep)
  if substr(d, keep + 1, 1) >= 5 then kept = inc(kept)
  if kept == '' then return '+ 0' (-places)
  kept = strip(kept, 'L', '0')
  if kept == '' then return '+ 0 0'
  return s kept (-places)

/* FORMAT (rexxref "FORMAT"): the number rounded as by number + 0, then laid
   out in `before` and `after` places, exponential when the places its
   integer part needs exceed `expt` or the places its decimal part needs
   exceed twice `expt`. Rounding to `after` places can carry into a new
   leading digit, which moves the exponent; the layout is decided again
   until it settles. */
format: procedure expose m.
  parse arg n, before, after, expp, expt
  k = m.0digits
  n = round(n, k)
  if expp == 0 then trig = ''
  else if expt \== '' then trig = expt
  else trig = k
  parse var n s d e
  x0 = ''
  if trig \== '' then do
    a = e + length(d) - 1
    if a >= trig | -e > 2 * trig then x0 = group(a)
  end
  if expp \== '' & x0 \== '' then
    if length(abs(x0)) > expp then return fail('93.941')
  x = x0
  do 8
    shape = s d (e - zero_if_empty(x))
    if after \== '' then shape = round_places(shape, after)
    parse var shape . rd re
    adj = zero_if_empty(x) + re + length(rd) - 1
    nx = ''
    if trig \== '' & (x \== '' | adj >= trig) then nx = group(adj)
    if nx == x then leave
    x = nx
  end
  shape = s d (e - zero_if_empty(x))
  if after \== '' then shape = round_places(shape, after)
  body = plain(shape, after)
  if expp \== '' & x \== '' & x \== 0 then
    if length(abs(x)) > expp then return fail('93.941')
  if before \== '' then do
    int = body
    if pos('.', int) > 0 then int = left(int, pos('.', int) - 1)
    if length(int) > before then return fail('93.942')
    body = copies(' ', before - length(int)) || body
  end
  if x == '' then return body
  if x == 0 then do
    if expp == '' then return body
    return body || copies(' ', expp + 2)
  end
  ed = abs(x)
  if expp \== '' then ed = right(ed, max(expp, length(ed)), '0')
  if x < 0 then return body'E-'ed
  return body'E+'ed

zero_if_empty: procedure
  if arg(1) == '' then return 0
  return arg(1)

/* DATATYPE(x, 'W'): x + 0 at the digits in force is a whole number. The
   addition is an arithmetic operation, so it raises LOSTDIGITS. */
datatype_w: procedure expose m.
  parse arg t
  k = m.0digits
  n = num(t)
  if n == '' then return 0
  call lost n, k, t
  if m.0event \== '' then return ''
  r = add(n, '+ 0 0', 0, k)
  if m.0event \== '' then return ''
  parse var r s d e
  if d == '0' | e = 0 then return 1
  a = length(d) + e
  if a > k | a <= 0 then return 0
  if e > 0 then return 1
  return verify(substr(d, a + 1), '0') == 0

/* MAX and MIN, as functions and as messages alike (Deviation 44): the
   target and every argument rounded to DIGITS, compared as numbers, FUZZ
   included, an earlier value kept on a tie. The target raises no
   LOSTDIGITS (Phase 9d plan, Ruled 10). An
   argument does when it has more digits than DIGITS, unless every value
   is written as a whole number and the target is valid under DIGITS
   (Deviation 44: decided by the written value). Under FUZZ each
   comparison raises it as any comparison does. */
maxmin: procedure expose m.
  parse arg name
  k = m.0digits
  if \m.0given.1 then return fail('40.5')
  t = num(m.0arg.1)
  if t == '' then return fail('93.943')
  quiet = integer_written(m.0arg.1)
  if quiet then quiet = valid_under(m.0arg.1, k)
  do i = 2 to m.0count while quiet
    if \m.0given.i then quiet = 0
    else quiet = integer_written(m.0arg.i)
  end
  best = round(t, k)
  do i = 2 to m.0count
    if \m.0given.i then return fail('40.5')
    c = num(m.0arg.i)
    if c == '' then return fail('93.904')
    if \quiet then do
      call lost c, k, unrounded(c, k)
      if m.0event \== '' then return ''
    end
    c = round(c, k)
    if \quiet & m.0fuzz > 0 then do
      call lost c, k - m.0fuzz, unrounded(c, k)
      call lost best, k - m.0fuzz, unrounded(best, k)
      if m.0event \== '' then return ''
    end
    order = compare(c, best)
    if name == 'MAX' & order > 0 then best = c
    if name == 'MIN' & order < 0 then best = c
  end
  return render(best, k)

/* Written as a whole number: an optional minus, then at most 18 digits
   with no leading zero. */
integer_written: procedure
  parse arg t
  if left(t, 1) == '-' then t = substr(t, 2)
  if t == '' | length(t) > 18 then return 0
  if verify(t, '0123456789') > 0 then return 0
  return \(left(t, 1) == '0' & length(t) > 1)

valid_under: procedure
  parse arg t, k
  if left(t, 1) == '-' then t = substr(t, 2)
  return length(t) <= min(k, 18)

/* ---- DO (rexxref "DO", ANSI X3.274 section 8.3.6) ---- */

/* A repetitor or FOR count is a whole number at the digits in force, zero
   or more, else 26.2 or 26.3 (Deviation 47). It is tested as DATATYPE(n,
   'W') tests, by rounding, so a count with more digits than DIGITS raises
   LOSTDIGITS first (ANSI X3.274 section 8.3.6), named by its number
   unrounded. Answers whether a loop of that count runs a first pass. */
do_count: procedure expose m.
  parse arg which, t
  code = '26.2'
  if which == 'for' then code = '26.3'
  n = num(t)
  if n == '' then return fail(code)
  call lost n, m.0digits, unrounded(n, m.0digits)
  if m.0event \== '' then return ''
  w = whole(n, m.0digits)
  if w == '' | left(w, 1) == '-' then return fail(code)
  if w == '0' then return 'none'
  return 'ran'
