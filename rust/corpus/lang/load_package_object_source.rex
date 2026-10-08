/* loadPackage's source goes through arrayArgument: 88.913 for an object that
 * is not an array, and a string is taken. */
signal on syntax name s1
.context~package~loadPackage('nm', .object~new)
s1: say condition('o')~code condition('o')~message
signal on syntax name s2
.context~package~loadPackage('nm2', 'say 1')
say 'ok2'
exit
s2: say condition('o')~code condition('o')~message
