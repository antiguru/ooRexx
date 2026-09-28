#include "oorexxapi.h"
static RexxCallContext *outer = NULL;
RexxRoutine1(RexxObjectPtr, Outer9, RexxObjectPtr, o)
{
    outer = context;
    return context->SendMessage0(o, "RUN");
}
RexxRoutine0(RexxObjectPtr, UseOuterVar)
{
    RexxObjectPtr v = outer->GetContextVariable("X");
    return v == NULLOBJECT ? context->String("null") : v;
}
RexxRoutine0(RexxObjectPtr, UseOuterSet)
{
    outer->SetContextVariable("Y", context->String("set-by-inner"));
    return context->String("ok");
}
RexxRoutine0(RexxObjectPtr, StaleRet)
{
    RexxObjectPtr s = context->String("released");
    context->ReleaseLocalReference(s);
    return s;
}
RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(StaleRet, StaleRet),
    REXX_TYPED_ROUTINE(Outer9, Outer9),
    REXX_TYPED_ROUTINE(UseOuterVar, UseOuterVar),
    REXX_TYPED_ROUTINE(UseOuterSet, UseOuterSet),
    REXX_LAST_ROUTINE()
};
RexxPackageEntry outer9_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "outer9", "1.0.0", NULL, NULL, routines, NULL
};
OOREXX_GET_PACKAGE(outer9);
