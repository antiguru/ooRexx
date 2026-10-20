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

/* This crate's stream semantics as an executable model.

   A persistent stream is a string holding the file's bytes, with a read
   position and a write position kept apart (rexxref iostrms.xml:226-227,
   funct.xml:4305-4306). Nothing is buffered: every write is in the string
   when the operation ends, and every position is a character position into
   it. A transient stream is a queue: writes join its end, reads take from
   its front, and it has no positions (iostrms.xml:186-197, 243-245).

   The rules are rexxref's: iostrms.xml ("Input Streams", "Output Streams",
   "Line versus Character Positioning", "Implementation") and funct.xml
   (CHARIN, CHAROUT, CHARS, LINEIN, LINEOUT, LINES, STREAM), with the
   programming guide's record passages (rexxpg rexio.xml:562-565, 613-616)
   and Deviations 46, 48 and 49 of docs/superpowers/plans/phase-4-exclusions.txt.
   ANSI X3.274 section 9.7's properties hold of it: a stream read from
   position 1 returns what was written there, and a repositioned write loses
   only what it overwrites. Where the documentation leaves a point open, the
   model takes this crate's answer and says so at the rule, naming the
   function in rust/crates/rexx-exec/src/dispatch/stream.rs and its line.

   stream_model runs one scenario. `initial` is the file's content before
   it, `transient` says the stream is a FIFO, and `ops` is an array of
   operations in the grammar the driver test (tests/stream_model.rs)
   describes. It answers one item per operation and then the final content,
   as the driver's native run answers them. */

::routine stream_model public
  use strict arg initial, transient, ops
  numeric digits 18
  m. = ''
  m.0data = initial
  m.0transient = transient
  m.0open = 0
  m.0state = 'UNKNOWN'
  out = ''
  do i = 1 to ops~items
    op = ops[i]
    r = operation(op)
    out = out || op~changestr(' ', '_') || '=' || show(r) m.0state query('WRITE') query('READ') || ';'
  end
  if transient then return out '[-]'
  return out '[' || show(m.0data) || ']'

/* ---- the operations ---- */

operation: procedure expose m.
  parse arg code a b .
  select
    when code == 'O' then return open(subword(arg(1), 2))
    when code == 'X' then return close()
    when code == 'CX' | code == 'LX' then return close_by_write()
    when code == 'CO' then return charout(1, a, b)
    when code == 'CE' then return charout(1, '', '')
    when code == 'CP' then return charout(0, '', a)
    when code == 'LO' then return lineout(1, a, b)
    when code == 'LE' then return lineout(1, '', '')
    when code == 'LP' then return lineout(0, '', a)
    when code == 'CI' then do
      if a == '' then a = 1
      return charin(b, a)
    end
    when code == 'LI' then do
      if b == '' then b = 1
      return linein(a, b)
    end
    when code == 'LZ' then return linein('', 0)
    when code == 'CH' then return chars()
    when code == 'LN' then return lines('N')
    when code == 'LC' then return lines('C')
    when code == 'S' then return seek(subword(arg(1), 2))
    when code == 'Q' then return query_position(subword(arg(1), 2))
    otherwise return 'model-op' code
  end

/* STREAM OPEN (funct.xml:4288-4358). BOTH is the default and keeps two
   positions; READ has no write position, which QUERY POSITION WRITE
   answers as 0 (this crate's `open`, :633). The write position starts at the end
   (funct.xml:4309-4310) and the read position at the start
   (iostrms.xml:183-184). REPLACE empties the stream (funct.xml:4315-4317).
   BINARY without RECLENGTH makes the whole stream one record
   (funct.xml:4354-4357). A second open reopens. */
open: procedure expose m.
  parse upper arg words
  m.0mode = 'BOTH'
  m.0append = 0
  m.0record = 0
  binary = 0
  do while words \== ''
    parse var words w words
    select
      when abbrev('READ', w, 3) then m.0mode = 'READ'
      when abbrev('WRITE', w, 1) then m.0mode = 'WRITE'
      when abbrev('BOTH', w, 2) then m.0mode = 'BOTH'
      when abbrev('APPEND', w, 2) then m.0append = 1
      when abbrev('REPLACE', w, 3) then m.0data = ''
      when abbrev('BINARY', w, 2) then binary = 1
      when abbrev('RECLENGTH', w, 3) then parse var words m.0record words
      when abbrev('NOBUFFER', w, 3) then nop
      otherwise return 'model-open' w
    end
  end
  if binary & m.0record = 0 then m.0record = length(m.0data)
  m.0open = 1
  m.0state = 'READY'
  if m.0transient then return 'READY:'
  m.0read = 1
  if m.0mode == 'READ' then m.0write = 0
  else m.0write = length(m.0data) + 1
  return 'READY:'

/* The implicit open of an operation on a closed stream: for reading and
   writing, positions as an explicit BOTH open sets them (iostrms.xml:527-530). */
implicit: procedure expose m.
  if m.0open then return
  m.0mode = 'BOTH'
  m.0append = 0
  m.0record = 0
  m.0open = 1
  m.0read = 1
  m.0write = length(m.0data) + 1
  return

/* STREAM CLOSE (funct.xml:4377-4381): READY: for an open stream, the null
   string for one that never was. A close writes nothing (Deviation 46). */
close: procedure expose m.
  was = m.0open
  m.0open = 0
  m.0state = 'UNKNOWN'
  if was then return 'READY:'
  return ''

/* CHAROUT or LINEOUT with neither string nor position closes the stream
   and answers 0 (funct.xml:1416-1418, 3214-3217). It writes nothing, on a
   record stream too (Deviation 46 as extended). */
close_by_write: procedure expose m.
  call close
  return 0

/* Which positions the open mode admits. A stream opened WRITE is write-only
   (funct.xml:4302): it reads nothing. */
readable: procedure expose m.
  return m.0mode \== 'WRITE'

writable: procedure expose m.
  return m.0mode \== 'READ'

/* A position given to a read or write: a positive whole number, else 93.907,
   and none at all on a transient stream, 93.958 (funct.xml:1318-1322,
   1405-1407, 3088-3091, 3201-3205). */
position_arg: procedure expose m.
  if m.0transient then return 'S93.958'
  if arg(1) < 1 then return 'S93.907'
  return ''

/* A write that cannot happen on this stream: a READ stream raises NOTREADY
   with ERROR, and CHAROUT answers what it could not write
   (funct.xml:1423-1426), LINEOUT 1 (funct.xml:3222-3226). */
unwritable: procedure expose m.
  m.0state = 'ERROR'
  return arg(1)

/* Where a write from position `p` lands. On an APPEND stream every write
   lands at the end: the write position cannot move into the stream as it
   was opened (funct.xml:4309-4312), and what the stream's own writes added
   is kept as well (the documentation is silent; the oracle's answer, and
   this crate's `landing`, :1670). */
landing: procedure expose m.
  if m.0append then return length(m.0data) + 1
  return arg(1)

/* What is left of the record holding position `p`. A position below 1,
   which only a SEEK reaches, counts as Rust's `rem_euclid` does (the
   documentation is silent; this crate's `record_remainder`, :1285). */
room: procedure expose m.
  parse arg p
  r = (p - 1) // m.0record
  if r < 0 then r = r + m.0record
  return m.0record - r

/* Writes `bytes` at character `p` of the stream: a write beyond the end
   leaves the gap as NUL characters (the documentation is silent; this
   crate's file system answer). A position below 1 transfers from
   character 1 (the documentation is silent; this crate's `offset_of`, :800). */
put: procedure expose m.
  parse arg bytes, p
  if m.0transient then do
    m.0data = m.0data || bytes
    return
  end
  q = max(p, 1)
  if q - 1 > length(m.0data) then m.0data = m.0data || copies('00'x, q - 1 - length(m.0data))
  m.0data = overlay(bytes, m.0data, q)
  return

/* The bytes from character `p`, at most `k` of them. A transient stream
   gives its first `k` and drops them. */
take: procedure expose m.
  parse arg p, k
  if m.0transient then do
    got = left(m.0data, min(k, length(m.0data)))
    m.0data = substr(m.0data, length(got) + 1)
    return got
  end
  q = max(p, 1)
  if q > length(m.0data) then return ''
  return substr(m.0data, q, min(k, length(m.0data) - q + 1))

/* CHAROUT (funct.xml:1388-1418): writes at the write position, or at
   `start` once the write position is set to it, and moves the write
   position by what was written (funct.xml:1399-1402). With no string it
   only positions and answers 0. */
charout: procedure expose m.
  parse arg given, s, start
  call implicit
  if \writable() then do
    if given then return unwritable(length(s))
    return unwritable(0)
  end
  m.0state = 'READY'
  if start \== '' then do
    bad = position_arg(start)
    if bad \== '' then return bad
    m.0write = start
  end
  if \given then return 0
  if m.0transient then do
    call put s
    return 0
  end
  at = landing(m.0write)
  call put s, at
  m.0write = at + length(s)
  return 0

/* LINEOUT (funct.xml:3176-3226): `line` sets the write position to the
   start of that line first; with no string nothing more is written and the
   answer is 0. A string is written with a line end after it, and the write
   position moves to the start of the next line. On a record stream the
   string completes its record: it is padded with blanks to the record's
   end (rexxpg rexio.xml:562-565), and a string longer than what is left of
   the record is the bare 93 (this crate's `write_record`, :1637; the documentation
   is silent). Only this pads (Deviation 46 as extended). A text stream
   ending in a ctrl-Z has it overwritten by a line written at the end
   (this crate's `CTRL_Z`, :787). */
lineout: procedure expose m.
  parse arg given, s, line
  call implicit
  if \writable() then return unwritable(1)
  m.0state = 'READY'
  if line \== '' then do
    bad = position_arg(line)
    if bad \== '' then return bad
    m.0write = line_start(line)
  end
  if \given then return 0
  if m.0transient then do
    call put s || '0a'x
    return 0
  end
  at = landing(m.0write)
  if m.0record > 0 then do
    k = room(at)
    if length(s) > k then return 'S93.0'
    call put left(s, k), at
    m.0write = at + k
    return 0
  end
  if at = length(m.0data) + 1 & right(m.0data, 1) == '1a'x then at = at - 1
  call put s || '0a'x, at
  m.0write = at + length(s) + 1
  return 0

/* CHARIN (funct.xml:1305-1335): `start` sets the read position, and one
   past the end of the stream is NOTREADY with the null string. A length of
   0 reads nothing (funct.xml:1325-1328). Otherwise it reads up to `length`
   characters and moves the read position by what it read. A read that gets
   none is NOTREADY in the ERROR state, one that gets some but fewer than
   asked in the NOTREADY state (the documentation names no state; this
   crate's `charin`, :1133). A write-only stream reads nothing, in the ERROR
   state (this crate's answer for a descriptor it could open only for
   writing). */
charin: procedure expose m.
  parse arg start, k
  call implicit
  m.0state = 'READY'
  if start \== '' then do
    bad = position_arg(start)
    if bad \== '' then return bad
    if start > length(m.0data) then do
      m.0state = 'NOTREADY'
      return ''
    end
    m.0read = start
  end
  if k = 0 then return ''
  if \readable() then do
    m.0state = 'ERROR'
    return ''
  end
  got = take(m.0read, k)
  if got == '' then do
    m.0state = 'ERROR'
    return ''
  end
  if \m.0transient then m.0read = m.0read + length(got)
  if length(got) < k then m.0state = 'NOTREADY'
  return got

/* LINEIN (funct.xml:3072-3103): `line` sets the read position to the start
   of that line (as SEEK ... LINE does), a count of 0 reads nothing, and
   otherwise it reads to the next line end, which it drops, and the read
   position moves past it (funct.xml:3081-3087). A last line with no line
   end is still a line; at the end of the stream the answer is the null
   string and the state NOTREADY. On a record stream a line is what is left
   of the current record, short at the end as CHARIN is (this crate's
   `read_record`, :1302). A line end's preceding carriage return is dropped too
   (this crate's `read_line_from`, :840). */
linein: procedure expose m.
  parse arg line, count
  call implicit
  m.0state = 'READY'
  if line \== '' then do
    bad = position_arg(line)
    if bad \== '' then return bad
    if readable() then m.0read = line_start(line)
  end
  if count = 0 then return ''
  if \readable() then do
    m.0state = 'ERROR'
    return ''
  end
  if m.0record > 0 & \m.0transient then do
    left = room(m.0read)
    got = take(m.0read, left)
    m.0read = m.0read + length(got)
    if got == '' then m.0state = 'ERROR'
    else if length(got) < left then m.0state = 'NOTREADY'
    return got
  end
  if m.0transient then do
    e = pos('0a'x, m.0data)
    if e = 0 then e = length(m.0data)
    got = take(1, e)
  end
  else do
    q = max(m.0read, 1)
    if q > length(m.0data) then do
      m.0state = 'NOTREADY'
      return ''
    end
    e = pos('0a'x, m.0data, q)
    if e = 0 then e = length(m.0data)
    got = substr(m.0data, q, e - q + 1)
    /* A position below 1 reads from character 1 and lands as if it had
       been 0 (this crate's `read_line_from`, :840, reading from offset 0). */
    m.0read = max(m.0read, 0) + length(got)
  end
  if right(got, 1) == '0a'x then do
    got = left(got, length(got) - 1)
    if right(got, 1) == '0d'x then got = left(got, length(got) - 1)
  end
  return got

/* CHARS (funct.xml:1475-1487): the characters from the read position to
   the end, line ends included. A transient stream cannot be measured
   without reading it, so it answers 1 (iostrms.xml:189-193). A write-only
   stream has none. */
chars: procedure expose m.
  call implicit
  if m.0transient then return 1
  if \readable() then return 0
  return max(0, length(m.0data) - (m.0read - 1))

/* LINES (funct.xml:3295-3307): Normal answers 1 when data remains after the
   read position and 0 when none does; Count answers the lines that remain,
   a last line with no line end included, or on a record stream the
   records that remain, the current partial one included
   (iostrms.xml:465-472). A transient stream answers 1, as CHARS does. */
lines: procedure expose m.
  call implicit
  if m.0transient then return 1
  if \readable() then return 0
  if m.0read > length(m.0data) then return 0
  if arg(1) == 'N' then return 1
  if m.0record > 0 then
    return (length(m.0data) + m.0record - 1) % m.0record - (m.0read - 1) % m.0record
  /* Counting text lines reads the stream, so the stream is READY after it
     (the documentation is silent; the oracle's `readSetup`). */
  m.0state = 'READY'
  rest = substr(m.0data, max(m.0read, 1))
  n = countstr('0a'x, rest)
  if right(rest, 1) \== '0a'x then n = n + 1
  return n

/* ---- lines ---- */

/* The line ends in the stream. */
line_ends: procedure expose m.
  return countstr('0a'x, m.0data)

/* The character position line `n` starts at: after the line end of line
   n - 1 (iostrms.xml:476-483), or on a record stream character
   (n - 1) * reclength + 1, past the end too (iostrms.xml:465-472,
   funct.xml:3203-3205). A text line past the last line end is the end of
   the stream: LINEOUT may name the line after the last one
   (funct.xml:3203-3209), and nothing further (this crate's `seek_line`, :2167). */
line_start: procedure expose m.
  parse arg n
  if n <= 1 then return 1
  if m.0record > 0 then return (n - 1) * m.0record + 1
  p = 0
  do n - 1
    p = pos('0a'x, m.0data, p + 1)
    if p = 0 then return length(m.0data) + 1
  end
  return p + 1

/* The line a SEEK ... LINE to `n` reaches: `n`, or past the last line end
   the line after it (this crate's `seek_line`, :2167; the documentation is
   silent). */
line_reached: procedure expose m.
  parse arg n
  if n <= 1 then return 1
  if m.0record > 0 then return n
  return min(n, line_ends() + 1)

/* The line character `p` lies in: one more than the line ends before it
   (iostrms.xml:476-483), or on a record stream the record holding it. */
line_of: procedure expose m.
  parse arg p
  if m.0record > 0 then return (max(p, 1) - 1) % m.0record + 1
  q = min(max(p, 1), length(m.0data) + 1)
  return countstr('0a'x, left(m.0data, q - 1)) + 1

/* The stream's lines, for SEEK <n LINE: a last line with no line end
   counts, and on a record stream a partial last record counts. */
line_count: procedure expose m.
  if m.0record > 0 then return (length(m.0data) + m.0record - 1) % m.0record
  n = line_ends()
  if m.0data \== '' & right(m.0data, 1) \== '0a'x then n = n + 1
  return n

/* ---- positioning ---- */

/* STREAM SEEK or POSITION (funct.xml:4393-4455). `=` is from the start,
   `<` from the end, `+` and `-` from the position the command names
   (Deviation 49). READ or WRITE names the position; with neither, a stream
   opened for both sets both, from the read position (funct.xml:4395-4396;
   which position a relative move counts from is this crate's answer,
   `position`, :1989),
   and a READ or WRITE stream the one it has. CHAR is the default. The
   answer is the new position. The target is not range-checked: the
   documentation's "appropriate error message" has no form to model, and
   this crate answers the position reached whatever it is. A transient
   stream has no positions, 93.958 (iostrms.xml:194-197). */
seek: procedure expose m.
  parse upper arg spec
  if m.0transient then return 'S93.958'
  style = '='
  rd = 0
  wr = 0
  byline = 0
  off = ''
  spec = spec~changestr('=', ' = ')~changestr('<', ' < ')~changestr('+', ' + ')~changestr('-', ' - ')
  do while spec \== ''
    parse var spec w spec
    select
      when w == '=' | w == '<' | w == '+' | w == '-' then style = w
      when abbrev('READ', w, 1) then rd = 1
      when abbrev('WRITE', w, 1) then wr = 1
      when abbrev('CHAR', w, 1) then nop
      when abbrev('LINE', w, 1) then byline = 1
      otherwise off = w
    end
  end
  m.0state = 'READY'
  if \rd & \wr then do
    if m.0mode == 'READ' then rd = 1
    else if m.0mode == 'WRITE' then wr = 1
    else do
      rd = 1
      wr = 1
      m.0write = m.0read
    end
  end
  call implicit
  if style == '-' then off = -off
  if \byline then do
    if rd then current = m.0read
    else current = m.0write
    select
      when style == '=' then landed = off
      when style == '<' then landed = length(m.0data) - off + 1
      otherwise landed = current + off
    end
    if rd then m.0read = landed
    if wr then m.0write = landed
    return landed
  end
  /* Line positioning in a stream open only for writing answers 0 and moves
     nothing (funct.xml:4454-4455 asks for an error; this crate's answer,
     `position`, :1989). */
  if \readable() then return 0
  if rd then current = line_of(m.0read)
  else current = line_of(m.0write)
  select
    when style == '=' then target = off
    when style == '<' then target = line_count() - off
    otherwise target = current + off
  end
  target = max(target, 1)
  line = line_reached(target)
  at = line_start(target)
  if rd then m.0read = at
  if wr then m.0write = at
  return line

/* STREAM QUERY POSITION (funct.xml:4531-4569): the read position, or the
   write position, by character or by line. With neither READ nor WRITE it
   is the read position on a stream that can read and the write position on
   one that cannot (funct.xml:4542-4544). A closed stream answers the null
   string, and a transient one 1 (this crate's `query_position`, :2289). A READ
   stream's write position is 0, in lines too. */
query_position: procedure expose m.
  parse upper arg spec
  wr = \readable()
  byline = 0
  do while spec \== ''
    parse var spec w spec
    select
      when abbrev('READ', w, 1) then wr = 0
      when abbrev('WRITE', w, 1) then wr = 1
      when abbrev('CHAR', w, 1) then nop
      when abbrev('LINE', w, 1) then byline = 1
      otherwise return 'model-query' w
    end
  end
  if \m.0open then return ''
  if m.0transient then return 1
  if wr then p = m.0write
  else p = m.0read
  if \byline then return p
  if wr & p = 0 then return 0
  return line_of(p)

/* The character positions the driver prints after every operation. */
query: procedure expose m.
  return query_position(arg(1))

/* The rendering both runs share: a line end as `|`, NUL as `@`, a blank as `_`. */
::routine show public
  return arg(1)~changestr('0a'x, '|')~changestr('00'x, '@')~changestr(' ', '_')
