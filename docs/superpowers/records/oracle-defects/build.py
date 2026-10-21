"""Builds the oracle-defects page from defects.json: python3 build.py OUT.html"""
import json, sys

here = __file__.rsplit("/", 1)[0] or "."
defects = json.load(open(f"{here}/defects.json"))
page = open(f"{here}/page.template.html").read()
data = json.dumps(defects, ensure_ascii=False).replace("</", "<\\/")
updated = max(d["added"] for d in defects)
open(sys.argv[1], "w").write(page.replace("__DATA__", data).replace("__UPDATED__", updated))
