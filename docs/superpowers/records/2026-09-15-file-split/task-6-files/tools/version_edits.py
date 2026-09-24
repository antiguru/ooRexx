"""The version-constants move's edits outside the moved block: the imports
parse_template.rs and its test module need, and every path that named a
version constant through `parse_template` renamed to `version`.

usage: version_edits.py SRC_DIR
"""
import re, sys
src = sys.argv[1]
def edit(rel, f):
    p = f"{src}/{rel}"; s = open(p).read(); t = f(s); assert t != s, rel; open(p, "w").write(t)
edit("parse_template.rs", lambda s: s.replace("use crate::{Code, Interp, Loud};\n",
     "use crate::version::{PLATFORM, VERSION};\nuse crate::{Code, Interp, Loud};\n", 1))
edit("parse_template/tests.rs", lambda s: s.replace("use super::*;\n",
     "use super::*;\nuse crate::version::{\n    BIT_WIDTH, BUILD_DATE, LANGUAGE_LEVEL, MAJOR_VERSION, MODIFICATION, RELEASE, VERSION_NUMBER,\n};\n", 1))
edit("dispatch/rexx_info.rs", lambda s: re.sub(r"\bparse_template\b", "version", s))
edit("trace.rs", lambda s: s.replace("crate::parse_template::PLATFORM", "crate::version::PLATFORM"))
edit("environment.rs", lambda s: s.replace("crate::parse_template::LINE_END", "crate::version::LINE_END"))
