parse arg dummy
call scenario "s0", "QR", "S =5 write", "R3", "C9 RR", "L1 n1"
call scenario "s1", "R1", "N", "L3 n3", "L yy2", "S <1 write line", "S <2 read line"
call scenario "s2", "L3 n3", "R1", "S <1 write line", "S <2 read line", "S <1 write line", "S =7"
call scenario "s3", "L3 n3", "S =1 read", "S =1 read", "S <2 read line", "S =1 read", "M", "S =2 write line", "C20 Z", "C3 Q"
call scenario "s4", "C9 RR", "R1", "I", "C9 RR", "QR", "C ab", "N"
call scenario "s5", "L1 n1", "C20 Z"
call scenario "s6", "L2 n2", "L2 n2", "QL", "C ab", "L3 n3"
call scenario "s7", "C9 RR", "S =1 read", "L2 n2", "S =3 line"
call scenario "s8", "S =3 line", "S =7", "QL", "K", "S =3 line", "C9 RR"
call scenario "s9", "C20 Z", "L2 n2", "R", "QR", "R", "R1"
call scenario "s10", "L2 n2", "QR", "M", "S =2 write line", "N", "N"
call scenario "s11", "C3 Q", "L5 n5", "I", "C20 Z"
call scenario "s12", "QR", "M", "S =1 read", "S <1 write line", "R3"
call scenario "s13", "L5 n5", "R", "QR", "K"
call scenario "s14", "L5 n5", "C20 Z", "QL", "S <1 write", "I", "R3"
call scenario "s15", "S =1 read", "L x1", "L2 n2"
call scenario "s16", "S =1 read", "S =2 write line", "K", "L3 n3", "L x1", "S <1 write line", "L yy2", "L1 n1", "L2 n2"
call scenario "s17", "L2 n2", "R", "QL", "L yy2"
call scenario "s18", "L yy2", "I", "S <1 write", "L x1", "R", "I"
call scenario "s19", "L5 n5", "C3 Q", "R1", "R3", "S <2 read line", "S <1 write line", "S =3 line", "R1", "N"
call scenario "s20", "L5 n5", "C3 Q", "S <2 read line", "S =5 write", "N", "S =2 write line", "L x1"
call scenario "s21", "S <1 write", "S =7", "R", "QL", "K", "S =5 write", "S <2 read line"
call scenario "s22", "QR", "L yy2", "C3 Q", "R1"
call scenario "s23", "L2 n2", "S =1 read", "R1", "I", "QL", "L x1", "S =2 write line", "QL"
call scenario "s24", "C ab", "QR", "QR", "I", "N", "L3 n3", "C9 RR", "R3", "R1"
call scenario "s25", "S <1 write line", "C3 Q", "M", "L yy2", "C9 RR", "L x1"
call scenario "s26", "R3", "QR", "N", "R1", "R", "S =3 line"
call scenario "s27", "C3 Q", "N"
call scenario "s28", "S <1 write line", "M", "S <1 write", "M", "L2 n2", "N"
call scenario "s29", "L5 n5", "K"
call scenario "s30", "C3 Q", "I", "QL", "I", "K", "S =5 write", "S =5 write", "L yy2", "S <1 write"
call scenario "s31", "C ab", "S <1 write line", "QR", "R1", "C ab", "C9 RR", "S <1 write line", "S <1 write"
call scenario "s32", "C20 Z", "S <1 write line", "N", "N", "S <1 write line", "C9 RR"
call scenario "s33", "K", "C ab", "R1", "N"
call scenario "s34", "R", "L2 n2", "C3 Q", "R3", "S =5 write", "S =1 read"
call scenario "s35", "L5 n5", "S <1 write"
call scenario "s36", "R1", "S =3 line", "L yy2", "C20 Z", "C ab", "S <1 write", "S <1 write line"
call scenario "s37", "I", "L x1", "C ab", "S =7", "R", "R", "I", "R3", "S <2 read line"
call scenario "s38", "L3 n3", "S <1 write", "R1"
call scenario "s39", "S =5 write", "S <1 write", "L1 n1", "L1 n1", "S =7", "M", "L2 n2", "S =1 read"
call scenario "s40", "C9 RR", "I", "S <1 write", "L2 n2", "S =5 write"
call scenario "s41", "QR", "L x1", "S <1 write line", "K", "C ab", "C9 RR", "K"
call scenario "s42", "S =7", "L3 n3", "L2 n2", "C3 Q", "R", "S <2 read line", "C9 RR"
call scenario "s43", "R1", "S =5 write", "QR", "S =3 line", "L3 n3", "C3 Q", "C20 Z"
call scenario "s44", "K", "R1", "S =2 write line", "C3 Q", "QR"
call scenario "s45", "S =3 line", "L2 n2"
call scenario "s46", "M", "R", "QL", "C ab", "S <2 read line", "R3"
call scenario "s47", "S <1 write", "L1 n1", "S <2 read line"
call scenario "s48", "S =2 write line", "S =7", "C9 RR", "L1 n1", "C20 Z", "S <1 write line", "L1 n1", "N", "QL"
call scenario "s49", "N", "R1", "S =1 read"
call scenario "s50", "S =3 line", "L1 n1", "L yy2", "K"
call scenario "s51", "S <1 write", "L3 n3", "L x1", "L5 n5", "I"
call scenario "s52", "R", "C20 Z", "L2 n2", "L2 n2", "S =5 write", "S <2 read line", "QL", "L1 n1", "S =1 read"
call scenario "s53", "S =3 line", "QR", "L3 n3"
call scenario "s54", "S =1 read", "I", "C3 Q", "L2 n2", "R3", "L yy2"
call scenario "s55", "L1 n1", "S =5 write"
call scenario "s56", "N", "K", "S <2 read line", "L yy2"
call scenario "s57", "I", "L yy2", "R1", "S =7", "S <1 write line", "R"
call scenario "s58", "L x1", "C ab", "S =1 read", "C ab", "C ab", "S =5 write", "S <1 write line"
call scenario "s59", "S =2 write line", "S =3 line", "L x1", "C20 Z", "C3 Q", "C9 RR", "S =2 write line", "M"
call scenario "s60", "R1", "K", "S <2 read line", "S =1 read", "S =1 read", "QR"
call scenario "s61", "R1", "S <2 read line"
call scenario "s62", "L1 n1", "C9 RR", "L x1", "L5 n5", "QL"
call scenario "s63", "C3 Q", "C20 Z", "L1 n1", "S <2 read line", "C20 Z", "L x1"
call scenario "s64", "C ab", "C9 RR", "S =3 line", "QR", "S <1 write", "S =7"
call scenario "s65", "L1 n1", "S =5 write", "C3 Q", "R1", "K", "L5 n5", "C20 Z"
call scenario "s66", "L5 n5", "S =3 line", "S =1 read", "S =2 write line", "S <2 read line"
call scenario "s67", "R1", "C9 RR", "S =2 write line"
call scenario "s68", "R3", "S <1 write line", "L3 n3", "R1", "L5 n5", "QL", "C ab", "S <1 write line"
call scenario "s69", "S =5 write", "L5 n5", "S <1 write", "R", "C20 Z"
call scenario "s70", "S <1 write line", "K", "C20 Z", "R1", "L5 n5", "L yy2", "I", "C ab"
call scenario "s71", "L1 n1", "S <1 write line", "S =7", "QR", "QL", "R1", "L yy2"
call scenario "s72", "S <1 write", "QL"
call scenario "s73", "C9 RR", "S =2 write line", "L x1", "C20 Z", "R3"
call scenario "s74", "L x1", "L3 n3", "L yy2", "C3 Q", "L3 n3", "S <1 write line", "R", "C20 Z"
call scenario "s75", "L yy2", "L yy2", "L2 n2", "N", "C3 Q", "L x1", "K", "C ab", "QL"
call scenario "s76", "K", "M", "S <1 write line", "S <1 write line", "N", "C20 Z", "C20 Z", "S <1 write line", "S =5 write"
call scenario "s77", "C9 RR", "L5 n5"
call scenario "s78", "C ab", "L x1", "R1", "C9 RR", "C ab", "S =2 write line", "R"
call scenario "s79", "N", "R1", "I", "L yy2", "S <1 write line", "R3", "L2 n2", "C20 Z", "L x1"
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
