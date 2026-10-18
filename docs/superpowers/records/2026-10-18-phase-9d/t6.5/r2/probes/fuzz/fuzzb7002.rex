parse arg dummy
call scenario "s0", "K", "L2 n2"
call scenario "s1", "S =2 write line", "N", "L2 n2", "K", "L x", "C9 RR"
call scenario "s2", "R", "S <1 write line"
call scenario "s3", "R3", "C9 RR", "R1", "C9 RR", "R3", "I", "L2 n2"
call scenario "s4", "S +1 write line", "R1", "I"
call scenario "s5", "S -2 write", "S +3 read", "L5 n5", "I"
call scenario "s6", "S -2 write", "S +1 read line", "S <1 write", "R1", "R", "QL", "S =3 line"
call scenario "s7", "S +1 read line", "S +1 read line", "C9 RR", "R3", "C ab", "S +1 read line", "S +1 read line"
call scenario "s8", "C ab", "R1", "I", "L x", "S =3 line", "I", "I", "L2 n2", "S <2 read line"
call scenario "s9", "L2 n2", "C ab", "S <1 write", "QL", "I"
call scenario "s10", "S =2 write line", "S +3 read", "N", "S +3 read", "L2 n2", "C9 RR", "S -2 write", "S =7"
call scenario "s11", "N", "S +3 read", "L2 n2", "S -2 write", "S =2 write line", "S -2 write"
call scenario "s12", "C20 Z", "K", "S -2 write", "C9 RR"
call scenario "s13", "L2 n2", "L y", "C ab", "S =3 line", "M", "L y"
call scenario "s14", "S <1 write line", "M", "S <1 write", "QR", "C ab", "L x"
call scenario "s15", "L x", "S =1 read", "S =7", "QR", "S <2 read line", "S +3 read", "R1"
call scenario "s16", "S <2 read line", "QL", "R", "C ab", "L1 n1", "N", "S <2 read line", "C3 Q", "S =1 read"
call scenario "s17", "L3 n3", "S <2 read line", "C ab", "R3"
call scenario "s18", "K", "C9 RR", "C ab", "S =2 write line"
call scenario "s19", "R3", "L3 n3"
call scenario "s20", "S <2 read line", "S <1 write line"
call scenario "s21", "C ab", "C20 Z", "S +1 write line", "L1 n1", "R3", "S =1 read", "S =3 line", "R", "L1 n1"
call scenario "s22", "L x", "M", "S +1 read line", "S =1 read", "I", "M", "S <2 read line", "S =2 write line", "S +3 read"
call scenario "s23", "QL", "S =2 write line", "N", "M", "N", "C ab", "L1 n1", "QR"
call scenario "s24", "L3 n3", "L y", "K", "M", "C20 Z", "L5 n5", "S +3 read", "C20 Z"
call scenario "s25", "R1", "S <1 write line", "S =7", "S -2 write", "M", "L2 n2", "QL", "S <1 write line"
call scenario "s26", "L y", "K", "L y", "L y", "L5 n5", "L1 n1"
call scenario "s27", "S <2 read line", "L5 n5", "I", "R3", "S =3 line", "S <1 write line", "L3 n3", "C ab"
call scenario "s28", "S +1 read line", "S +1 read line", "S +1 read line", "N", "S <1 write", "I", "I", "R1", "S -2 write"
call scenario "s29", "S =7", "S <1 write line"
call scenario "s30", "L1 n1", "S +1 read line", "QR", "S =1 read"
call scenario "s31", "S +1 write line", "R3", "L1 n1", "S =2 write line", "L5 n5"
call scenario "s32", "S =2 write line", "L2 n2", "S +1 read line", "QL", "S +1 write line"
call scenario "s33", "I", "L x", "R1", "L1 n1", "S =5 write", "L x", "C ab", "I"
call scenario "s34", "C9 RR", "S =2 write line", "L5 n5", "C ab", "R", "QR", "N", "M"
call scenario "s35", "L x", "L5 n5", "S <2 read line", "L y", "R1", "S =2 write line"
call scenario "s36", "S =3 line", "L3 n3", "S +1 write line"
call scenario "s37", "R", "L5 n5", "L1 n1", "S <2 read line"
call scenario "s38", "L5 n5", "S =5 write", "L1 n1", "L5 n5", "L y", "L1 n1", "L2 n2"
call scenario "s39", "QL", "S +3 read", "S +3 read", "S <1 write line", "S =7"
call scenario "s40", "C ab", "L x", "L2 n2", "S <1 write line", "R", "QR", "L3 n3"
call scenario "s41", "C9 RR", "S =7", "L5 n5", "R", "L2 n2", "S <1 write", "S -2 write", "S +3 read"
call scenario "s42", "K", "K", "K", "S =5 write", "L1 n1", "S =5 write", "L y"
call scenario "s43", "S +1 read line", "QL", "K", "C9 RR", "L1 n1"
call scenario "s44", "QL", "L1 n1", "S <1 write", "I", "R1"
call scenario "s45", "L1 n1", "C20 Z", "L5 n5", "L x", "S =5 write", "L1 n1", "L x"
call scenario "s46", "S =3 line", "N", "L3 n3"
call scenario "s47", "S =2 write line", "C3 Q", "S -2 write", "L1 n1", "L5 n5", "L3 n3", "S =1 read"
call scenario "s48", "M", "L y", "C20 Z", "C3 Q", "QL", "C20 Z", "S =1 read", "S =2 write line"
call scenario "s49", "R", "S -2 write"
call scenario "s50", "S -2 write", "M", "C ab", "R", "R3", "S =7"
call scenario "s51", "S =3 line", "S +1 write line", "L2 n2"
call scenario "s52", "R", "R3", "L3 n3", "R3"
call scenario "s53", "S <2 read line", "S -2 write", "R", "L3 n3", "L5 n5", "C9 RR", "K"
call scenario "s54", "L2 n2", "L x"
call scenario "s55", "S =1 read", "QL", "C9 RR", "L3 n3"
call scenario "s56", "L5 n5", "S <2 read line", "S <2 read line", "S <1 write", "R", "C20 Z", "S <1 write", "S <1 write line"
call scenario "s57", "S <1 write line", "QL", "N", "M", "L2 n2"
call scenario "s58", "S +3 read", "S =5 write", "R1", "S =7", "K", "C ab", "N", "C20 Z"
call scenario "s59", "S +1 write line", "S +1 read line", "R3", "M"
call scenario "s60", "S +1 read line", "N", "L1 n1"
call scenario "s61", "S =5 write", "K", "S =1 read", "S -2 write", "C ab", "L2 n2", "N"
call scenario "s62", "C9 RR", "S <1 write line"
call scenario "s63", "S -2 write", "M", "R", "QL", "R3"
call scenario "s64", "L3 n3", "S =7"
call scenario "s65", "N", "R", "R1", "L2 n2", "S +1 write line", "S =3 line", "C20 Z", "R"
call scenario "s66", "L2 n2", "C9 RR"
call scenario "s67", "S =1 read", "R", "C20 Z", "S <1 write line", "S =2 write line", "S <1 write", "S =7"
call scenario "s68", "S =3 line", "C20 Z"
call scenario "s69", "L2 n2", "L3 n3", "QR", "S <1 write", "C3 Q"
call scenario "s70", "R1", "S =3 line"
call scenario "s71", "R1", "R", "C3 Q"
call scenario "s72", "QR", "C ab"
call scenario "s73", "S <1 write", "S =1 read", "S =2 write line", "C9 RR", "S +3 read", "L x"
call scenario "s74", "S =5 write", "QL", "QR", "S =3 line"
call scenario "s75", "I", "L2 n2", "S =5 write", "S =5 write"
call scenario "s76", "M", "C ab", "L y", "M", "R", "S =1 read", "R1", "R1", "S =3 line"
call scenario "s77", "S <1 write line", "S +1 read line"
call scenario "s78", "S +1 write line", "R3", "S <1 write", "C3 Q", "R1", "S =7", "L y", "S =7"
call scenario "s79", "S +3 read", "S =5 write", "S =2 write line", "R", "R1", "M"
exit

scenario:
  f = 'b' || arg(1)~changestr(' ', '_') || '.txt'
  call charout f, 'AAAABBBBCCCCDDDD'
  call charout f
  call stream f, 'c', 'open both binary reclength 4'
  out = arg(1) ':'
  do i = 2 to arg()
    parse value arg(i) with op text
    select
      when op = 'L' then r = lineout(f, text)
      when op~left(1) = 'L' then r = lineout(f, text, op~substr(2))
      when op = 'C' then r = charout(f, text)
      when op~left(1) = 'C' then r = charout(f, text, op~substr(2))
      when op = 'R' then r = linein(f)
      when op = 'I' then r = charin(f, , 2)
      when op = 'QL' then r = stream(f, 'c', 'query position write line')
      when op = 'QR' then r = stream(f, 'c', 'query position read line')
      when op~left(1) = 'R' then r = linein(f, op~substr(2))
      when op = 'S' then r = stream(f, 'c', 'seek' text)
      when op = 'Q' then r = stream(f, 'c', 'query position write line')
      when op = 'N' then r = chars(f)
      when op = 'M' then r = lines(f)
      when op = 'K' then r = lines(f, 'N')
    end
    out = out op'='r 'w' || stream(f, 'c', 'query position write') 'r' || stream(f, 'c', 'query position read')
  end
  call stream f, 'c', 'close'
  say out
  say '   [' || charin(f, 1, 60)~changestr('00'x, '@')~changestr('0a'x, '|') || ']'
  call stream f, 'c', 'close'
  return
