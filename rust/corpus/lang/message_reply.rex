/* Message~reply and ~replyWith start a copy of the message, which they
   answer; the original stays reusable and takes any new receiver and
   arguments. */
m = .message~new('abc', 'length')
r = m~reply
say r~result r~hasResult r~hasError r~completed (r == m) m~hasResult
say m~reply~result
r = m~reply('aeiou')
say r~result r~target m~target
m = .message~new('abc', 'subchar', 'individual', 3)
r = m~reply(, 1)
say r~result r~arguments[1] m~arguments[1]
r = m~replyWith('def', .array~of(2))
say r~result r~target m~target m~arguments[1]
r = m~replyWith(, .array~of(3))
say r~result r~target
o = .k~new
gate = .message~new(o, 'open')
r = .message~new(o, 'late', 'i', gate)~reply
say 'before' r~hasResult r~completed
gate~send
say 'after' r~result r~hasResult r~completed
r = .message~new(o, 'boom')~reply
r~wait
say 'failed' r~hasError r~completed r~errorCondition~code
m~start
m~wait
signal on syntax
m~reply
exit
syntax:
  say 'reuse' condition('o')~code
::class k
::method open unguarded
  return 'opened'
::method late unguarded
  use arg gate
  gate~wait
  return 'late value'
::method boom unguarded
  return 1/0
