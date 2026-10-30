import sys
orig = open('/tmp/claude-1000/t87c-rr2-eval.rs.orig').read()
parse_body = """        let probe = 0u8;
        self.enter_eval_node(&raw const probe)?;
        let value = self.eval_node(code, expr);
        self.activity.depth -= 1;
        if let Ok(v) = &value {
            self.trace_intermediate(code, expr, *v)?;"""
eval_body = """        let probe = 0u8;
        self.enter_eval_node(&raw const probe)?;
        let value = self.eval_node(code, expr);
        self.activity.depth -= 1;
        // `TRACE I`'s own value events (D17), inserted here and in"""
assert orig.count(parse_body) == 1 and orig.count(eval_body) == 1
m = sys.argv[1]
def sub(old, a, b):
    assert old.count(a) == 1, (m, a)
    return old.replace(a, b)
if m == 'A': new = sub(orig, parse_body, parse_body.replace("        self.activity.depth -= 1;\n", ""))
elif m == 'B': new = sub(orig, parse_body, parse_body.replace("        let value = self.eval_node(code, expr);\n        self.activity.depth -= 1;\n", "        self.activity.depth -= 1;\n        let value = self.eval_node(code, expr);\n"))
elif m == 'C': new = sub(orig, parse_body, parse_body.replace("        self.enter_eval_node(&raw const probe)?;\n", "        let _ = probe; self.activity.depth += 1;\n"))
elif m == 'E': new = sub(orig, eval_body, eval_body.replace("        let value = self.eval_node(code, expr);\n        self.activity.depth -= 1;\n", "        self.activity.depth -= 1;\n        let value = self.eval_node(code, expr);\n"))
elif m == 'F': new = sub(orig, parse_body, parse_body.replace("        self.enter_eval_node(&raw const probe)?;\n", "        let _ = probe;\n").replace("        self.activity.depth -= 1;\n", ""))
elif m == 'O': new = orig
open('eval.rs', 'w').write(new)
