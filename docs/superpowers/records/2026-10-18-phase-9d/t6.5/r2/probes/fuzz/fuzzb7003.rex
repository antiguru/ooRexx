parse arg dummy
call scenario "s0", "S <1 write line", "QR", "S =5 write"
call scenario "s1", "C9 RR", "R3", "S <1 write line"
call scenario "s2", "L1 n1", "QR", "N", "S <1 write", "L1 n1", "R"
call scenario "s3", "C9 RR", "QL", "S <2 read line"
call scenario "s4", "S =1 read", "R", "S +3 read"
call scenario "s5", "QR", "S =5 write", "QL", "R3"
call scenario "s6", "S <1 write", "C ab", "L5 n5", "S =5 write", "S =2 write line", "C3 Q", "L x"
call scenario "s7", "S <1 write line", "L2 n2"
call scenario "s8", "L3 n3", "C20 Z", "QL"
call scenario "s9", "K", "L x", "S =5 write", "S =7", "R1"
call scenario "s10", "QL", "L3 n3", "S =7"
call scenario "s11", "R", "S <1 write line", "R", "S =3 line"
call scenario "s12", "QL", "R3"
call scenario "s13", "S =7", "L2 n2", "S =2 write line"
call scenario "s14", "S +1 read line", "S +3 read", "L x"
call scenario "s15", "S +1 read line", "L1 n1"
call scenario "s16", "S <1 write line", "S +1 write line", "S +1 read line", "S =1 read", "L2 n2", "S -2 write", "L3 n3", "S =3 line", "C ab"
call scenario "s17", "S +3 read", "S +1 read line", "C20 Z", "S =5 write", "R1"
call scenario "s18", "N", "L5 n5", "C20 Z", "C9 RR", "S +1 read line"
call scenario "s19", "L5 n5", "L3 n3", "I", "S =7", "L x", "C9 RR", "S <1 write", "I"
call scenario "s20", "R1", "S <1 write", "C3 Q", "I", "R3", "L y", "L3 n3", "R3", "QL"
call scenario "s21", "C3 Q", "K", "S =1 read", "S +1 read line", "S +1 write line", "L1 n1", "L x"
call scenario "s22", "S =1 read", "S =7", "S <2 read line", "L3 n3"
call scenario "s23", "L2 n2", "R", "S <1 write line", "S +1 read line", "L5 n5"
call scenario "s24", "K", "R", "M"
call scenario "s25", "S =3 line", "R3", "S =2 write line", "C ab", "L3 n3", "C ab"
call scenario "s26", "R", "S +1 write line"
call scenario "s27", "I", "S +3 read", "L5 n5"
call scenario "s28", "R1", "QL", "L1 n1", "N", "K", "K", "S <1 write line", "S <2 read line"
call scenario "s29", "S =7", "S +1 read line", "S =7", "S =3 line", "S +1 read line", "S =3 line"
call scenario "s30", "L y", "C3 Q", "L2 n2"
call scenario "s31", "S -2 write", "S <1 write", "N", "R3"
call scenario "s32", "S +1 write line", "S <1 write line", "L5 n5", "S -2 write", "I", "C9 RR"
call scenario "s33", "M", "R3", "L1 n1", "S +3 read"
call scenario "s34", "C9 RR", "S +1 write line", "K", "L5 n5", "L y", "L5 n5", "L y"
call scenario "s35", "R", "L x"
call scenario "s36", "L5 n5", "L x", "S +1 write line", "S =3 line", "C3 Q"
call scenario "s37", "K", "S -2 write", "QL", "S =5 write", "C ab", "L5 n5"
call scenario "s38", "S =2 write line", "C9 RR", "S <2 read line", "L x", "S =7", "S +1 read line"
call scenario "s39", "S <1 write line", "N", "R3", "L3 n3", "L1 n1", "M"
call scenario "s40", "S <2 read line", "K", "S +1 read line", "QL", "S +3 read"
call scenario "s41", "R3", "L3 n3", "S -2 write", "QL", "C3 Q", "L x", "N", "I", "C9 RR"
call scenario "s42", "C9 RR", "S <2 read line", "L2 n2", "I", "L3 n3", "C3 Q", "QR", "R1", "S <2 read line"
call scenario "s43", "M", "S =7"
call scenario "s44", "S =2 write line", "S <1 write line", "S <2 read line", "C20 Z", "S <1 write", "R1"
call scenario "s45", "R3", "C20 Z"
call scenario "s46", "L5 n5", "L5 n5", "L y", "K", "S <1 write line", "S -2 write", "S =5 write", "S <2 read line", "S <1 write line"
call scenario "s47", "S =7", "S <2 read line", "S <1 write", "I", "S =1 read", "K"
call scenario "s48", "R3", "I", "S <2 read line", "QR"
call scenario "s49", "S <1 write line", "S -2 write", "S =2 write line", "L y", "S <1 write line", "QR", "N", "S =2 write line", "QR"
call scenario "s50", "S <1 write line", "R", "S +3 read", "R3", "S =5 write"
call scenario "s51", "S <1 write line", "C3 Q", "QL", "S =1 read", "C ab", "QR", "L y", "R1"
call scenario "s52", "K", "K", "S <2 read line", "QL", "L5 n5", "C ab", "R3"
call scenario "s53", "N", "S +1 write line", "L2 n2", "C20 Z", "L x", "S -2 write", "S -2 write", "L2 n2", "L1 n1"
call scenario "s54", "S -2 write", "C20 Z", "S =3 line", "C9 RR", "S +3 read", "C20 Z", "I"
call scenario "s55", "L5 n5", "S -2 write", "R1", "C9 RR", "C3 Q", "QL", "R"
call scenario "s56", "R3", "S =1 read", "S +1 read line", "QR", "N", "QL", "C9 RR"
call scenario "s57", "QR", "S <1 write line", "L1 n1"
call scenario "s58", "S -2 write", "S =1 read", "L2 n2", "I"
call scenario "s59", "C ab", "L x", "S =1 read"
call scenario "s60", "L5 n5", "L2 n2", "S <1 write", "M", "S <1 write line", "S =3 line", "S =3 line", "L3 n3"
call scenario "s61", "S -2 write", "S <1 write", "L y", "M", "S +1 read line", "S =2 write line", "L2 n2", "L5 n5", "L2 n2"
call scenario "s62", "R3", "QL", "S =1 read"
call scenario "s63", "S =2 write line", "S +1 write line", "L3 n3", "M", "S +1 write line", "S <1 write", "C9 RR", "L5 n5", "L2 n2"
call scenario "s64", "S +1 read line", "C3 Q", "K", "S <1 write", "S <2 read line", "S =2 write line", "L y", "C20 Z"
call scenario "s65", "L y", "M", "C ab", "S =5 write", "S =5 write", "S =3 line", "S =1 read"
call scenario "s66", "S +3 read", "K"
call scenario "s67", "R1", "L3 n3"
call scenario "s68", "S =5 write", "QL", "S =1 read", "S <2 read line", "QR", "C9 RR"
call scenario "s69", "R", "QL", "R", "N"
call scenario "s70", "L3 n3", "QL"
call scenario "s71", "S <1 write line", "C3 Q", "S =7", "L x", "R"
call scenario "s72", "QL", "I", "S +3 read", "L y"
call scenario "s73", "S =5 write", "C3 Q", "C20 Z", "C3 Q", "S =7", "L1 n1", "C9 RR", "C ab", "M"
call scenario "s74", "S =5 write", "L x", "S +1 read line", "S =2 write line"
call scenario "s75", "S =7", "S +1 read line", "S =7", "L3 n3"
call scenario "s76", "QR", "S =5 write", "C9 RR"
call scenario "s77", "S +1 read line", "S +1 write line", "C ab", "R", "S +3 read", "S <2 read line"
call scenario "s78", "R1", "R", "C9 RR", "I", "L1 n1"
call scenario "s79", "S <1 write line", "QR", "L y", "R1"
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
