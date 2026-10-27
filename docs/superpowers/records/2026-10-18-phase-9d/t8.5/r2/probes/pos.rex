q = .array~of('p','q')
k = .kmake~new
n = .nstr~new
d = .directory~new
s = 'plain'
num = 41
say '1 left ||' ((>q) || '<')
say '2 right ||' ('>' || (>q))
say '3 left abut' (>q)'<'
say '4 right abut' ('>')(>q)
say '5 left blank' (>q) '<'
say '6 right blank' '>' (>q)
say '7 say:'; say (>q)
say '8 len' length(>q)
say '9 pos' pos('q', >q)
say '10 substr' substr(>q, 1, 1)
say '11 kmake' (>k) || '|' ('|')(>k) length(>k)
say '12 nstr' (>n) || '|' ('|')(>n) length(>n)
say '13 dir' (>d) || '|' ('|')(>d) length(>d)
say '14 str' (>s) || '|' ('|')(>s) length(>s)
say '15 ~string' (>q)~string '/' (>k)~string '/' (>n)~string
say '16 ~makestring' (>k)~makestring
r = >q
say '17 req' r~request('STRING')
say '18 num right' 1 + (>num)
say '19 num left' (>num) + 1
say '20 abs' abs(>num) max(>num, 3) d2x(>num)
say '21 cmp' ((>s) = 'plain') ('plain' = (>s)) ((>s) == 'plain') ('plain' == (>s))
say '22 cmp k' ('kmake' = (>k)) ('kmake' == (>k))
::class kmake
::method makestring; return 'kmake'
::class nstr
::method string; return 'nstr'
