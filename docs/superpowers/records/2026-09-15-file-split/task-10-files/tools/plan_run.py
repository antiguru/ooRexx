"""The assignment of BASE (36bbb3684) run.rs units to children: `impl Interp`
members by BASE line range, and the items outside that impl by key. Keys
survive earlier commits' line shifts, so later commits look them up by key.

usage: plan_run.py BASE_RUN_RS  -> writes plan-run.json beside this file
"""
import json, os, sys
import splitlib

METHOD_RANGES = {
    "indent": [(5435, 5447)],
    "select": [(5276, 5433)],
    "interpret": [(7595, 7854), (8010, 8029)],
    "settings": [(7856, 8008), (8031, 8250)],
    "condition": [(2856, 3791)],
    "call": [(3793, 4857)],
    "loops": [(5463, 7286)],
}
ITEMS = {
    "indent": ["fn fill_indents", "fn all_indents", "fn static_indent", "fn indent_in_range"],
    "raised": ["fn raised_symbol_expected", "fn raised_digit_led", "fn raised_dot_led",
               "fn raised_if_not_logical", "fn raised_guard_not_logical",
               "fn raised_when_not_logical", "fn raised_select_no_when",
               "fn raise_syntax_condition", "fn raised_naming_the_operand",
               "fn raised_from_settings", "fn raised_while_not_logical",
               "fn raised_until_not_logical", "fn raised_repetition_count_not_whole",
               "fn raised_for_count_not_whole", "fn raised_leave_no_loop",
               "fn raised_iterate_no_loop", "fn raised_leave_no_match",
               "fn raised_iterate_no_match", "fn raised_iterate_wrong_kind"],
    "select": ["enum Absorbed", "fn absorb", "struct IfTargets", "fn if_targets",
               "struct WhenTargets", "fn when_resume", "fn otherwise_resume", "fn when_targets",
               "struct SelectParts", "fn select_parts", "fn otherwise_range", "fn select_exit",
               "struct SelectResume", "enum SelectEscape", "fn select_escape", "fn skip_else"],
    "interpret": [],
    "settings": ["const MAX_ADDRESS_NAME_LENGTH"],
    "condition": ["fn condition_name"],
    "call": ["enum CallResolution", "enum Entered", "enum CallEntry", "fn entered_receiver",
             "const MAX_ACTIVATION_DEPTH"],
    "loops": ["fn control_slot", "fn numeric_less", "fn round_via_unary_plus"],
}
# The loop types, `DoOutcome` through `LoopHeaderValues`, with their impls.
ITEM_RANGES = {"loops": [(270, 699)]}
units = splitlib.load_units(sys.argv[1])
keys = {u["key"] for u in units}
plan = {}
for child in METHOD_RANGES.keys() | ITEMS.keys():
    methods = [u["key"] for u in units if u["key"].startswith("impl Interp::")
               and any(a <= u["first"] and u["last"] <= b for a, b in METHOD_RANGES.get(child, []))]
    for u in units:
        for a, b in METHOD_RANGES.get(child, []):
            if (a <= u["first"] <= b or a <= u["last"] <= b) and u["key"] not in methods:
                sys.exit(f"{child}: {u['key']} {u['first']}-{u['last']} straddles or is not an Interp member")
    ranged = [u["key"] for u in units
              if any(a <= u["first"] and u["last"] <= b for a, b in ITEM_RANGES.get(child, []))]
    ITEMS[child] = ranged + ITEMS.get(child, [])
    missing = [k for k in ITEMS.get(child, []) if k not in keys]
    assert not missing, (child, missing)
    plan[child] = {"items": ITEMS.get(child, []), "methods": methods}
json.dump(plan, open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "plan-run.json"), "w"), indent=1)
for c, p in sorted(plan.items()):
    print(c, len(p["items"]), "items", len(p["methods"]), "methods")
