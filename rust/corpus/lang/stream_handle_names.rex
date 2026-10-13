/* A HANDLE:n stream: the answers that do not read or write the descriptor,
   which agree with the oracle. Reads, writes, QUERY HANDLE after an open and
   a second open are Deviation 35, and nothing here touches them. */
do name over .array~of('HANDLE:0', 'handle:1', 'HANDLE:2')
  s = .Stream~new(name)
  say name 'init' s~state s~description s~string
  say name 'unopened' '[' || s~query('streamtype') || ']' '[' || s~query('handle') || ']' -
    '[' || s~query('exists') || ']' (s~qualify == .File~new(name)~absolutePath)
  say name 'open' s~open('read') s~state s~description
  say name 'opened' '[' || s~query('exists') || ']' '[' || s~qualify || ']'
  say name 'close' s~close s~state s~description '[' || s~query('exists') || ']' '[' || s~qualify || ']'
end

say '-- the descriptor is an int'
do name over .array~of('HANDLE:01', 'HANDLE: 1', 'HANDLE:+2', 'HANDLE:1.0', 'HANDLE:1e0', -
    'HANDLE:x', 'HANDLE:', 'HANDLE:1.5', 'HANDLE:2147483648', 'HANDLE:2.0000000000000000001')
  call try name
end

say '-- open takes the options a file takes'
s = .Stream~new('HANDLE:0')
call badopen s
exit

try:
  signal on syntax name refused
  s = .Stream~new(arg(1))
  say '[' || arg(1) || '] init' s~state
  return
refused:
  say '[' || arg(1) || ']' condition('o')~code condition('o')~message
  return

badopen:
  signal on syntax name badopened
  say arg(1)~open('zork')
  return
badopened:
  say 'open zork' condition('o')~code
  return
