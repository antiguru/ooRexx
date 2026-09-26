// Routines that call the table members no shipped test extension reaches,
// built against the frozen api/ headers and loaded by name from a probe.
#include "oorexxapi.h"
#include <stdio.h>
#include <string.h>

static const char *text(RexxCallContext *c, RexxObjectPtr o)
{
    if (o == NULLOBJECT) return "<null>";
    return c->StringData((RexxStringObject)o);
}

// A condition raised and then read back before it is cleared.
RexxRoutine2(RexxObjectPtr, CondInfo, size_t, n, OPTIONAL_RexxObjectPtr, sub)
{
    if (argumentExists(2)) context->RaiseException1(n, sub);
    else context->RaiseException0(n);
    RexxDirectoryObject d = context->GetConditionInfo();
    context->ClearCondition();
    return d;
}

// CheckCondition before a raise, after it, and after ClearCondition.
RexxRoutine1(int, CondCheck, size_t, n)
{
    int before = context->CheckCondition() ? 1 : 0;
    context->RaiseException0(n);
    int after = context->CheckCondition() ? 1 : 0;
    context->ClearCondition();
    int cleared = context->CheckCondition() ? 1 : 0;
    return before * 100 + after * 10 + cleared;
}

// DecodeConditionInfo's fields, as text.
RexxRoutine2(RexxStringObject, CondDecode, size_t, n, OPTIONAL_RexxObjectPtr, sub)
{
    if (argumentExists(2)) context->RaiseException1(n, sub);
    else context->RaiseException0(n);
    RexxDirectoryObject d = context->GetConditionInfo();
    RexxCondition c;
    memset(&c, 0x5a, sizeof(c));
    context->DecodeConditionInfo(d, &c);
    context->ClearCondition();
    char buffer[2048];
    snprintf(buffer, sizeof(buffer), "code=%ld rc=%ld position=%zu name=[%s] message=[%s] errortext=[%s] description=[%s] additional=%s program=%s",
        (long)c.code, (long)c.rc, c.position,
        text(context, (RexxObjectPtr)c.conditionName), text(context, (RexxObjectPtr)c.message),
        text(context, (RexxObjectPtr)c.errortext), text(context, (RexxObjectPtr)c.description),
        c.additional == NULLOBJECT ? "null" : "array",
        c.program == NULLOBJECT ? "null" : "set");
    return context->String(buffer);
}

// DisplayCondition over a raised condition, then over none.
RexxRoutine1(wholenumber_t, CondDisplay, size_t, n)
{
    context->RaiseException0(n);
    wholenumber_t rc = context->DisplayCondition();
    context->ClearCondition();
    return rc * 1000 + context->DisplayCondition();
}

// A condition RaiseCondition raised, read back and optionally cleared.
RexxRoutine4(RexxObjectPtr, CondUser, CSTRING, name, logical_t, clear, OPTIONAL_RexxObjectPtr, additional, OPTIONAL_RexxObjectPtr, result)
{
    context->RaiseCondition(name, context->String("the description"), additional, result);
    RexxDirectoryObject d = context->GetConditionInfo();
    if (clear) context->ClearCondition();
    return d;
}

// GetConditionInfo with nothing raised.
RexxRoutine0(RexxObjectPtr, CondNone)
{
    RexxObjectPtr d = context->GetConditionInfo();
    return d == NULLOBJECT ? context->String("none") : d;
}

// A condition raised and left for the call to raise once it returns.
RexxRoutine2(int, RaiseKept, size_t, n, OPTIONAL_RexxObjectPtr, sub)
{
    if (argumentExists(2)) context->RaiseException1(n, sub);
    else context->RaiseException0(n);
    return 7;
}

RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(RaiseKept, RaiseKept),
    REXX_TYPED_ROUTINE(CondInfo, CondInfo),
    REXX_TYPED_ROUTINE(CondCheck, CondCheck),
    REXX_TYPED_ROUTINE(CondDecode, CondDecode),
    REXX_TYPED_ROUTINE(CondDisplay, CondDisplay),
    REXX_TYPED_ROUTINE(CondUser, CondUser),
    REXX_TYPED_ROUTINE(CondNone, CondNone),
    REXX_LAST_ROUTINE()
};

RexxPackageEntry reach_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "reach",
    "1.0",
    NULL,
    NULL,
    routines,
    NULL
};

OOREXX_GET_PACKAGE(reach);
