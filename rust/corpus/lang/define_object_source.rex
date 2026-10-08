/* define with a method that is neither a string, an array nor a Method
 * object: 93.974. */
signal on syntax
.object~subclass('K')~define('M', .object~new)
say 'no'
exit
syntax: say rc condition('O')~code
