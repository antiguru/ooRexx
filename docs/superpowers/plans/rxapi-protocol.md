# rxapi wire protocol analysis (decision D7)

**Question:** can the new Rust interpreter speak the existing C++ `rxapi` daemon's
IPC protocol without linking any C++, or must the ~12k LOC of `rexxapi/` be ported?

**Answer: yes, a Rust client can speak it reliably. D7 stays "bridge to the C++ rxapi."**

The protocol is a raw dump of one C++ struct (`ServiceMessage`, 600 bytes on all
64-bit platforms) followed by an optional length-prefixed payload, exchanged over a
Unix domain socket (Unix) or a local named pipe (Windows). That sounds fragile, but
three properties make it a workable boundary in practice:

1. The struct contains only scalar types with natural alignment — its layout is
   identical under GCC, Clang, and MSVC on every 64-bit target (verified empirically,
   see §2).
2. Client and server can never be on different machines or of different endianness:
   the transport is strictly host-local, and the rendezvous name embeds the ooRexx
   version triple, the pointer width (`32`/`64`), and the username, so mismatched
   builds never even find each other's socket (§3).
3. There is a version handshake (value `100`) checked at connection time (§1).

All line numbers refer to the tree at branch `plan/rust-rewrite`.

---

## 1. Versioning

**There is a protocol version, and it is genuinely checked — but only by the client,
and only at session establishment.**

- The version constant is `REXXAPI_VERSION = 100`, defined in
  `rexxapi/common/ServiceMessage.hpp:199` (oddly, as a member of the
  `ServiceMessageParameters` enum alongside queue flags).
- There is **no version field in the message header itself**. The version travels as
  the reply to an explicit ping:
  - The client's first act on a new session is to send a
    `ClientMessage(APIManager, CONNECTION_ACTIVE)` message
    (`rexxapi/client/LocalAPIManager.cpp:220-225`).
  - The server handles `CONNECTION_ACTIVE` by writing `REXXAPI_VERSION` into
    `parameter1` of the echoed message (`rexxapi/server/APIServer.cpp:240-244`).
  - The client compares: `if (message.parameter1 != REXXAPI_VERSION)` and throws
    `ServiceException(API_FAILURE, "Open Object REXX version conflict. Incorrect
    version of RxAPI server detected")` — `rexxapi/client/LocalAPIManager.cpp:227-230`
    (first attempt) and `:258-261` (retry loop after auto-starting the daemon). The
    exception propagates as a hard API failure; `connectionEstablished` is never set.
- The **server performs no version check of any kind** on incoming messages. It
  trusts that anything connecting to its socket sends `sizeof(ServiceMessage)` bytes
  in its own layout (`rexxapi/server/APIServer.cpp:143-191` reads and dispatches with
  no validation beyond the `messageTarget`/`operation` switch; unknown `APIManager`
  operations get `INVALID_OPERATION`, `rexxapi/server/APIServer.cpp:247-249`).

Note the circularity: the version check itself is transmitted inside the raw struct,
so it only detects *protocol-constant* mismatches between builds that already agree
on layout. Layout mismatches are instead prevented by the rendezvous name (§3), which
embeds the release triple and pointer width. The two mechanisms together are adequate
for a same-machine, same-user IPC; neither would survive networking, which the design
never attempts.

**What a Rust client must do:** after connecting, send a `CONNECTION_ACTIVE` message
and verify `parameter1 == 100` in the reply, mirroring
`LocalAPIManager::establishServerConnection()`.

## 2. Layout portability

**Yes — messages are a fixed-layout C++ object written to the wire byte-for-byte.**

`ServiceMessage::writeMessage()` sends `(void *)this` for `sizeof(ServiceMessage)`
bytes plus the attached payload (`rexxapi/common/ServiceMessage.cpp:141-152`);
`readMessage()`/`readResult()` recv straight into `(char *)this`
(`ServiceMessage.cpp:68-112` server side, `:160-213` client side, both with correct
partial-read loops). The subclass `ClientMessage`
(`rexxapi/client/ClientMessage.hpp:47`) adds no data members, so both directions use
the identical 600-byte image.

The wire struct, `rexxapi/common/ServiceMessage.hpp:406-420`:

```cpp
ServerManager messageTarget;         // end receiver of the message
ServerOperation operation;           // operation to be performed
SessionID session;                   // the sender of the message (uintptr_t; client PID)
uintptr_t parameter1;                // first parameter passed
uintptr_t parameter2;                // second parameter passed
uintptr_t parameter3;                // the third parameter passed
uintptr_t parameter4;                // the fourth parameter passed
uintptr_t parameter5;                // the fifth parameter passed
ServiceReturn result;                // return result
ErrorCode errorCode;                 // error code from other side
void     *messageData;               // extra data attached to the message.
size_t    messageDataLength;         // size of the extra data.
bool      retainMessageData;         // indicates the server should not release result memory.
char      nameArg[NAMESIZE];         // buffer for name arguments (NAMESIZE = 256)
char      userid[MAX_USERID_LENGTH]; // name of the user (256, common/platform/*/SysProcess.hpp:52-53)
```

The class has no virtual functions and no base class, so the object image is exactly
these members. Layout verified empirically by compiling a probe against the real
header (g++ x86-64):

| field             | offset | size | wire meaning |
|-------------------|-------:|-----:|--------------|
| messageTarget     | 0      | 4    | enum `ServerManager` (ServiceMessage.hpp:73-80): QueueManager=0, RegistrationManager=1, MacroSpaceManager=2, APIManager=3 |
| operation         | 4      | 4    | enum `ServerOperation` (ServiceMessage.hpp:89-138): ADD_MACRO=0 … CLOSE_CONNECTION=39 (CONNECTION_ACTIVE=38) |
| session           | 8      | 8    | client PID (`LocalAPIManager.cpp:183`, `SysProcess::getPid()`) |
| parameter1..5     | 16..55 | 5×8  | operation-specific integers |
| result            | 56     | 4    | enum `ServiceReturn` (ServiceMessage.hpp:140-187): MESSAGE_OK=0, SERVER_ERROR=1, … |
| errorCode         | 60     | 4    | enum `ErrorCode` (ServiceException.hpp:44-68): NO_ERROR_CODE=0, … |
| messageData       | 64     | 8    | **garbage on the wire** — a host pointer; receiver overwrites it (see below) |
| messageDataLength | 72     | 8    | byte count of payload following the header |
| retainMessageData | 80     | 1    | **ignored on the wire** — receiver resets it (ServiceMessage.cpp:110, :177) |
| nameArg           | 81     | 256  | NUL-terminated string arg; carries the error text when result==SERVER_ERROR |
| userid            | 337    | 256  | NUL-terminated username; keys the per-user server instance (APIServer.cpp:322-341) |
| (tail padding)    | 593    | 7    | uninitialized in C++; send zeros |
| **total**         |        | **600** | |

Assessment against the three concerns:

- **Compiler padding.** On 64-bit targets there is *no interior padding*: the enum
  pairs (0-8 and 56-64) pack two 4-byte ints into each 8-byte slot, `bool` at 80 is
  followed only by `char` arrays, and only 7 tail-padding bytes exist. Every member
  is a scalar at natural alignment, so GCC, Clang, and MSVC (default `/Zp8`) all
  produce the same 600-byte image. The C++ side transmits uninitialized tail padding
  and a stale `messageData` pointer, but no receiver reads either: server-side
  `readMessage` immediately re-derives `messageData` by allocating a fresh buffer
  when `messageDataLength != 0` (ServiceMessage.cpp:84-111), and client-side
  `readResult` does the same (ServiceMessage.cpp:183-207).
- **Endianness.** Nothing is byte-swapped anywhere; `recv`/`send` move raw memory
  (`rexxapi/common/platform/unix/SysCSStream.cpp:78-127`, Windows
  `ReadFile`/`WriteFile` equivalents in
  `rexxapi/common/platform/windows/SysCSNamedPipeStream.cpp`). This is safe only
  because the transport is host-local (§3): both endpoints are always the same
  machine. A Rust client on the same host uses native byte order and matches by
  construction.
- **Pointer width.** The struct is riddled with width-dependent types: `uintptr_t`
  (session + 5 parameters), `void *`, `size_t`. A 32-bit build produces a 564-byte
  struct with entirely different offsets — flatly incompatible with a 64-bit build.
  The design's defense is that 32- and 64-bit builds use *different rendezvous
  names* (`"64"`/`"32"` under `#ifdef __REXX64__` —
  `common/platform/unix/SysCSStream.cpp:522-528` and
  `common/platform/windows/SysCSNamedPipeStream.cpp:374-380`), so mixed-width client
  and daemon never connect to each other; each width spawns its own daemon. There is
  no `long` member, so LP64 (Unix) vs LLP64 (Windows 64-bit) makes no difference.

**Second wire struct.** Registration operations attach a `ServiceRegistrationData`
(`rexxapi/common/ServiceMessage.hpp:203-277`) as the payload, again written raw
(`rexxapi/client/LocalRegistrationManager.cpp:73-74`, `:97-98`, `:166-167`, and read
back at `:252`, `:277`, `:316`):

```cpp
char moduleName[MAX_NAME_LENGTH];          // 256: name of the library
char procedureName[MAX_NAME_LENGTH];       // 256: the procedure within the library
size_t dropAuthority;                      // scope of drop authority (OWNER_ONLY=4 / DROP_ANY=5)
uintptr_t userData[2];                     // saved user data
uintptr_t entryPoint;                      // explicit entry point address
```

Verified layout: offsets 0/256/512/520/536, sizeof 544, no padding. `entryPoint`
and `userData` hold client-process pointers; they are opaque to the server and only
meaningful when round-tripped back to the registering process — a Rust client treats
them as opaque `u64`s. All other payloads (queue items, macro images) are plain byte
strings with length taken from `messageDataLength`
(`rexxapi/client/LocalQueueManager.cpp:430`, `:453`).

## 3. Transport

Strictly host-local on every platform; no TCP, no shared memory.

- **Unix (Linux, macOS, BSD, AIX): `AF_UNIX` / `SOCK_STREAM` socket.**
  Client connect: `socket(AF_UNIX, SOCK_STREAM, 0)` + `connect()` in
  `SysLocalSocketConnection::connect`,
  `rexxapi/common/platform/unix/SysCSStream.cpp:238-274`. Server bind/listen:
  `SysServerLocalSocketConnectionManager::bind`, same file `:338-384` (backlog 20);
  accept loop at `:282-302`. The socket path is
  `<dir>/.ooRexx-<ver>.<rel>.<mod>-<64|32>-<username>.service`, where `<dir>` is
  `$XDG_RUNTIME_DIR`, else `$TMPDIR`, else `/tmp` (name generation
  `SysCSStream.cpp:444-529`; version macros `ORX_VER/ORX_REL/ORX_MOD` come from the
  build, currently 5.3.0 — `CMakeLists.txt:84-86`). Example:
  `/run/user/1000/.ooRexx-5.3.0-64-moritz.service`.
- **Windows: local named pipe.** Client: `CreateFile` on the pipe name with retry
  via `WaitNamedPipe`
  (`rexxapi/common/platform/windows/SysCSNamedPipeStream.cpp:209-238`). Server:
  `CreateNamedPipe(userPipeName, PIPE_ACCESS_DUPLEX, PIPE_TYPE_BYTE | PIPE_WAIT |
  PIPE_REJECT_REMOTE_CLIENTS, ...)` — one new instance per inbound connection —
  followed by `ConnectNamedPipe` (`SysCSNamedPipeStream.cpp:270-293`).
  `PIPE_REJECT_REMOTE_CLIENTS` enforces host-locality. Pipe name:
  `\\.\pipe\ooRexx <ver>.<rel>.<mod>-<64|32>-<userid>`
  (`SysCSNamedPipeStream.cpp:359-395`); a mutex named the same (minus `\\.\pipe\`)
  guards single-instance (`:303-320`).
- **Note:** an unused TCP implementation exists
  (`SysServerSocketConnectionManager` using `sockaddr_in`,
  `common/platform/windows/SysCSStream.cpp` and part of the unix file), but neither
  client (`rexxapi/client/platform/unix/SysLocalAPIManager.cpp:189-201`,
  `.../windows/SysLocalAPIManager.cpp:224-236`) nor server instantiates it for rxapi
  traffic. Ignore it.

**Daemon lifecycle.** If the connect fails, the client auto-starts the daemon:
Unix double-`fork`/`setsid`/`execvp` of `../bin/rxapi` relative to the librexxapi
location, then `$PATH`, then `./rxapi`
(`rexxapi/client/platform/unix/SysLocalAPIManager.cpp:74-143`); Windows
`CreateProcess` of `RXAPI.EXE` (`.../windows/SysLocalAPIManager.cpp:83-177`). Then it
retries the ping up to 200 times at 10 ms intervals
(`rexxapi/client/LocalAPIManager.cpp:240-276`). Concurrent starters are resolved by
the bind/mutex: losers silently exit.

## Recommendation: keep D7 as "bridge to the C++ rxapi"

**A Rust process can speak this protocol reliably without linking any C++.** The
entire client side reduces to: one `#[repr(C)]` 600-byte struct, one 544-byte
attachment struct, four enum tables, a name-generation function, a connect-or-spawn
routine, and a request/response loop over a `UnixStream`/named pipe. The protocol's
apparent fragility (raw structs, host order, `uintptr_t` fields) is fully contained
by its design constraints: same host, same user, same pointer width, same release
triple — all enforced by the rendezvous name before a single byte is exchanged.

### What a Rust implementation must be careful about

1. **Exact layout replication.** `#[repr(C)]` with the field order above; add a
   compile-time assert that `size_of::<ServiceMessage>() == 600` and
   `size_of::<ServiceRegistrationData>() == 544`. Represent the four enums as `u32`
   with the exact discriminants from `ServiceMessage.hpp:73-200` and
   `ServiceException.hpp:44-68`. Send zeros for `messageData`, `retainMessageData`,
   and the 7 tail-padding bytes (the C++ side sends junk there; nothing reads it).
2. **Rendezvous name.** Must be generated with the *target rxapi's* version triple
   (currently `5.3.0`), the `64` width tag, and the username, following the exact
   `snprintf` formats at `SysCSStream.cpp:522` (note the 80-char directory-length
   cap and env-var fallback order at `:486-504`) and
   `SysCSNamedPipeStream.cpp:374`. Getting one byte of this wrong doesn't fail — it
   silently spawns a second, empty daemon, which would be a miserable bug to find.
3. **Handshake.** First message on a fresh session: `CONNECTION_ACTIVE` to
   `APIManager`, assert `parameter1 == 100` in the reply. Send `CLOSE_CONNECTION`
   (one-way, no reply — `LocalAPIManager.cpp:336-351`) before dropping a pooled
   connection; on process exit send `PROCESS_CLEANUP` as the C++ client does, so the
   daemon frees session queues.
4. **Framing.** Request = 600-byte header + `messageDataLength` payload bytes,
   ideally in one write (the C++ side coalesces, `SysCSStream.cpp:140-193`, but the
   reader loops on partial reads so it isn't required). Response = same shape.
   Errors arrive as `result == SERVER_ERROR (1)` with `errorCode != 0` and the
   message text in `nameArg` — mirror `raiseServerError()`
   (`ServiceMessage.hpp:319-326`).
5. **Strings.** `nameArg`/`userid`/`moduleName`/`procedureName` are NUL-terminated,
   truncated at 255 + NUL (`Utilities::strncpy` semantics). `session` is the client
   PID; `userid` selects the per-user data instance on the server, so it must match
   what `SysProcess::getUserID` would return.
6. **Version skew risk (accepted).** The layout is defined by compiler convention,
   not by a spec; a future ooRexx release could reshape the struct. The rendezvous
   name isolates release lines (a 5.4 daemon would live at a different socket), so
   the failure mode of skew is "Rust client targets one specific rxapi release
   series," which is exactly the bridge arrangement D7 assumes. Pinning the bridged
   rxapi version and re-validating the probe table per bump is sufficient.

The only scenario that would flip this decision — an unversioned protocol where
mismatched builds silently corrupt each other — does not exist here: mismatches are
prevented by name segregation and detected by the version ping. Porting 12k LOC to
avoid writing ~1-2k lines of well-specified Rust client code is not justified.

---

*Layout numbers verified by compiling a probe (`offsetof`/`sizeof` against the real
`ServiceMessage.hpp`) with g++ on x86-64 Linux; re-run the same probe on any new
target platform before trusting the table there.*
