k = .K~new
k~at = 'via-set'
say 'main' k~at k~peek~at
::class inner
::attribute at
::class k
::attribute d
::method peek
  expose d
  return d
::method init
  expose d
  d = .Inner~new
::attribute at delegate d
