q = .array~of('p','q')
say 'request' (>q)~request('STRING')~changestr('0a'x, ',')
signal on nostring
say 'say' (>q)
exit
nostring: say 'NOSTRING' condition('d'); exit
