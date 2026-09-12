/* SIGNAL ON NOTREADY: a read past the end transfers to the label, and the
   clause that raised never finishes -- the SAY around the read prints
   nothing. The handler reads the condition back through the four letters this
   crate answers; CONDITION('A') and ('O') are a Directory and an object and
   refuse here, so neither is asked for. */
say 'signal on notready'
c = .Stream~new('n.txt')
c~lineout('one')
c~close
signal on notready name caught
s = .Stream~new('n.txt')
say 'first [' || s~linein || ']'
say 'before' s~linein 'after'
say 'the raising clause finished, which it must not have'
exit 0

caught:
say 'C=' condition('C')
say 'D=' condition('D')
say 'I=' condition('I')
say 'S=' condition('S')
say 'state [' || s~state || '] [' || s~description || ']'
say 'close [' || s~close || ']'
exit 0
