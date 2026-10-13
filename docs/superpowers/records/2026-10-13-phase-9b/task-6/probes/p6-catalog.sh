printf '$set 1\n1 one\n2 two &1 &2\n3 \n$set 2\n5 five\n' > in.msg && gencat w.cat in.msg   # the catalog p6.rex reads, argument: its absolute path
