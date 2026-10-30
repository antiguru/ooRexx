/* SysSetFileDateTime and SysGetFileDateTime: dates and times that scan and
   ones that do not, fields past their range carrying into the next, a
   reading the local zone skips and one it repeats (each also as seconds
   since the epoch for ordinary and skipped readings, and a validity
   property for the two allowed instants of a repeated reading), years before 1000 and before 1970, the access time a set keeps,
   either half omitted, links, a directory, names that resolve to nothing,
   and the argument errors. The daylight-saving lines are the zone's own
   transitions in Europe and in North America. Last, TZ is set by the
   program itself, to zones including one whose daylight saving is winter
   time, a POSIX rule, an empty value and a name no file has. Everything is
   made under a directory this program makes and removes. */
d = 'datetime.d'
call SysMkDir d
f = d'/a.txt'
call lineout f, 'x'
call lineout f
call s '2020-03-04', '05:06:07'
call s '2020-03-29', '02:30:00', 'epoch'
call s '2020-03-29', '01:59:59', 'epoch'
call s '2020-03-29', '03:00:00', 'epoch'
call overlap 'Europe/Zurich', '2020-10-25', '02:30:00', '1603585800 1603589400'
call s '2020-10-25', '01:59:59', 'epoch'
call s '2020-10-25', '03:00:00', 'epoch'
call s '2020-03-08', '02:30:00', 'epoch'
call overlap 'America/New_York', '2020-11-01', '01:30:00', '1604208600 1604212200'
call s '2020-07-01', '12:00:00', 'epoch'
call s '2020-13-01', '00:00:00'
call s '2020-02-30', '00:00:00'
call s '2020-00-00', '00:00:00'
call s '2020-01-01', '24:00:00'
call s '2020-01-01', '99:99:99'
call s '2020-1-2', '3:4:5'
call s ' 2020-01-02', ' 3: 4: 5'
call s '2020-01-02junk', '03:04:05junk'
call s '20200-01-01', '00:00:00'
call s '+2020-01-01', '00:00:00'
call s '+202-01-01', '00:00:00'
call s '-202-01-01', '00:00:00', 'epoch'
call s '1800-01-01', '00:00:00', 'epoch'
call s '0999-05-06', '00:00:00'
call s '1969-12-31', '23:59:59', 'epoch'
call s '2020-01-01', '05:06'
call s '', ''
call s '2021-06-07', ''
call s '2021-06-08'
call s , '11:12:13'
call s '2020-1-1', '1:2:3' || '00'x || 'x'
call s '2020-1-1' || '00'x || 'x', '1:2:3'
call s 'a020-01-01', '00:00:00'
call s '2020-01-01', '00:00:-1'
call s '2020-01-01', '-1:00:00'
call s '2020-01-01', '+1:+2:+3'
call s '2020-01-01', '1: 2:3'
call s '2020-01-01', '1 :2:3'
call s '2020-01-01', '1:2:'
call s '2020-01-01', '-:2:3'
'touch -a -d @1000000000' f
say 'keeps access' SysSetFileDateTime(f, '2011-12-13', '14:15:16')
'stat -c "%X %Y"' f
say 'now         ' SysSetFileDateTime(f) (left(SysGetFileDateTime(f), 10) == date('I'))
'stat -c "%X"' f
call SysSetFileDateTime f, '2002-02-02', '02:02:02'
say 'selectors   ' SysGetFileDateTime(f, 'w') SysGetFileDateTime(f, 'Write') ,
  SysGetFileDateTime(f, 'axe') SysGetFileDateTime(f, 'A' || '00'x)
'ln -s a.txt' d'/link'
'ln -s nowhere' d'/dangling'
say 'link        ' SysSetFileDateTime(d'/link', '2003-03-03', '03:03:03') SysGetFileDateTime(f)
say 'dangling    ' SysSetFileDateTime(d'/dangling', '2003-03-03', '03:03:03') SysGetFileDateTime(d'/dangling')
say 'directory   ' SysSetFileDateTime(d, '2004-04-04', '04:04:04') SysGetFileDateTime(d) SysGetFileDateTime(d'/')
say 'blank, utf8 ' SysFileCopy(f, d'/a b' || 'c3a9ff'x) SysSetFileDateTime(d'/a b' || 'c3a9ff'x, '2005-05-05', '05:05:05') ,
  SysGetFileDateTime(d'/a b' || 'c3a9ff'x)
say 'missing     ' SysSetFileDateTime(d'/none', '2020-01-01', '00:00:00') SysSetFileDateTime(d'/none') ,
  SysGetFileDateTime(d'/none') SysGetFileDateTime('') SysSetFileDateTime('')
call SysMkDir d'/closed'
call SysFileCopy f, d'/closed/in.txt'
'chmod 000' d'/closed'
say 'closed      ' SysGetFileDateTime(d'/closed/in.txt') SysSetFileDateTime(d'/closed/in.txt', '2006-06-06')
'chmod 700' d'/closed'
call t "call SysGetFileDateTime"
call t "call SysGetFileDateTime f, ''"
call t "call SysGetFileDateTime f, 'x'"
call t "call SysGetFileDateTime d'/none', 'x'"
call t "call SysGetFileDateTime f, .nil"
call t "call SysGetFileDateTime f, 'w', 'x'"
call t "call SysSetFileDateTime"
call t "call SysSetFileDateTime f, .nil"
call t "call SysSetFileDateTime f, , .nil"
call t "call SysSetFileDateTime f, 1, 2, 3"
call t "r = SysSetFileDateTime(f, '2020-01-01'); say 'returned' r"
-- the program's own TZ, set through the environment, is the zone both read
call value 'TZ', 'Europe/Dublin', 'ENVIRONMENT'
call s '2020-03-29', '01:30:00', 'epoch'
call s '2020-03-29', '00:59:59', 'epoch'
call s '2020-03-29', '02:00:00', 'epoch'
call overlap 'Europe/Dublin', '2020-10-25', '01:30:00', '1603585800 1603589400'
call s '2021-03-28', '01:30:00', 'epoch'
call s '2045-03-26', '01:30:00', 'epoch'
call value 'TZ', 'UTC', 'ENVIRONMENT'
call s '2020-06-01', '12:00:00', 'epoch'
call value 'TZ', 'Asia/Kolkata', 'ENVIRONMENT'
say 'kolkata     ' SysGetFileDateTime(f)
call s '2020-06-01', '12:00:00', 'epoch'
call value 'TZ', 'IST-1GMT0,M10.5.0,M3.5.0/1', 'ENVIRONMENT'
call s '2020-03-29', '01:30:00', 'epoch'
call s '2020-07-01', '12:00:00', 'epoch'
call value 'TZ', ':Europe/Zurich', 'ENVIRONMENT'
call s '2020-03-29', '02:30:00', 'epoch'
call value 'TZ', '', 'ENVIRONMENT'
call s '2020-07-01', '12:00:00', 'epoch'
call value 'TZ', 'No/Such_Zone', 'ENVIRONMENT'
call s '2020-07-01', '12:00:00', 'epoch'
'rm -rf' d
call SysGetFileDateTime f, 'Q'
exit

s: procedure expose f
  if arg(1, 'e') & arg(2, 'e') then r = SysSetFileDateTime(f, arg(1), arg(2))
  else if arg(1, 'e') then r = SysSetFileDateTime(f, arg(1))
  else r = SysSetFileDateTime(f, , arg(2))
  say left('['translate(arg(1), '?', '00'x)'] ['translate(arg(2), '?', '00'x)']', 34) r ,
    SysGetFileDateTime(f) SysGetFileDateTime(f, 'W')
  if arg(3) == 'epoch' then 'stat -c "  %Y"' f
  if arg(3) == 'overlap' then do
    address system 'stat -c %Y' f with output stem epoch.
    say '  valid overlap' (wordpos(strip(epoch.1), arg(4)) > 0) ,
      (SysGetFileDateTime(f) == arg(1)' 'arg(2))
  end
  return

-- The reference does not select one occurrence of a repeated local reading.
-- A named zone fixes the allowed pair without depending on prior file atime.
overlap: procedure expose f
  use arg zone, day, clock, candidates
  saved = value('TZ', , 'ENVIRONMENT')
  call value 'TZ', zone, 'ENVIRONMENT'
  call s day, clock, 'overlap', candidates
  call value 'TZ', saved, 'ENVIRONMENT'
  return

t: procedure expose f d
  parse arg code
  signal on syntax name bad
  interpret code
  say 'ok   ' code
  return
bad:
  c = condition('O')
  say 'error' code '|' c~code c~message
  return
