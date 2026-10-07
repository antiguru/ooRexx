/* COUNTER on a controlled loop: the passes entered, not the control's end
   value; an ITERATE still enters the next pass, and nested counters are
   separate. */
do counter c i = 1 to 3; end; say c i
do counter c i = 1 to 3; if i = 2 then iterate; end; say c
do counter o 2; do counter n 3; end; say o n; end
