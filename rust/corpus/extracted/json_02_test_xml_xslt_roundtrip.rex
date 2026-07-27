/* extracted from json_02::test_xml_xslt_roundtrip */
::routine main public
  expose thisLocation executableLocation

  /* Parse the reference JSON file */
  json = .json~new
  constructsFile = thisLocation || "test_all_constructs.json"
  obj1 = .json~fromJsonFile(constructsFile)

  /* Generate XSD and DTD XML files */
  xsdFile = thisLocation"test_xslt_xsd_tmp.xml"
  dtdFile = thisLocation"test_xslt_dtd_tmp.xml"
  .json~jsonToXmlFile(obj1, xsdFile, "xsd")
  .json~jsonToXmlFile(obj1, dtdFile, "dtd")

  /* json.dtd must be findable relative to the DTD XML file */
  dtdSchemaFile = thisLocation"json.dtd"
  needDtdCleanup = .false
  If SysFileExists(thisLocation"docbook/json.dtd"), \SysFileExists(dtdSchemaFile) Then Do
    address system "cp" thisLocation"docbook/json.dtd" dtdSchemaFile
    needDtdCleanup = .true
  End

  /* Try xsltproc first, then runXSLT.rxj */
  xsltAvailable = .false
  xslFile = executableLocation"xmlToJson.xsl"
  If \SysFileExists(xslFile) Then
    xslFile = thisLocation"docbook/xmlToJson.xsl"

  /* Attempt xsltproc */
  Signal On Syntax Name TryRunXSLT_json
  outXsd = .array~new
  errXsd = .array~new
  address system "xsltproc" xslFile xsdFile -
    with output using (outXsd) error using (errXsd)
  If outXsd~items > 0 Then Do
    xsltAvailable = .true
    xsltTool = "xsltproc"
  End
  Signal XsltCheckDone_json

TryRunXSLT_json:
  /* Attempt runXSLT.rxj (requires BSF4ooRexx) */
  Signal On Syntax Name NoXsltTool_json
  outXsd = .array~new
  errXsd = .array~new
  address system "rexx runXSLT.rxj" xslFile xsdFile -
    with output using (outXsd) error using (errXsd)
  If outXsd~items > 0 Then Do
    xsltAvailable = .true
    xsltTool = "runXSLT.rxj"
  End
  Signal XsltCheckDone_json

NoXsltTool_json:
  Signal Off Syntax

XsltCheckDone_json:
  Signal Off Syntax

  If \xsltAvailable Then Do
    /* No XSLT processor — clean up and skip */
    Call SysFileDelete xsdFile
    Call SysFileDelete dtdFile
    If needDtdCleanup Then Call SysFileDelete dtdSchemaFile
    Return
  End

  /* XSD round-trip: parse the XSLT output and compare */
  xsdJson = outXsd~makeString('L', "0A"x)
  xsdDoc = json~fromJSON(xsdJson)
  xsdOrigJson = json~toJSON(obj1)
  xsdRtJson   = json~toJSON(xsdDoc)
  self~assertEquals(xsdOrigJson, xsdRtJson, -
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
  dtdJson = outDtd~makeString('L', "0A"x)
  dtdDoc = json~fromJSON(dtdJson)
  dtdRtJson = json~toJSON(dtdDoc)
  self~assertEquals(xsdOrigJson, dtdRtJson, -
    "XSLT dtd round-trip ("xsltTool")")

  /* Clean up temporary files */
  Call SysFileDelete xsdFile
  Call SysFileDelete dtdFile
  If needDtdCleanup Then Call SysFileDelete dtdSchemaFile


/*============================================================================*/
/*  Shared routines                                                           */
/*============================================================================*/

/** Recursive deep-equality comparison for JSON-decoded structures.
 *  Handles .Directory, .Array, .JsonBoolean, .JsonString, .String, .nil.
 */
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
