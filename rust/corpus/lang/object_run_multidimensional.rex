/* ~run's A argument style reaches arrayArgument's named overload under a
   different name, so the same 88.913 reports `argument array` where
   ~sendWith reports `message arguments`. */

o = .K~new
say o~one
say o~two
say 'unreached'

::CLASS K
::METHOD ONE
  return self~run("return 'ran' arg()", 'A', .array~new(2))
::METHOD TWO
  return self~run("return 'ran' arg()", 'A', .array~new(2,2))
