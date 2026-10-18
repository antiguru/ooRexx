/* A package a run-time load requires starts from the requiring package's
   own settings: its ::OPTIONS, and the running activation's only under its
   NUMERIC INHERIT. .Routine~newFile and .Method~newFile of a file with no
   ::OPTIONS give the inheriting file 9, .Routine~new with DIGITS 6 gives it
   6, and an inheriting .Package~new source gives 4. */
numeric digits 4
.routine~newfile('rr_plain.rex')~call
.routine~new('x', .array~of("say 'r' digits() .c6~k", "::requires 'rr_q6.cls'", "::options digits 6"))~call
m = .method~newfile('rr_method.rex')
m = .method~new('m', .array~of("return 1", "::requires 'rr_qn.cls'"))
p = .package~new('rr_src.rex', .array~of("say 'src' digits()", "::requires 'rr_z.cls'", "::options numeric inherit"))
