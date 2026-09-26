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

// A buffer string filled through its data address and finished shorter.
RexxRoutine1(RexxStringObject, BufStr, CSTRING, text)
{
    size_t length = strlen(text);
    RexxBufferStringObject s = context->NewBufferString(length + 3);
    size_t made = context->BufferStringLength(s);
    char *data = (char *)context->BufferStringData(s);
    memcpy(data, text, length);
    RexxStringObject done = context->FinishBufferString(s, length);
    char buffer[256];
    snprintf(buffer, sizeof(buffer), "%zu %zu %d [%.*s]", made, context->BufferStringLength(s), done == (RexxStringObject)s,
        (int)context->StringLength(done), context->StringData(done));
    return context->String(buffer);
}

// A buffer's length and whether it reads as one.
RexxRoutine1(RexxStringObject, BufLen, size_t, n)
{
    RexxBufferObject b = context->NewBuffer(n);
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%zu %d %d", context->BufferLength(b), (int)context->IsBuffer(b), (int)context->IsBuffer(context->String("x")));
    return context->String(buffer);
}

// A pointer's value round-tripped through NewPointer.
RexxRoutine1(RexxStringObject, PtrValue, RexxObjectPtr, p)
{
    char buffer[64];
    POINTER v = context->IsPointer(p) ? context->PointerValue((RexxPointerObject)p) : NULL;
    RexxPointerObject again = context->NewPointer(v);
    snprintf(buffer, sizeof(buffer), "%d %d %d", (int)context->IsPointer(p), (int)(context->PointerValue(again) == v), (int)context->IsPointer(again));
    return context->String(buffer);
}

// The running method's CSELF, read as the int a Buffer holds.
RexxMethod0(int, CSelfRead)
{
    int *p = (int *)context->GetCSelf();
    return p == NULL ? -1 : *p;
}

// An object's CSELF from a given scope upwards.
RexxRoutine2(int, ScopedRead, RexxObjectPtr, o, RexxObjectPtr, scope)
{
    int *p = (int *)context->threadContext->functions->ObjectToCSelfScoped(context->threadContext, o, scope);
    return p == NULL ? -1 : *p;
}

// A reference to the calling activation's variable.
RexxRoutine1(RexxObjectPtr, CtxRef, CSTRING, name)
{
    RexxObjectPtr r = (RexxObjectPtr)context->GetContextVariableReference(name);
    return r == NULLOBJECT ? context->String("none") : r;
}

// The two environment directories.
RexxRoutine1(RexxObjectPtr, Env, logical_t, local)
{
    return local ? (RexxObjectPtr)context->GetLocalEnvironment() : (RexxObjectPtr)context->GetGlobalEnvironment();
}

// The caller's RexxContext.
RexxRoutine0(RexxObjectPtr, CallerCtx)
{
    return context->GetCallerContext();
}

// InvalidRoutine, then an answer the raise replaces.
RexxRoutine0(int, Invalid)
{
    context->InvalidRoutine();
    return 3;
}

// IsOfType, IsMethod and IsRoutine over one object.
RexxRoutine2(RexxStringObject, Types, RexxObjectPtr, o, CSTRING, name)
{
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%d %d %d", (int)context->IsOfType(o, name),
        (int)context->IsMethod(o), (int)context->IsRoutine(o));
    return context->String(buffer);
}

// FindClass through the thread table.
RexxRoutine1(RexxObjectPtr, FindCls, CSTRING, name)
{
    RexxObjectPtr c = (RexxObjectPtr)context->FindClass(name);
    return c == NULLOBJECT ? context->String("none") : c;
}

// ForwardMessage with every override but the receiver left out.
RexxMethod1(RexxObjectPtr, FwdTo, RexxObjectPtr, to)
{
    return context->ForwardMessage(to, NULL, NULL, NULL);
}

// LoadLibrary by name.
RexxRoutine1(logical_t, LoadLib, CSTRING, name)
{
    return context->LoadLibrary(name);
}

// A global reference kept across calls, then released, then used again.
static RexxObjectPtr kept = NULLOBJECT;

RexxRoutine1(int, Keep, RexxObjectPtr, o)
{
    kept = context->RequestGlobalReference(o);
    return kept == o;
}

RexxRoutine0(RexxObjectPtr, Kept)
{
    return kept == NULLOBJECT ? context->String("none") : kept;
}

RexxRoutine0(int, Release)
{
    context->ReleaseGlobalReference(kept);
    return 1;
}

// A local reference released, then read again through a global one.
RexxRoutine1(RexxObjectPtr, LocalRelease, RexxObjectPtr, o)
{
    RexxObjectPtr g = context->RequestGlobalReference(o);
    context->ReleaseLocalReference(o);
    return g;
}

// A package entry registered under a name, and its routine then called.
RexxRoutine1(RexxStringObject, Registered, CSTRING, text)
{
    return context->String(text);
}

static RexxRoutineEntry registered_routines[] = {
    REXX_TYPED_ROUTINE(RegisteredEcho, Registered),
    REXX_LAST_ROUTINE()
};

static RexxPackageEntry registered_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "registered",
    "1.0",
    NULL,
    NULL,
    registered_routines,
    NULL
};

RexxRoutine1(int, Register, CSTRING, name)
{
    return (int)context->RegisterLibrary(name, &registered_entry);
}

// The instance a thread context links, and a nested attach.
RexxRoutine1(RexxStringObject, Nested, RexxObjectPtr, o)
{
    RexxInstance *instance = context->GetInterpreterInstance();
    RexxThreadContext *attached = NULL;
    logical_t ok = instance->AttachThread(&attached);
    RexxObjectPtr r = attached->SendMessage0(o, "STRING");
    attached->DetachThread();
    char buffer[128];
    snprintf(buffer, sizeof(buffer), "%d %d %d [%s]", (int)ok, instance == context->threadContext->instance,
        attached != NULL, context->CString(r));
    return context->String(buffer);
}

RexxMethodEntry methods[] = {
    REXX_METHOD(FwdTo, FwdTo),
    REXX_METHOD(CSelfRead, CSelfRead),
    REXX_LAST_METHOD()
};

RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(BufStr, BufStr),
    REXX_TYPED_ROUTINE(Keep, Keep),
    REXX_TYPED_ROUTINE(Kept, Kept),
    REXX_TYPED_ROUTINE(Release, Release),
    REXX_TYPED_ROUTINE(LocalRelease, LocalRelease),
    REXX_TYPED_ROUTINE(Register, Register),
    REXX_TYPED_ROUTINE(Nested, Nested),
    REXX_TYPED_ROUTINE(LoadLib, LoadLib),
    REXX_TYPED_ROUTINE(Env, Env),
    REXX_TYPED_ROUTINE(CallerCtx, CallerCtx),
    REXX_TYPED_ROUTINE(Invalid, Invalid),
    REXX_TYPED_ROUTINE(Types, Types),
    REXX_TYPED_ROUTINE(FindCls, FindCls),
    REXX_TYPED_ROUTINE(CtxRef, CtxRef),
    REXX_TYPED_ROUTINE(BufLen, BufLen),
    REXX_TYPED_ROUTINE(PtrValue, PtrValue),
    REXX_TYPED_ROUTINE(ScopedRead, ScopedRead),
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
    methods
};

OOREXX_GET_PACKAGE(reach);
