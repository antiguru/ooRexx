/* extracted from yaml::test37_XsltRoundtrip */
::routine main public
  expose parser thisLocation executableLocation

  /* Parse the reference YAML file */
  parser2 = .Yaml~new
  original = parser2~parseFile(thisLocation"test_all_constructs.yaml")
  am = parser2~anchorMap

  /* Generate XSD and DTD XML files */
  xsdFile = thisLocation"test33_xsd.xml"
  dtdFile = thisLocation"test33_dtd.xml"
  .Yaml~yamlToXmlFile(original, xsdFile, "xsd", am)
  .Yaml~yamlToXmlFile(original, dtdFile, "dtd", am)

  /* Try xsltproc first, then runXSLT.rxj */
  xsltAvailable = .false
  xslFile = executableLocation"xmlToYaml.xsl"

  /* Attempt xsltproc */
  Signal On Syntax Name TryRunXSLT
  outXsd = .array~new
  errXsd = .array~new
  address system "xsltproc" xslFile xsdFile -
    with output using (outXsd) error using (errXsd)
  If outXsd~items > 0 Then Do
    xsltAvailable = .true
    xsltTool = "xsltproc"
  End
  Signal XsltCheckDone

TryRunXSLT:
  /* Attempt runXSLT.rxj (requires BSF4ooRexx) */
  Signal On Syntax Name NoXsltTool
  outXsd = .array~new
  errXsd = .array~new
  address system "rexx runXSLT.rxj" xslFile xsdFile -
    with output using (outXsd) error using (errXsd)
  If outXsd~items > 0 Then Do
    xsltAvailable = .true
    xsltTool = "runXSLT.rxj"
  End
  Signal XsltCheckDone

NoXsltTool:
  Signal Off Syntax

XsltCheckDone:
  Signal Off Syntax

  If \xsltAvailable Then Do
    /* No XSLT processor — clean up and skip */
    Call SysFileDelete xsdFile
    Call SysFileDelete dtdFile
    Return
  End

  /* XSD round-trip: parse the XSLT output and compare */
  xsdYaml = outXsd~makeString('L', "0A"x)
  parser3 = .Yaml~new
  xsdDoc = parser3~parseString(xsdYaml)
  self~assertTrue(YAML.deepEqual(original, xsdDoc), -
    "XSLT xsd round-trip ("xsltTool")")

  /* DTD round-trip */
  outDtd = .array~new
  errDtd = .array~new
  If xsltTool == "xsltproc" Then
    address system "xsltproc" xslFile dtdFile -
      with output using (outDtd) error using (errDtd)
  Else
    address system "rexx runXSLT.rxj" xslFile dtdFile -
      with output using (outDtd) error using (errDtd)
  dtdYaml = outDtd~makeString('L', "0A"x)
  parser4 = .Yaml~new
  dtdDoc = parser4~parseString(dtdYaml)
  self~assertTrue(YAML.deepEqual(original, dtdDoc), -
    "XSLT dtd round-trip ("xsltTool")")

  /* Clean up temporary XML files */
  Call SysFileDelete xsdFile
  Call SysFileDelete dtdFile

/*------------------------------------------------------------------------*/
/* 38. Complex mapping keys (? indicator)                                 */
/*------------------------------------------------------------------------*/
::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
