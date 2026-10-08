/* .context~executable in a class method and in an instance method. */
say .k~go
say .k~new~go2 == .k~method('GO2')
say .k~class~id
::class k
::method go class
  return .context~executable~class~id .context~executable~source~items
::method go2
  return .context~executable
