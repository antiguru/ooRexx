f = 'tz.txt'
call lineout f, 'x'
call lineout f
call value 'TZ', 'UTC', 'ENVIRONMENT'
say SysSetFileDateTime(f, '2020-06-01', '12:00:00') SysGetFileDateTime(f)
call value 'TZ', 'Asia/Kolkata', 'ENVIRONMENT'
say SysGetFileDateTime(f)
say SysSetFileDateTime(f, '2020-06-01', '12:00:00') SysGetFileDateTime(f)
call value 'TZ', 'UTC', 'ENVIRONMENT'
say SysGetFileDateTime(f)
