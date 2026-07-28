/* Keywords are not reserved words. Every instruction keyword is also a legal
   variable name, so a symbol is a keyword only by *position* -- first token of
   a clause that is not an assignment. A scanner that classifies keywords
   lexically cannot produce this program's output. */

/* Each of these is an assignment, because the second token is `=`. */
if = 2
do = 3
say = 4
end = 5
parse = 6
then = 7
call = 8
select = 9
when = 10
otherwise = 11
return = 12
exit = 13
signal = 14
trace = 15
numeric = 16

say if do say end parse then
say call select when otherwise
say return exit signal trace numeric

/* The same spelling as keyword and as variable in one clause. */
if if = 2 then say "if-as-keyword saw if-as-variable =" if
                 else say "wrong branch"

/* DO still loops while `do` holds a value. */
do i = 1 to 2
  say "iteration" i "and do =" do
end

/* SELECT/WHEN/OTHERWISE while all three are variables. */
select
  when when = 10 then say "when-as-keyword saw when =" when
  otherwise say "otherwise"
end

/* A stem whose name is a keyword. */
end. = 0
end.1 = 7
say "end.1 =" end.1

/* Keyword as a compound tail, and as a function-call target spelling. */
stem. = 0
stem.if = 99
say "stem.if =" stem.if

/* PARSE using a variable named parse. */
parse value "a b" with first second
say "parsed" first second "while parse =" parse

/* A label may also spell a keyword. */
call trace_label
exit 0

trace_label:
  say "called a label named trace_label; trace =" trace
  return
