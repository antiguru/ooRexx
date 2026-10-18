parse arg dummy
call scenario "s0", "S <1 write line", "R", "QL"
call scenario "s1", "S <1 write line", "S <2 read line", "L1 n1", "S <2 read line", "R1", "S +1 write line", "L3 n3"
call scenario "s2", "C ab", "S +3 read", "S =5 write", "C3 Q", "QR", "S <2 read line", "L1 n1"
call scenario "s3", "N", "C3 Q", "C20 Z", "N", "R", "S <1 write line"
call scenario "s4", "C ab", "S <1 write"
call scenario "s5", "L1 n1", "S <1 write line", "S <1 write"
call scenario "s6", "L x", "L x", "S =1 read", "S <2 read line", "S =2 write line", "R3"
call scenario "s7", "L y", "M", "S =2 write line"
call scenario "s8", "S =3 line", "L1 n1"
call scenario "s9", "C9 RR", "K", "S =1 read", "R3", "S =7", "S =1 read"
call scenario "s10", "L y", "S =1 read", "S =3 line", "C3 Q", "C20 Z", "C20 Z"
call scenario "s11", "S -2 write", "S =5 write"
call scenario "s12", "N", "QR", "K", "S -2 write"
call scenario "s13", "I", "S =7", "K", "S =3 line", "C9 RR"
call scenario "s14", "L1 n1", "S =3 line", "C ab", "S -2 write", "S =2 write line", "L3 n3"
call scenario "s15", "S =1 read", "S =2 write line", "S <1 write", "L3 n3", "N", "QL", "S +1 write line", "K"
call scenario "s16", "N", "S -2 write", "L5 n5", "S =7", "QR"
call scenario "s17", "R", "L x", "S <1 write line"
call scenario "s18", "R", "C9 RR", "S =5 write", "S <1 write", "S +3 read", "S =1 read", "QR"
call scenario "s19", "S =7", "L y", "C3 Q", "S +3 read", "S +3 read", "R3", "L3 n3", "L x", "QR"
call scenario "s20", "M", "L5 n5", "S =5 write", "QR", "S =1 read", "R", "S +1 write line", "C9 RR", "C20 Z"
call scenario "s21", "S <1 write line", "C3 Q", "S -2 write", "M", "QR", "S +1 read line", "S +3 read", "S =3 line", "L2 n2"
call scenario "s22", "L5 n5", "L y", "S =5 write"
call scenario "s23", "S +3 read", "QL", "S +1 write line", "M", "S +1 read line"
call scenario "s24", "C9 RR", "C20 Z", "QR", "QR", "S =2 write line", "C ab", "C9 RR"
call scenario "s25", "M", "QL", "K", "L3 n3"
call scenario "s26", "R3", "L2 n2", "N", "S =7", "S <1 write", "C9 RR", "C9 RR"
call scenario "s27", "S =1 read", "S =2 write line", "K", "C ab", "S +1 write line", "R3", "I", "S +3 read", "S =2 write line"
call scenario "s28", "S <1 write line", "C3 Q"
call scenario "s29", "S -2 write", "S <1 write", "S <2 read line", "R", "I"
call scenario "s30", "L2 n2", "S <1 write", "R3"
call scenario "s31", "C ab", "S +1 write line", "S <2 read line", "S <1 write line", "C20 Z", "C ab", "S <1 write line", "L y"
call scenario "s32", "L1 n1", "S <1 write", "L3 n3"
call scenario "s33", "C20 Z", "R1", "K", "L3 n3"
call scenario "s34", "S =1 read", "S <1 write", "S =7", "L3 n3", "R3", "S +1 write line", "S -2 write", "I"
call scenario "s35", "S =2 write line", "L2 n2", "S =5 write", "R", "S +1 read line", "K", "C ab"
call scenario "s36", "C9 RR", "QL", "L3 n3", "L y", "L3 n3"
call scenario "s37", "R", "S <1 write line", "L2 n2", "S -2 write", "S +1 write line"
call scenario "s38", "QR", "C9 RR", "R", "S =3 line", "L y", "S =7", "S =5 write", "R"
call scenario "s39", "S =3 line", "I", "R3", "M", "S <1 write line", "C3 Q"
call scenario "s40", "S <1 write line", "QR", "C9 RR", "R"
call scenario "s41", "S +1 write line", "S <2 read line", "S <1 write line", "L1 n1", "S <2 read line", "S =2 write line", "S =7", "C3 Q"
call scenario "s42", "L3 n3", "S =7", "R3", "C3 Q", "C3 Q", "L x", "S +1 write line"
call scenario "s43", "R1", "QL"
call scenario "s44", "R", "S <2 read line", "C20 Z", "S <1 write"
call scenario "s45", "S =3 line", "QL", "S +1 read line", "L5 n5", "L2 n2", "R3", "N", "S <1 write line", "K"
call scenario "s46", "S -2 write", "K", "C20 Z", "K", "R3"
call scenario "s47", "S +1 write line", "S =7", "S <2 read line", "C3 Q", "L1 n1", "C9 RR", "K"
call scenario "s48", "L y", "M", "M", "C ab", "S +1 read line", "L3 n3", "S =5 write", "S =5 write"
call scenario "s49", "R1", "M", "S <1 write", "S <1 write line", "S =3 line", "S <1 write line", "S =7", "C9 RR", "I"
call scenario "s50", "L3 n3", "C20 Z", "S +1 read line"
call scenario "s51", "S =2 write line", "S =2 write line", "S <1 write", "S =5 write", "QR", "K", "S +3 read", "R1"
call scenario "s52", "S <2 read line", "L3 n3", "L x", "S <1 write", "S <1 write", "S <1 write line", "L1 n1"
call scenario "s53", "S =7", "L x", "S <2 read line"
call scenario "s54", "S +3 read", "S =1 read", "C20 Z", "I", "C3 Q", "S =7"
call scenario "s55", "K", "S -2 write", "L x", "R3"
call scenario "s56", "C9 RR", "S -2 write", "S +1 read line", "R", "L y", "L5 n5"
call scenario "s57", "C20 Z", "L x", "L x", "S +3 read", "R", "S =7", "L5 n5", "L x"
call scenario "s58", "L3 n3", "R", "S <2 read line"
call scenario "s59", "QL", "QR", "R3", "S =3 line", "M", "S <2 read line", "S =2 write line", "S +3 read"
call scenario "s60", "K", "S <1 write", "S <2 read line", "S <1 write line", "S <1 write", "C20 Z", "S -2 write", "N"
call scenario "s61", "S +3 read", "C3 Q", "R", "C ab"
call scenario "s62", "K", "S -2 write"
call scenario "s63", "S <1 write line", "S =2 write line", "L y", "S =3 line", "R3", "L2 n2", "S <1 write line", "C20 Z", "L2 n2"
call scenario "s64", "S <1 write line", "S <1 write", "S =3 line", "C20 Z", "R1", "S <1 write", "S =2 write line", "C3 Q"
call scenario "s65", "L3 n3", "QR"
call scenario "s66", "S =7", "S +1 write line", "QL", "L1 n1"
call scenario "s67", "S -2 write", "L5 n5", "S <1 write line", "L y", "S =1 read", "C20 Z"
call scenario "s68", "I", "R3", "R"
call scenario "s69", "L1 n1", "K", "S =1 read", "S =1 read", "S =3 line"
call scenario "s70", "S <1 write", "L5 n5", "L2 n2", "S <1 write line", "S +3 read", "S <1 write line"
call scenario "s71", "S =3 line", "S =3 line", "L2 n2", "R", "S =1 read", "S -2 write"
call scenario "s72", "I", "S +1 read line", "QL", "QR", "I", "L2 n2", "S +3 read", "L2 n2"
call scenario "s73", "QL", "L3 n3", "M", "C9 RR"
call scenario "s74", "L2 n2", "S =7", "S +1 read line", "C9 RR", "C20 Z", "R3"
call scenario "s75", "N", "S =3 line"
call scenario "s76", "S <1 write line", "C3 Q", "R1", "R3", "S =5 write", "C9 RR", "S <2 read line", "L5 n5", "M"
call scenario "s77", "S =5 write", "L y", "R", "QL", "K"
call scenario "s78", "S -2 write", "R1", "N", "C20 Z", "L2 n2", "S +3 read"
call scenario "s79", "S =7", "N", "L1 n1", "S +1 write line", "L3 n3", "QL"
exit

scenario:
  f = 'b' || arg(1)~changestr(' ', '_') || '.txt'
  call charout f, 'AAAABBBBCCCCDDDD'
  call charout f
  call stream f, 'c', 'open both binary reclength 4 nobuffer'
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
