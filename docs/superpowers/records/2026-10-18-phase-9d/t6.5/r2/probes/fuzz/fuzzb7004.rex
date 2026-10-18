parse arg dummy
call scenario "s0", "S -2 write", "L3 n3", "C3 Q", "N"
call scenario "s1", "C ab", "S =7", "S =7", "R1", "L2 n2"
call scenario "s2", "L y", "S <2 read line", "S -2 write", "C3 Q", "S +1 write line", "QR"
call scenario "s3", "S =2 write line", "L y", "S <2 read line", "QL", "C3 Q", "L3 n3", "S +1 write line", "S +1 read line"
call scenario "s4", "QR", "R3", "L1 n1", "R", "S <2 read line", "L3 n3", "S =7"
call scenario "s5", "C ab", "N", "M", "L x", "S +3 read", "K"
call scenario "s6", "S -2 write", "S =7", "S +1 write line", "L1 n1", "QR", "S =7", "C3 Q", "N", "R3"
call scenario "s7", "S =2 write line", "L y", "S +1 read line", "M", "M", "L2 n2"
call scenario "s8", "QL", "L y"
call scenario "s9", "K", "QL"
call scenario "s10", "S =5 write", "S +3 read", "K", "S +1 write line"
call scenario "s11", "L1 n1", "R3", "S +1 write line", "S +1 write line", "S +1 write line", "L y"
call scenario "s12", "C3 Q", "L1 n1", "R1", "S =1 read", "R3", "K", "S <1 write line", "S <2 read line"
call scenario "s13", "K", "S +3 read", "S <2 read line", "K", "S +1 write line", "I", "S <1 write", "S <1 write line"
call scenario "s14", "K", "QR", "C ab"
call scenario "s15", "C3 Q", "S =5 write", "N", "S =5 write", "L1 n1", "S +1 read line", "L1 n1", "S =7", "L3 n3"
call scenario "s16", "S =5 write", "R1", "L5 n5", "S =7", "R", "S =1 read"
call scenario "s17", "L x", "C ab"
call scenario "s18", "S <1 write line", "M", "S +1 write line"
call scenario "s19", "L y", "L3 n3", "K", "S +1 write line", "QR", "S =7", "I", "C ab"
call scenario "s20", "S +1 read line", "S =7", "L2 n2", "S <2 read line"
call scenario "s21", "L5 n5", "R3", "K", "C20 Z", "QL", "S =2 write line", "L x", "L3 n3", "L1 n1"
call scenario "s22", "L y", "C3 Q", "S <1 write line", "L x", "S <2 read line", "M", "L2 n2", "C3 Q"
call scenario "s23", "L3 n3", "S <1 write", "I", "S =7"
call scenario "s24", "S <1 write", "L1 n1", "L2 n2"
call scenario "s25", "S =1 read", "L y", "L2 n2", "S +3 read", "S +1 read line", "C9 RR", "R", "S <1 write line", "I"
call scenario "s26", "R", "L x", "S -2 write", "K", "QL", "M", "S +3 read"
call scenario "s27", "S =7", "L1 n1", "S =7", "S +1 read line", "S =1 read", "R1", "C20 Z", "QL", "L5 n5"
call scenario "s28", "QL", "S =7", "S =3 line", "S +1 read line", "S =1 read", "L y", "M", "C20 Z"
call scenario "s29", "S =7", "S +1 write line", "S =1 read", "L y", "S <2 read line", "L3 n3", "S +1 read line", "S +1 read line", "S -2 write"
call scenario "s30", "I", "QL", "R", "C3 Q"
call scenario "s31", "C9 RR", "S <1 write", "S <1 write"
call scenario "s32", "QL", "S =7", "QL"
call scenario "s33", "L x", "C3 Q", "C20 Z", "N", "QL", "S =3 line", "S =7", "N", "S <1 write"
call scenario "s34", "S <2 read line", "L2 n2", "S =3 line", "L2 n2", "L x", "C ab", "S <1 write line", "L x", "S =2 write line"
call scenario "s35", "R3", "QL", "L3 n3", "S =1 read", "QR"
call scenario "s36", "R", "L2 n2", "S =3 line", "S <1 write", "S +1 write line", "N", "S +3 read"
call scenario "s37", "L3 n3", "S -2 write", "R1", "N", "L2 n2", "L x", "S <1 write", "L2 n2"
call scenario "s38", "C20 Z", "S =3 line", "S +3 read", "C3 Q", "S +1 read line", "N", "S =1 read", "L3 n3", "S =7"
call scenario "s39", "S =7", "L y", "I", "C9 RR", "L y", "N"
call scenario "s40", "S <1 write line", "R1"
call scenario "s41", "S =1 read", "L x", "L5 n5", "C9 RR", "L1 n1", "C20 Z", "S +1 read line"
call scenario "s42", "S =5 write", "L1 n1"
call scenario "s43", "S <2 read line", "S +1 read line", "S =1 read", "S +1 write line", "S <1 write", "L5 n5"
call scenario "s44", "S =3 line", "R3", "S =5 write", "R"
call scenario "s45", "S +1 write line", "M", "L y", "N", "C20 Z"
call scenario "s46", "S <2 read line", "S -2 write", "S -2 write", "K", "C9 RR", "S =7"
call scenario "s47", "QR", "QR"
call scenario "s48", "C9 RR", "I", "M"
call scenario "s49", "S +1 read line", "QR"
call scenario "s50", "L y", "S +1 write line", "C9 RR", "L x"
call scenario "s51", "S <1 write", "M", "S +1 write line"
call scenario "s52", "S <2 read line", "S +1 write line", "C9 RR", "S =3 line", "S =5 write", "K", "L5 n5", "L y"
call scenario "s53", "S =1 read", "L x", "C3 Q"
call scenario "s54", "S +3 read", "R3", "S +1 read line", "S +3 read"
call scenario "s55", "R", "R", "S =2 write line", "L5 n5"
call scenario "s56", "S +1 read line", "S =2 write line", "S +1 read line", "S <1 write line", "S =1 read", "L3 n3", "N"
call scenario "s57", "S =2 write line", "S -2 write", "L3 n3", "R3"
call scenario "s58", "S +1 write line", "S =2 write line"
call scenario "s59", "S +3 read", "C3 Q", "L1 n1", "S <1 write", "C ab", "S =3 line"
call scenario "s60", "S +1 write line", "S =5 write", "C9 RR", "S -2 write", "C20 Z", "C9 RR", "S =7", "S -2 write"
call scenario "s61", "S <1 write", "QL", "S =5 write", "M", "QR", "R3", "C3 Q", "L5 n5", "N"
call scenario "s62", "K", "S +1 write line"
call scenario "s63", "S +3 read", "S -2 write", "R1"
call scenario "s64", "QL", "S +3 read", "S =1 read", "QR", "S +1 write line", "R", "QR", "C3 Q", "R"
call scenario "s65", "L5 n5", "I", "R", "S =3 line", "M", "L x", "K", "S +3 read", "L1 n1"
call scenario "s66", "QL", "C ab", "S =1 read", "R1", "QR"
call scenario "s67", "S +1 write line", "S =2 write line", "L2 n2", "R3", "R3", "S +1 write line", "N", "S <2 read line"
call scenario "s68", "R", "S =7"
call scenario "s69", "S -2 write", "C20 Z", "L5 n5", "S -2 write", "S =7", "N"
call scenario "s70", "I", "L x", "S =2 write line", "S =5 write", "S <2 read line", "R", "L y", "QR", "L x"
call scenario "s71", "L1 n1", "R"
call scenario "s72", "L3 n3", "C9 RR", "S +3 read"
call scenario "s73", "S +3 read", "I", "S =3 line"
call scenario "s74", "R", "QR"
call scenario "s75", "L2 n2", "L y", "S +1 write line", "QR", "C ab", "R1", "I"
call scenario "s76", "S -2 write", "S <1 write", "S =3 line", "L x"
call scenario "s77", "R1", "S +1 read line", "S -2 write", "S =3 line", "S -2 write", "S +1 write line"
call scenario "s78", "S <1 write line", "R3", "K", "L x", "I", "S =2 write line", "S +3 read"
call scenario "s79", "L y", "R1", "S <2 read line"
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
