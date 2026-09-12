/* VALUE's two external pools: the process environment this interpreter holds,
   and .environment itself under the empty selector. Every line reads back what
   the line before it wrote, so a write that goes nowhere is visible. */
say '[' || value('P7_CORPUS_VAR', , 'ENVIRONMENT') || ']'
say '[' || value('P7_CORPUS_VAR', 'one', 'ENVIRONMENT') || ']'
say '[' || value('P7_CORPUS_VAR', , 'ENVIRONMENT') || ']'
say '[' || value('P7_CORPUS_VAR', , 'environment') || ']'
say '[' || value('P7_CORPUS_VAR', '', 'ENVIRONMENT') || ']'
say '[' || value('P7_CORPUS_VAR', , 'ENVIRONMENT') || ']'
say '[' || value('P7_CORPUS_VAR', .nil, 'ENVIRONMENT') || ']'
say '[' || value('P7_CORPUS_VAR', , 'ENVIRONMENT') || ']'
say '[' || value('P7_CORPUS_ENTRY', , '') || ']'
call value 'P7_CORPUS_ENTRY', 'stored', ''
say '[' || value('P7_CORPUS_ENTRY', , '') || ']'
say '[' || .P7_CORPUS_ENTRY || ']'
