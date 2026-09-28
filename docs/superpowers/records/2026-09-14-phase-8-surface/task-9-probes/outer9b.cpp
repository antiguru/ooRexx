#include "oorexxapi.h"
static RexxCallContext *outer = NULL;
RexxRoutine1(RexxObjectPtr, Outer, RexxObjectPtr, o)
{
    outer = context;
    return context->SendMessage0(o, "RUN");
}
RexxRoutine1(RexxObjectPtr, OGet, CSTRING, name)
{
    RexxObjectPtr v = outer->GetContextVariable(name);
    return v == NULLOBJECT ? context->String("null") : v;
}
RexxRoutine2(RexxObjectPtr, OSet, CSTRING, name, RexxObjectPtr, value)
{
    outer->SetContextVariable(name, value);
    return context->String("set");
}
RexxRoutine1(RexxObjectPtr, ODrop, CSTRING, name)
{
    outer->DropContextVariable(name);
    return context->String("dropped");
}
RexxRoutine0(RexxObjectPtr, OAll)
{
    return outer->GetAllContextVariables();
}
RexxRoutine1(RexxObjectPtr, IGet, CSTRING, name)
{
    RexxObjectPtr v = context->GetContextVariable(name);
    return v == NULLOBJECT ? context->String("null") : v;
}
RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(Outer, Outer),
    REXX_TYPED_ROUTINE(OGet, OGet),
    REXX_TYPED_ROUTINE(OSet, OSet),
    REXX_TYPED_ROUTINE(ODrop, ODrop),
    REXX_TYPED_ROUTINE(OAll, OAll),
    REXX_TYPED_ROUTINE(IGet, IGet),
    REXX_LAST_ROUTINE()
};
RexxPackageEntry outer9b_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "outer9b", "1.0.0", NULL, NULL, routines, NULL
};
OOREXX_GET_PACKAGE(outer9b);
