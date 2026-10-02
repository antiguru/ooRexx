#include "oorexxapi.h"
#include <stdio.h>
static RexxCallContext *outer = NULL;
RexxRoutine1(RexxObjectPtr, Outer, RexxObjectPtr, o)
{
    outer = context;
    return context->SendMessage0(o, "RUN");
}
RexxRoutine1(RexxObjectPtr, OM, CSTRING, which)
{
    char buf[64];
    switch (which[0]) {
    case 'a': { RexxArrayObject a = outer->GetArguments(); return a == NULL ? context->String("nullarr") : context->WholeNumberToObject(context->ArraySize(a)); }
    case 'b': { RexxObjectPtr a = outer->GetArgument(1); return a == NULL ? context->String("null") : a; }
    case 'n': { CSTRING n = outer->GetRoutineName(); return context->String(n == NULL ? "nullname" : n); }
    case 'r': { RexxRoutineObject r = outer->GetRoutine(); return r == NULL ? context->String("nullr") : (RexxObjectPtr)r; }
    case 'd': { snprintf(buf, sizeof buf, "%u", (unsigned)outer->GetContextDigits()); return context->String(buf); }
    case 'f': { snprintf(buf, sizeof buf, "%u", (unsigned)outer->GetContextFuzz()); return context->String(buf); }
    case 'o': { snprintf(buf, sizeof buf, "%u", (unsigned)outer->GetContextForm()); return context->String(buf); }
    case 'c': { RexxObjectPtr c = outer->GetCallerContext(); return c == NULL ? context->String("nullc") : c; }
    case 'k': { RexxClassObject c = outer->FindContextClass("C"); return c == NULL ? context->String("nullk") : (RexxObjectPtr)c; }
    case 's': { RexxStemObject s = outer->ResolveStemVariable(context->String("S.")); return s == NULL ? context->String("nulls") : (RexxObjectPtr)s; }
    case 'v': { RexxVariableReferenceObject v = outer->GetContextVariableReference("X"); return v == NULL ? context->String("nullv") : (RexxObjectPtr)v; }
    case 'i': { outer->InvalidRoutine(); return context->String("invalid"); }
    case 't': { outer->ThrowException1(40001, context->String("x")); return context->String("thrown"); }
    case 'S': { RexxObjectPtr s = outer->String("hello"); return s == NULL ? context->String("nullS") : s; }
    case 'g': { RexxObjectPtr v = outer->GetContextVariable("X"); return v == NULL ? context->String("null") : v; }
    }
    return context->String("?");
}
RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(Outer, Outer),
    REXX_TYPED_ROUTINE(OM, OM),
    REXX_LAST_ROUTINE()
};
RexxPackageEntry outer7_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "outer7", "1.0.0", NULL, NULL, routines, NULL
};
OOREXX_GET_PACKAGE(outer7);
