/* A compiled source sees the public classes its parent package got through
   ::REQUIRES, not the non-public ones; the parent's own classes come first. */
/* parent's ::requires: public class visible, non-public not; public required vs parent installed */
say 'reqpub:' .routine~new('r', 'return .rpub~new~name')~call
say 'reqhidden:' .routine~new('r', 'return .rhid')~call
say 'dup:' .routine~new('r', 'return .dup~new~name')~call
say 'own req beats parent?:' .routine~new('r', .array~of('return .dup2~new~name', '::class dup2', '::method name', '  return "own2"'))~call
::requires 'lib.rex'
::class dup
::method name
  return 'main dup'
