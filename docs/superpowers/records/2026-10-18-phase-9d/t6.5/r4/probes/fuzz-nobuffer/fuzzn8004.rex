parse arg dummy
call scenario "s0", "S =1 read", "M", "L5 n5", "C ab", "S =7", "S =7", "S =7", "S <2 read line", "S =1 read"
call scenario "s1", "C20 Z", "C3 Q", "S =3 line"
call scenario "s2", "S <1 write line", "QR", "C3 Q", "S =1 read", "L5 n5", "M"
call scenario "s3", "S =5 write", "M"
call scenario "s4", "QR", "L5 n5", "S <1 write line", "R", "QL", "L2 n2", "S =3 line", "S =1 read"
call scenario "s5", "R", "S =7"
call scenario "s6", "L2 n2", "S <1 write", "R", "C20 Z"
call scenario "s7", "L5 n5", "QL", "L1 n1", "I", "R", "QL", "C9 RR", "S =2 write line", "QR"
call scenario "s8", "L2 n2", "I", "S =7", "M", "L yy2", "C ab", "C ab", "S <1 write"
call scenario "s9", "S =2 write line", "N", "C9 RR", "I", "S =5 write", "R1", "QL"
call scenario "s10", "S =2 write line", "C20 Z", "L2 n2", "L yy2", "S <1 write", "QR", "L2 n2", "L yy2", "I"
call scenario "s11", "L x1", "S =3 line"
call scenario "s12", "R1", "S =5 write", "L5 n5", "M", "S =1 read", "S =7", "S =2 write line", "N"
call scenario "s13", "M", "C3 Q", "R", "L yy2", "L yy2"
call scenario "s14", "M", "QL"
call scenario "s15", "L2 n2", "L1 n1", "S <1 write line", "C20 Z", "R", "R", "R3", "L1 n1"
call scenario "s16", "QL", "S =7", "S =3 line", "L5 n5", "I", "I", "L2 n2"
call scenario "s17", "QL", "K"
call scenario "s18", "R", "QL"
call scenario "s19", "S =5 write", "QL", "L1 n1", "C3 Q", "C20 Z", "S <1 write line", "K", "L5 n5"
call scenario "s20", "S <1 write", "C9 RR", "L2 n2"
call scenario "s21", "C20 Z", "C9 RR"
call scenario "s22", "L5 n5", "S =7"
call scenario "s23", "L yy2", "QR"
call scenario "s24", "L2 n2", "S =3 line", "C3 Q", "S =2 write line", "S =1 read", "S =2 write line", "QL"
call scenario "s25", "R1", "S =7", "R1", "K", "L1 n1", "S =1 read", "L yy2", "S <2 read line"
call scenario "s26", "L1 n1", "QR", "C ab", "N", "S =7", "M", "L5 n5", "S =7", "L3 n3"
call scenario "s27", "C9 RR", "K", "C ab", "S <2 read line", "I", "R1", "L yy2", "S <2 read line", "R"
call scenario "s28", "C3 Q", "S =7", "S <1 write", "L2 n2"
call scenario "s29", "R3", "S <1 write line", "S =5 write", "QL", "L yy2", "QR", "S =5 write"
call scenario "s30", "R1", "C20 Z"
call scenario "s31", "L yy2", "QR", "S <1 write line", "C20 Z", "L x1"
call scenario "s32", "L3 n3", "S =1 read", "M", "R1"
call scenario "s33", "R3", "R", "L5 n5", "M", "C ab", "L yy2", "M", "L5 n5"
call scenario "s34", "C3 Q", "K", "M", "C9 RR", "S <1 write line", "L x1", "C ab", "C3 Q"
call scenario "s35", "R3", "S =5 write", "L1 n1", "S =1 read", "R3", "C9 RR", "L2 n2", "C3 Q"
call scenario "s36", "C20 Z", "L5 n5", "S =3 line", "S =2 write line"
call scenario "s37", "S =1 read", "C3 Q", "L1 n1", "S <1 write line"
call scenario "s38", "L2 n2", "S =7", "C9 RR", "L x1", "S <2 read line", "R3", "R1"
call scenario "s39", "C20 Z", "L5 n5", "L5 n5"
call scenario "s40", "L3 n3", "C20 Z"
call scenario "s41", "S <1 write line", "L3 n3", "N", "L3 n3"
call scenario "s42", "L5 n5", "C20 Z", "L2 n2", "R3", "R1", "L x1"
call scenario "s43", "S =2 write line", "L yy2", "R3", "I", "R", "S =5 write", "K", "L x1", "N"
call scenario "s44", "QR", "C20 Z", "S <2 read line", "QL", "R", "C ab"
call scenario "s45", "C20 Z", "N", "S =5 write", "S =1 read", "L x1", "L x1", "L1 n1", "S <2 read line"
call scenario "s46", "C3 Q", "C3 Q", "C20 Z"
call scenario "s47", "M", "L1 n1", "R", "K", "C3 Q", "M"
call scenario "s48", "R3", "S <1 write line", "I", "S =7"
call scenario "s49", "L yy2", "C20 Z", "L2 n2", "C20 Z"
call scenario "s50", "K", "QL"
call scenario "s51", "N", "I"
call scenario "s52", "S <1 write line", "L5 n5", "L1 n1"
call scenario "s53", "S =1 read", "M", "M", "M", "L5 n5", "S =3 line", "K", "K"
call scenario "s54", "K", "S =3 line", "S =5 write"
call scenario "s55", "K", "L yy2", "S =7", "C9 RR", "L5 n5"
call scenario "s56", "S <2 read line", "S =2 write line", "QR", "N", "I", "K", "S =2 write line", "L5 n5"
call scenario "s57", "S <1 write line", "S <2 read line"
call scenario "s58", "S =2 write line", "C9 RR", "S =2 write line", "S =5 write", "L3 n3"
call scenario "s59", "S =3 line", "L2 n2", "S =2 write line", "R3"
call scenario "s60", "M", "S =1 read", "S =5 write", "S =2 write line", "QL"
call scenario "s61", "L1 n1", "L1 n1", "S <1 write line"
call scenario "s62", "C ab", "L2 n2", "S <1 write", "S =1 read", "S =3 line", "R", "L x1"
call scenario "s63", "I", "R1", "S =7", "C ab", "QR", "I", "L x1", "L x1"
call scenario "s64", "C20 Z", "R3", "C3 Q", "QL"
call scenario "s65", "K", "S <2 read line", "S =7", "N", "S =7"
call scenario "s66", "S =7", "M", "C3 Q", "S =7", "S =2 write line"
call scenario "s67", "C20 Z", "L5 n5", "S =2 write line"
call scenario "s68", "K", "S =3 line"
call scenario "s69", "QR", "C ab", "QL", "S =3 line"
call scenario "s70", "L yy2", "C20 Z"
call scenario "s71", "R1", "S =3 line", "M"
call scenario "s72", "S <1 write line", "S =5 write", "S =3 line"
call scenario "s73", "S =1 read", "S <1 write line", "S =7", "QL", "R", "R1", "M", "M", "S <1 write"
call scenario "s74", "N", "C3 Q", "L yy2", "K", "N", "S <2 read line", "L yy2", "I"
call scenario "s75", "S =5 write", "QL", "L yy2", "C3 Q", "C ab", "QL", "R"
call scenario "s76", "S =5 write", "C20 Z", "R1", "L2 n2"
call scenario "s77", "S <2 read line", "N", "L1 n1", "R3", "C ab", "QL", "L1 n1", "R", "L1 n1"
call scenario "s78", "S =5 write", "QR", "R3", "S =5 write", "QR", "L yy2", "S =2 write line"
call scenario "s79", "S <1 write line", "M", "QL", "R", "I", "S <2 read line"
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
