object1 = .subexample~new
say object1~repeat(4, "call1")
say object1~repeater(4, "call2")
say "Main ended."
exit
::class example
::method repeat
use arg reps,msg
reply "Repeating" msg
do reps
  say msg
end
::class subexample subclass example
::method repeater
use arg reps,msg
reply "Repeating" msg
do reps
  say msg
end
