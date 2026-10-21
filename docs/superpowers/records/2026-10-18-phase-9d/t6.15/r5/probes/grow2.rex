/* the re-review's grow2: a count, a read, another writer appends to the standard input file, a count */
say 'A' lines(, 'C') linein()
address system 'printf "x\ny\n" >> ' || arg(1)
say 'B' chars() lines(, 'C')
