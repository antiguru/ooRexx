/* `.CONTEXT`'s readers and a `StackFrame`'s are all declared at a parameter
   count of zero (Setup.cpp), so an argument is refused by the send before any
   body runs. The refusal is untrapped here because `rc` inside a handler
   carries only the major code -- 93 -- and the subcode and message are the
   part worth pinning. */

say 'A1 [' || .context~class~id || ']'
q = .context~digits(1)
say 'A2 [not reached]'
