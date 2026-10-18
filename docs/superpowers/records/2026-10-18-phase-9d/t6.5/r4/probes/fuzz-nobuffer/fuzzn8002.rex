parse arg dummy
call scenario "s0", "R3", "S <2 read line", "S =5 write", "K", "S =3 line"
call scenario "s1", "C3 Q", "R1", "S <1 write line", "R1"
call scenario "s2", "L1 n1", "C ab", "C20 Z", "M", "QL", "S =3 line", "C9 RR", "L yy2", "L yy2"
call scenario "s3", "S =3 line", "C ab"
call scenario "s4", "C ab", "S =7", "R1"
call scenario "s5", "R", "S =5 write", "L1 n1", "R", "M", "M", "N", "C ab", "R3"
call scenario "s6", "L3 n3", "L1 n1", "S =3 line", "L5 n5", "S =1 read"
call scenario "s7", "QR", "R3", "I"
call scenario "s8", "N", "R1", "R3"
call scenario "s9", "S =5 write", "QR", "R1", "S =3 line", "R3"
call scenario "s10", "L3 n3", "M", "I", "S <2 read line", "S =3 line", "L x1", "C20 Z"
call scenario "s11", "C ab", "C20 Z", "S =2 write line", "S =2 write line"
call scenario "s12", "C20 Z", "S =3 line", "QR", "L1 n1", "C9 RR", "QR", "L x1", "L x1"
call scenario "s13", "L3 n3", "L x1"
call scenario "s14", "C3 Q", "C9 RR", "L1 n1", "S =2 write line", "N", "C3 Q", "L yy2", "S =7"
call scenario "s15", "L2 n2", "R1", "C3 Q", "S =2 write line", "QL", "K", "C20 Z", "L x1", "S =2 write line"
call scenario "s16", "L1 n1", "L yy2"
call scenario "s17", "L yy2", "C9 RR", "L5 n5", "L5 n5", "L3 n3"
call scenario "s18", "S =5 write", "R3", "C ab", "S =5 write", "S <2 read line", "S <1 write", "L5 n5"
call scenario "s19", "R3", "L yy2", "C20 Z"
call scenario "s20", "S <1 write", "S =3 line", "L2 n2", "S <2 read line", "R3", "S =7"
call scenario "s21", "R", "K", "S =1 read", "S <1 write line", "S <1 write", "S =7", "R1", "R1"
call scenario "s22", "M", "S <1 write line", "C ab", "S <1 write"
call scenario "s23", "C9 RR", "L x1", "R", "I", "L3 n3", "S <2 read line", "S <1 write line", "I"
call scenario "s24", "S =2 write line", "R1", "R3", "C20 Z", "S =2 write line"
call scenario "s25", "L2 n2", "C9 RR", "S <1 write"
call scenario "s26", "S <1 write line", "QR", "S =3 line", "S <2 read line", "QR", "L5 n5", "S =1 read", "C20 Z", "L1 n1"
call scenario "s27", "S =7", "R1", "L x1", "L5 n5", "QR", "S <1 write line", "C ab", "C3 Q", "S <2 read line"
call scenario "s28", "S =5 write", "L3 n3", "C3 Q", "QL", "QL", "K", "S <2 read line", "C ab", "C3 Q"
call scenario "s29", "L3 n3", "QL", "L3 n3", "S <2 read line"
call scenario "s30", "L2 n2", "M", "S =2 write line", "L3 n3", "QR", "C20 Z"
call scenario "s31", "S =7", "C9 RR", "R", "QR", "L yy2", "S =5 write", "M"
call scenario "s32", "S =2 write line", "S =3 line"
call scenario "s33", "S =3 line", "S =3 line", "C3 Q", "L3 n3", "S <1 write line", "L5 n5", "R1", "S <1 write line", "R"
call scenario "s34", "C3 Q", "S =2 write line", "R", "C3 Q", "QR", "L3 n3", "L1 n1", "QR"
call scenario "s35", "S =1 read", "M", "S =5 write"
call scenario "s36", "R", "QL", "K", "K"
call scenario "s37", "S <1 write", "L1 n1", "S =1 read", "QL", "S =5 write", "R1", "QR", "L5 n5"
call scenario "s38", "M", "L3 n3", "I", "R3", "C9 RR", "K"
call scenario "s39", "C ab", "S =7", "N", "S =7", "S =1 read", "I", "C20 Z"
call scenario "s40", "L5 n5", "L2 n2", "S =1 read", "R1", "S <2 read line"
call scenario "s41", "S =1 read", "L1 n1", "S =7", "L x1", "S <1 write", "C20 Z", "S =7", "L yy2"
call scenario "s42", "S <1 write", "N"
call scenario "s43", "C20 Z", "S <2 read line", "M", "S <1 write line", "QR", "S =2 write line", "S =2 write line", "S <2 read line", "QL"
call scenario "s44", "M", "L yy2", "M", "R3"
call scenario "s45", "L1 n1", "R", "L yy2", "S =7", "S =1 read", "L yy2", "R3", "R"
call scenario "s46", "L5 n5", "L2 n2", "S =2 write line", "S <1 write", "M"
call scenario "s47", "C ab", "S =7", "L5 n5", "S =3 line", "QL", "L x1", "S <2 read line", "L5 n5"
call scenario "s48", "S <2 read line", "L yy2", "L x1", "L2 n2"
call scenario "s49", "K", "QR", "L x1", "N", "R3", "N", "S <2 read line", "C20 Z"
call scenario "s50", "L yy2", "R3"
call scenario "s51", "M", "L2 n2", "S <1 write", "S =3 line", "QL", "QL", "L3 n3", "N", "S =1 read"
call scenario "s52", "QR", "S <1 write", "S <1 write line", "L yy2", "QR", "N", "QR", "C9 RR", "S =3 line"
call scenario "s53", "L2 n2", "K", "S <1 write"
call scenario "s54", "N", "L2 n2", "S =3 line", "C3 Q", "S =1 read", "S =7", "S =3 line", "QR", "S =2 write line"
call scenario "s55", "L2 n2", "I"
call scenario "s56", "QL", "S =7", "M"
call scenario "s57", "S <1 write line", "I", "L5 n5", "L yy2", "C ab", "I", "S <1 write", "L x1", "C9 RR"
call scenario "s58", "QR", "C9 RR", "QR", "S =7", "R1", "QR", "N", "L2 n2"
call scenario "s59", "M", "R", "R3", "S =1 read", "R"
call scenario "s60", "S =7", "M"
call scenario "s61", "S <2 read line", "C ab"
call scenario "s62", "R3", "C ab", "R3", "L x1", "C20 Z", "S =1 read", "L2 n2"
call scenario "s63", "L1 n1", "R3", "S <2 read line", "S =2 write line", "L3 n3"
call scenario "s64", "L3 n3", "S <1 write line"
call scenario "s65", "QL", "S =7", "L2 n2", "L5 n5", "S =5 write", "L1 n1", "S =7"
call scenario "s66", "R", "N", "R1", "S =1 read", "S =5 write", "S <1 write line"
call scenario "s67", "L yy2", "L2 n2", "L2 n2", "R1", "QL"
call scenario "s68", "N", "M", "L1 n1", "R3", "QL"
call scenario "s69", "QL", "C ab", "S =3 line", "L5 n5", "S =3 line", "K", "C ab", "L x1", "M"
call scenario "s70", "L x1", "C9 RR", "N", "L1 n1", "R1", "L1 n1"
call scenario "s71", "I", "L yy2", "L x1", "QL", "S =5 write"
call scenario "s72", "S =5 write", "S <2 read line"
call scenario "s73", "S =5 write", "C ab", "R1", "S <1 write line", "L yy2", "QR"
call scenario "s74", "S =2 write line", "S =2 write line", "S <2 read line", "S <2 read line", "L1 n1"
call scenario "s75", "L1 n1", "M", "M"
call scenario "s76", "I", "S =3 line", "L x1", "S =1 read", "R", "L2 n2"
call scenario "s77", "I", "R1", "C20 Z"
call scenario "s78", "R1", "S <2 read line", "C9 RR", "S <2 read line", "S =3 line"
call scenario "s79", "M", "S <2 read line", "S <2 read line", "M", "S =7", "I", "C9 RR", "S <1 write"
exit

scenario:
  f = 'f' || arg(1)~changestr(' ', '_') || '.txt'
  call charout f, 'line1' || '0a'x || 'line2' || '0a'x || 'line3' || '0a'x
  call charout f
  call stream f, 'c', 'open both nobuffer'
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
