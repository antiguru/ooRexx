r = .context~package~findRoutine('SYSVERSION')
say r~class
if r \= .nil then say r~package~name
say .context~package~importedPackages~items
do p over .context~package~importedPackages; say p~name; end
signal on syntax
::requires 'rexxutil' library
