parse arg dummy
call scenario "s0", "C ab", "S <2 read line", "QR", "S =7", "QR", "I"
call scenario "s1", "N", "C ab", "M", "K", "QL", "C20 Z", "L x", "L y", "L1 n1"
call scenario "s2", "S =7", "S =3 line"
call scenario "s3", "I", "C20 Z", "L1 n1"
call scenario "s4", "S <1 write", "K"
call scenario "s5", "QL", "S =7", "S =2 write line", "N"
call scenario "s6", "K", "S =7", "L x", "L1 n1", "C3 Q", "R3"
call scenario "s7", "S =7", "M", "S =3 line"
call scenario "s8", "R1", "S <1 write line", "C20 Z", "L3 n3", "S <1 write", "S =3 line"
call scenario "s9", "C ab", "C20 Z", "S =7", "C20 Z"
call scenario "s10", "C ab", "C9 RR", "C ab", "I", "R1", "N", "S <1 write line"
call scenario "s11", "QR", "S =2 write line", "I", "C9 RR", "L y", "L5 n5", "S =3 line", "S =7"
call scenario "s12", "S <2 read line", "R"
call scenario "s13", "S =2 write line", "R3", "S =1 read"
call scenario "s14", "C9 RR", "S =2 write line", "R3", "C9 RR", "C3 Q", "S =1 read"
call scenario "s15", "L5 n5", "S <1 write line", "K"
call scenario "s16", "S =2 write line", "S =2 write line", "C3 Q", "L x"
call scenario "s17", "S <2 read line", "C9 RR", "L1 n1", "C20 Z", "S <1 write"
call scenario "s18", "C9 RR", "S <2 read line", "L1 n1", "L1 n1", "C20 Z", "L y", "N", "C9 RR"
call scenario "s19", "C ab", "L3 n3", "M", "S <2 read line", "S =3 line", "S <1 write", "S =7"
call scenario "s20", "S =3 line", "S =2 write line", "L2 n2", "M", "I", "C20 Z", "S =2 write line", "L x", "K"
call scenario "s21", "S <1 write line", "C3 Q"
call scenario "s22", "R3", "S <1 write", "L3 n3", "QR", "S =1 read", "L1 n1"
call scenario "s23", "S <1 write line", "C3 Q"
call scenario "s24", "C9 RR", "S <2 read line", "K", "L y", "R"
call scenario "s25", "S <1 write line", "L x"
call scenario "s26", "L y", "R1", "N"
call scenario "s27", "QL", "C20 Z", "QR", "R1", "K", "M", "C20 Z", "N", "S <2 read line"
call scenario "s28", "R3", "C3 Q", "S =5 write"
call scenario "s29", "L y", "N", "N", "S =5 write", "M"
call scenario "s30", "L y", "R", "M", "I", "S <2 read line", "S <2 read line"
call scenario "s31", "C9 RR", "S <2 read line", "S =5 write", "S =3 line", "R1", "R"
call scenario "s32", "K", "L y", "L y", "L3 n3"
call scenario "s33", "R3", "S =1 read", "I", "S =3 line"
call scenario "s34", "L2 n2", "R1", "M", "L2 n2", "QR", "L5 n5", "S <1 write line", "C3 Q"
call scenario "s35", "S <1 write", "R", "C3 Q", "QR", "S <2 read line", "N", "S =5 write", "R1", "S <1 write"
call scenario "s36", "R", "R1"
call scenario "s37", "L5 n5", "L x", "L5 n5", "QL", "K", "L5 n5", "L3 n3", "L y", "S =2 write line"
call scenario "s38", "C3 Q", "K", "L x", "C3 Q", "K", "C3 Q", "I", "QL"
call scenario "s39", "S =2 write line", "L5 n5", "L y"
call scenario "s40", "N", "N", "S =7", "R3", "S =3 line", "QR"
call scenario "s41", "L3 n3", "S =5 write"
call scenario "s42", "C20 Z", "M", "R1", "S =2 write line", "N", "QR", "N", "L x", "C3 Q"
call scenario "s43", "K", "L x", "S =1 read", "C3 Q", "C9 RR"
call scenario "s44", "L5 n5", "S <1 write line", "S =3 line"
call scenario "s45", "L y", "R", "S <1 write line", "C3 Q", "QL"
call scenario "s46", "S =1 read", "C ab", "I"
call scenario "s47", "S =5 write", "R", "S =7", "K", "S <1 write"
call scenario "s48", "C3 Q", "QR", "L5 n5", "I", "QR"
call scenario "s49", "C3 Q", "QL", "QL", "C3 Q", "S =7", "S =3 line", "S =5 write"
call scenario "s50", "S <1 write", "L3 n3", "R1", "S =1 read", "S =3 line", "L3 n3", "S =1 read", "L1 n1"
call scenario "s51", "R", "QL", "L5 n5", "R", "C20 Z", "C9 RR", "S =5 write", "R3", "K"
call scenario "s52", "L x", "K", "S =3 line", "I", "S =3 line", "C3 Q", "R", "S =5 write"
call scenario "s53", "S =5 write", "S <1 write", "S =3 line", "L y", "C9 RR", "S =1 read"
call scenario "s54", "S =1 read", "L x", "L5 n5", "C20 Z"
call scenario "s55", "C3 Q", "S =7", "R", "C ab", "L1 n1", "L3 n3", "S =2 write line", "S =5 write", "S <1 write line"
call scenario "s56", "S <2 read line", "R1", "M", "S =3 line", "N", "L3 n3", "C9 RR", "K"
call scenario "s57", "S =7", "S <1 write line", "C3 Q", "C3 Q", "S <1 write line"
call scenario "s58", "L5 n5", "S <2 read line", "N", "L2 n2", "M", "I", "L5 n5"
call scenario "s59", "L1 n1", "R3", "S <1 write", "I", "QL", "R", "QR", "S =1 read"
call scenario "s60", "K", "C9 RR", "S <2 read line", "L1 n1", "L2 n2", "C20 Z"
call scenario "s61", "C ab", "S <1 write line", "R", "S <2 read line", "K"
call scenario "s62", "N", "R", "C3 Q", "S <2 read line"
call scenario "s63", "L3 n3", "S =3 line", "S <2 read line", "L3 n3", "C3 Q", "L2 n2", "L2 n2", "M"
call scenario "s64", "R", "L5 n5", "S =7"
call scenario "s65", "C20 Z", "L y", "S =3 line", "R3", "N", "S =7", "I", "L y", "S <1 write line"
call scenario "s66", "C ab", "C20 Z", "L x"
call scenario "s67", "S =1 read", "R3"
call scenario "s68", "K", "S <2 read line", "C20 Z", "R3", "M", "C ab", "L3 n3"
call scenario "s69", "L5 n5", "L2 n2", "C9 RR", "S =5 write", "N", "S =1 read", "L2 n2"
call scenario "s70", "L3 n3", "S <1 write line", "N", "S =3 line", "R", "R1", "C9 RR", "S <1 write line", "C3 Q"
call scenario "s71", "S <1 write line", "S <2 read line", "L5 n5"
call scenario "s72", "K", "S =3 line", "K", "I", "M"
call scenario "s73", "R", "C20 Z", "C3 Q"
call scenario "s74", "N", "L2 n2", "C9 RR", "M", "R"
call scenario "s75", "M", "R3", "L y", "R", "M", "L2 n2", "I", "C9 RR"
call scenario "s76", "L3 n3", "C20 Z", "C ab", "QR"
call scenario "s77", "L5 n5", "L1 n1", "L x", "R", "L2 n2", "C9 RR", "M", "S =1 read"
call scenario "s78", "C ab", "L x", "R", "S =5 write", "K", "S =5 write"
call scenario "s79", "L5 n5", "C ab", "N"
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
