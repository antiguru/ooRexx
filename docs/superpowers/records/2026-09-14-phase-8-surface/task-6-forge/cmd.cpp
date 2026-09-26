// The Task 6 forge: command handlers registered through AddCommandEnvironment
// that reach what orxfunction's handlers do not -- the exit context's own
// members, its Throw members, conditions raised from a handler, and a
// redirecting handler reading and writing each stream through every member.
#include "oorexxapi.h"
#include <string.h>
#include <stdio.h>

// What the destructors below ran, read back by Dtors.
static char dtorLog[256];

struct Noisy
{
    const char *name;
    ~Noisy() { strncat(dtorLog, name, sizeof(dtorLog) - strlen(dtorLog) - 2); strcat(dtorLog, ";"); }
};

RexxRoutine0(RexxStringObject, Dtors)
{
    RexxStringObject logged = context->String(dtorLog);
    dtorLog[0] = '\0';
    return logged;
}

static bool is(CSTRING command, CSTRING word)
{
    return strncmp(command, word, strlen(word)) == 0;
}

// The word after the first blank, or the empty string.
static CSTRING rest(CSTRING command)
{
    CSTRING blank = strchr(command, ' ');
    return blank == NULL ? "" : blank + 1;
}

RexxObjectPtr RexxEntry directHandler(RexxExitContext *context, RexxStringObject address, RexxStringObject command)
{
    CSTRING text = context->CString(command);
    if (is(text, "VARS"))
    {
        RexxObjectPtr x = context->GetContextVariable("X");
        context->SetContextVariable("Y", command);
        context->DropContextVariable("Z");
        RexxDirectoryObject all = context->GetAllContextVariables();
        RexxObjectPtr items = context->SendMessage0(all, "ITEMS");
        char line[200];
        snprintf(line, sizeof(line), "x=%s items=%s nox=%s", x == NULLOBJECT ? "NULL" : context->ObjectToStringValue(x),
            context->ObjectToStringValue(items), context->GetContextVariable("NOSUCHVAR") == NULLOBJECT ? "NULL" : "set");
        return context->String(line);
    }
    if (is(text, "CALLER"))
    {
        return context->GetCallerContext();
    }
    if (is(text, "REF"))
    {
        RexxVariableReferenceObject ref = context->GetContextVariableReference("X");
        if (ref == NULLOBJECT)
        {
            return context->String("noref");
        }
        context->SetVariableReferenceValue(ref, context->String("viaref"));
        return context->VariableReferenceName(ref);
    }
    if (is(text, "RAISENORES"))
    {
        context->RaiseCondition(rest(text), context->String("desc"), NULLOBJECT, NULLOBJECT);
        return context->String("ret");
    }
    if (is(text, "RAISE"))
    {
        context->RaiseCondition(rest(text), context->String("desc"), context->String("add"), context->String("res"));
        return context->String("ret");
    }
    if (is(text, "SYNTAX"))
    {
        context->RaiseException1(93900, context->String("boom"));
        return context->String("ret");
    }
    if (is(text, "THROW0"))
    {
        Noisy local = {"throw0"};
        context->ThrowException0(40001);
        return context->String("after throw0");
    }
    if (is(text, "THROW1"))
    {
        Noisy local = {"throw1"};
        context->ThrowException1(93900, context->String("thrown"));
        return context->String("after throw1");
    }
    if (is(text, "THROW2"))
    {
        Noisy local = {"throw2"};
        context->ThrowException2(40004, context->String("TWO"), context->String("2"));
        return context->String("after throw2");
    }
    if (is(text, "THROWA"))
    {
        Noisy local = {"throwa"};
        context->ThrowException(93900, context->ArrayOfOne(context->String("arr")));
        return context->String("after throwa");
    }
    if (is(text, "THROWC"))
    {
        Noisy local = {"throwc"};
        context->ThrowCondition(rest(text), context->String("tdesc"), context->String("tadd"), context->String("tres"));
        return context->String("after throwc");
    }
    if (is(text, "NULL"))
    {
        return NULLOBJECT;
    }
    if (is(text, "NUM"))
    {
        return context->WholeNumberToObject(5);
    }
    if (is(text, "ARRAY"))
    {
        return context->ArrayOfOne(context->String("item"));
    }
    return command;
}

// Writes the prefix and the line as one output line.
static void prefixed(RexxIORedirectorContext *io, bool error, CSTRING prefix, CSTRING data, size_t length)
{
    char line[400];
    size_t used = strlen(prefix);
    memcpy(line, prefix, used);
    size_t take = length < sizeof(line) - used ? length : sizeof(line) - used;
    memcpy(line + used, data, take);
    if (error)
    {
        io->WriteError(line, used + take);
    }
    else
    {
        io->WriteOutput(line, used + take);
    }
}

RexxObjectPtr RexxEntry redirectingHandler(RexxExitContext *context, RexxStringObject address, RexxStringObject command, RexxIORedirectorContext *io)
{
    CSTRING text = context->CString(command);
    char answer[200];
    if (is(text, "ECHO"))
    {
        CSTRING data;
        size_t length;
        size_t count = 0;
        io->ReadInput(&data, &length);
        while (data != NULL)
        {
            count++;
            prefixed(io, false, "o:", data, length);
            prefixed(io, true, "e:", data, length);
            io->ReadInput(&data, &length);
        }
        io->ReadInput(&data, &length);
        CSTRING again = data;
        io->ReadInputBuffer(&data, &length);
        snprintf(answer, sizeof(answer), "%zu %s %s %zu", count, again == NULL ? "end" : "more", data == NULL ? "null" : "buffer", length);
        return context->String(answer);
    }
    if (is(text, "BUFFIRST"))
    {
        CSTRING data;
        size_t length;
        io->ReadInputBuffer(&data, &length);
        CSTRING first = data;
        size_t firstLength = length;
        if (data != NULL)
        {
            io->WriteOutputBuffer(data, length);
        }
        io->ReadInput(&data, &length);
        CSTRING line = data;
        io->ReadInputBuffer(&data, &length);
        snprintf(answer, sizeof(answer), "%zu %s %s %zu %s", firstLength, first == NULL ? "null" : "buffer",
            line == NULL ? "end" : "more", length, data == first ? "same" : "moved");
        return context->String(answer);
    }
    if (is(text, "BUFFERS"))
    {
        io->WriteErrorBuffer("a\r", 2);
        io->WriteError("b", 1);
        io->WriteErrorBuffer("c\nd", 3);
        io->WriteOutputBuffer("\r\nx\r", 4);
        io->WriteOutputBuffer("\ny", 2);
        io->WriteOutputBuffer("", 0);
        io->WriteOutput("z\nw", 3);
        io->WriteOutputBuffer("tail\r", 5);
        return context->String("buffers");
    }
    if (is(text, "FLAGS"))
    {
        snprintf(answer, sizeof(answer), "%d%d%d%d%d", (int)io->IsRedirectionRequested(), (int)io->IsInputRedirected(),
            (int)io->IsOutputRedirected(), (int)io->IsErrorRedirected(), (int)io->AreOutputAndErrorSameTarget());
        return context->String(answer);
    }
    if (is(text, "WTHROW"))
    {
        Noisy local = {"wthrow"};
        io->WriteOutput("before", 6);
        context->ThrowException1(93900, context->String("wthrown"));
        return context->String("after wthrow");
    }
    if (is(text, "WRAISE"))
    {
        io->WriteOutput("before", 6);
        context->RaiseCondition(rest(text), context->String("wdesc"), NULLOBJECT, context->String("7"));
        return context->String("wret");
    }
    if (is(text, "WSYNTAX"))
    {
        io->WriteOutput("before", 6);
        context->RaiseException1(93900, context->String("wboom"));
        return context->String("wret");
    }
    return context->String("unknown");
}

// Registers `name` as a direct ("d"), redirecting ("r") or unknown ("x",
// type 3) handler.
RexxRoutine2(int, AddCmd, CSTRING, name, CSTRING, kind)
{
    if (kind[0] == 'd')
    {
        context->AddCommandEnvironment(name, (REXXPFN)directHandler, DIRECT_COMMAND_ENVIRONMENT);
    }
    else if (kind[0] == 'r')
    {
        context->AddCommandEnvironment(name, (REXXPFN)redirectingHandler, REDIRECTING_COMMAND_ENVIRONMENT);
    }
    else
    {
        context->AddCommandEnvironment(name, (REXXPFN)directHandler, 3);
    }
    return 0;
}

RexxRoutineEntry routines[] =
{
    REXX_TYPED_ROUTINE(AddCmd, AddCmd),
    REXX_TYPED_ROUTINE(Dtors, Dtors),
    REXX_LAST_ROUTINE()
};

RexxPackageEntry cmd_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "cmd",
    "1.0",
    NULL,
    NULL,
    routines,
    NULL
};

OOREXX_GET_PACKAGE(cmd);
