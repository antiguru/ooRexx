/* A failed Queue~insert past the extent, then a put inside the extent the
   insert asked for. */
q = .Queue~new
signal on syntax name s1
q~insert('a', 99)
s1: say 'insert' condition('o')~code
signal on syntax name s2
q~put('m', 50)
s2: say 'put after the failed insert' condition('o')~code
signal on syntax name s3
.Queue~new~put('m', 50)
s3: say 'put on a fresh queue' condition('o')~code
