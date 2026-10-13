do n over .array~of('HANDLE:1', 'HANDLE:01', 'HANDLE: 1', 'HANDLE:1 ', 'HANDLE:+1', 'HANDLE:1.0', 'HANDLE:1.5', 'HANDLE:1e0', 'HANDLE:', 'HANDLE:2147483648', 'HANDLE:9999999999', 'HANDLE:0.0000000001', 'HANDLE:1.0000000001', 'HANDLE:1.000000000000000001', 'HANDLE:2.0000000000000000001', 'HANDLE:2147483647.4')
  call try n
end
exit
try:
  signal on syntax
  s = .Stream~new(arg(1))
  say '[' || arg(1) || '] ok' s~state
  return
syntax:
  say '[' || arg(1) || ']' condition('o')~code condition('o')~message
  return
