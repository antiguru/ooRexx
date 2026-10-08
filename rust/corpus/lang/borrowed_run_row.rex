/* Object's RUN row set on an instance runs an attribute getter there. */
o = .t~new
say o~go
::class t
::method go
  self~setMethod('rn', .object~method('RUN'))
  return self~rn(.u~method('A'))
::class u
::attribute a
