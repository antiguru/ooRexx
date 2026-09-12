/* Everything a Stream answers before anything is opened. The stream named is
   this program's own file, because nothing can create one until the character
   and line entry points land -- and its size and timestamp are then the same
   file on both interpreters. */
parse source . . myself
s = .stream~new(myself)
say '[' || s~string || ']' == '[' || myself || ']'
say '[' || s~state || ']'
say '[' || s~description || ']'
say s~qualify == myself
say s~query('exists') == myself
say s~query('size') > 0
say '[' || s~query('streamtype') || ']'
say '[' || s~query('handle') || ']'
say s~query('datetime') \= ''
say s~query('timestamp') \= ''
/* A close of a stream that was never opened answers the null string, and a
   flush of one answers READY: -- neither opens anything. */
say '[' || s~close || ']'
say '[' || s~flush || ']'
say '[' || s~state || ']'
/* A name nothing resolves to still qualifies, and still answers nothing for
   the queries that stat it. */
n = .stream~new('nosuch_phase7_file.txt')
say '[' || n~state || ']' '[' || n~description || ']'
say '[' || n~query('exists') || ']' '[' || n~query('size') || ']'
say n~qualify \= ''
/* A directory exists as a path but is not a file the stream layer can find,
   while its size still stats. */
d = .stream~new('.')
say '[' || d~query('exists') || ']' '[' || d~query('size') || ']'
