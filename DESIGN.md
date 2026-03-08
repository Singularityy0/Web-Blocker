# Web-Blocker —>> Technical Design 

This document is the canonical reference for every architectural and implementation decision in the codebase.

---


## Project Goal

A Windows desktop application that lets a user block websites.  *simple enough!!>*
---
### Current approach
by redirecting
theirDNS lookups to `127.0.0.1` in the system hosts file. Features:

- Permanent blocks (stay until manually removed).
- Timed blocks (auto expire after a user specified duration).
- Oneclick global enable / disable toggle.
- Automatic coverage of `www.`, `m.`, and `app.` sub-domain variants.
- Safe, crash consistent writes to a protected system file.

## Module Map

```
src/
  main.rs        Entry point. Startup sanitisation, then launches the GUI.
  app.rs         Iced Sandbox: Message enum, view() layout, update() handler.
  blocker.rs     BlockedSites model + all business logic (add/remove/toggle/expire).
  hosts.rs       All raw hosts file I/O (backup, clean, append, sanitise, write).
  duration.rs    parse_duration(): human-readable suffix parsing with clear errors.
  permissions.rs check_permissions(): probes write access instead of parsing whoami.
```

Each module has a single, coped responsibility.  No module touches the hosts file except `hosts.rs`.  No module contains UI code except `app.rs`.

---

##  Model

### `BlockedSites` (defined in `blocker.rs`)

This is the serialisable struct that gets written to `blocked_sites.json`.

```
BlockedSites {
    permanent_sites:     HashSet<String>
    timed_sites:         HashMap<String, Option<SystemTime>>
    is_blocking_enabled: bool
}
```

**`permanent_sites`**
A `HashSet` of bare domain names (e.g. `"pclub.in"`).  `HashSet` is used
so duplicates are impossible by construction,inserting the same domain twice
is a no op.

**`timed_sites`**
A `HashMap` mapping a domain to an optional expiry timestamp.
- `Some(SystemTime)` — the block expires at that instant.
- `None` — treated as permanent within the timed list (legacy path, kept for backward compatibility with JSON files written by earlier versions).

**`is_blocking_enabled`**
A single global flag.  When `false`, the app ensures no blocker entries exist
in the hosts file regardless of what is in the two site lists.

### `BlockerState` (defined in `blocker.rs`)

The runtime wrapper that is NOT serialised.  It holds a `BlockedSites` and the
resolved `hosts_path` (`PathBuf`), and exposes all the mutating operations the
UI needs.

---

## How the Hosts File Works

The Windows hosts file lives at:

```
C:\Windows\System32\drivers\etc\hosts
```

The OS DNS resolver consults this file before hitting any external DNS server.
If a hostname appears in the file mapped to `127.0.0.1` (IPv4 loopback) and
`::1` (IPv6 loopback), all connections to that hostname are silently refused by the local machine.

This app writes entries in this form:

```
# Website Blocker - Start
127.0.0.1 pclub.in
::1 pclub.in
127.0.0.1 www.pclub.in
::1 www.pclub.in
127.0.0.1 m.pclub.in
::1 m.pclub.in
127.0.0.1 app.pclub.in
::1 app.pclub.com
# Website Blocker - End
```

Both IPv4 and IPv6 entries are written so that the block works regardless of
which IP stack the browser uses.

After adding or removing entries the user may need to flush the DNS cache:

```
ipconfig /flushdns
```

---

## Module: `hosts.rs`

This is the most technically complex module(at least for me) because of the Windows file access
constraints described below.

### The Windows Write Problem i.e  os error 5

The hosts file is always held open for reading by at least two Windows system services: the DNS Client service (`dnscache`) and Windows Defender.  This
creates a problem for any process trying to write to it. (atleast thats what I could deduce)

Two obvious approaches both fail:

**Approach A —> `fs2::lock_exclusive()`**
The `fs2` crate calls `LockFileEx` from the Windows API.  `LockFileEx` requires
that the file handle was opened with `FILE_SHARE_READ | FILE_SHARE_WRITE` in
its sharing flags.  Rust's `std::fs::OpenOptions` opens files with *exclusive*
sharing by default (no share flags).  When DNS Client or Defender already has the file open, opening it with exclusive sharing fails immediately 
`os error 5: Access Denied`, before the lock is even attempted.

**Approach B —> write to `hosts.tmp`, then `fs::rename()` into place**
This is the standard "atomic write" trick used widely in Unix programming. [to read](https://rcrowley.org/2010/01/06/things-unix-can-do-atomically.html) . On Windows, however, `MoveFileEx` (which backs `fs::rename`) is forbidden from moving a file *into* a protected system directory
(`C:\Windows\System32\drivers\etc\`) even as Administrator.  This also returns
`os error 5: Access Denied`.

### Solution —> `CreateFileW` with Shared Access

My third approach was to bypass `std::fs::OpenOptions` entirely and call
`CreateFileW` from the Win32 API directly, passing explicit share flags:

```rust
CreateFileW(
    path_ptr,
    GENERIC_WRITE,
    FILE_SHARE_READ | FILE_SHARE_WRITE,   // < the key flags
    null(),
    OPEN_EXISTING,
    FILE_ATTRIBUTE_NORMAL,
    0,
)
```

`FILE_SHARE_READ | FILE_SHARE_WRITE` tells the kernel: "grant me write access,
but allow other processes that already have read handles (DNS Client, Defender) to keep those handles open."  The kernel honours this because there is no conflict between one writer and multiple concurrent readers, as long as the readers do not *also* need exclusive access (they don't ,they just read).

The returned `HANDLE` is then wrapped in a `std::fs::File` via `File::from_raw_handle` so that the standard `Write` trait, `set_len`, `seek`,
and automatic `CloseHandle` on drop all work normally.

The write sequence is:
1. `set_len(0)` = truncate the file to zero bytes.
2. `seek(SeekFrom::Start(0))` = move the cursor back to the beginning
   (`set_len` does not move the cursor on Windows).
3. `write_all(new_content)` = write the full new content.
4. `flush()` = ensure all bytes are pushed to the OS buffer.

This function is compiled only on Windows (`#[cfg(target_os = "windows")]`).
On other platforms `fs::write()` is used instead.

### Section Markers

Two string constants delimit the block of entries this app owns

```
MARKER_START = "# Website Blocker - Start"
MARKER_END   = "# Website Blocker - End"
```

Every function that reads or writes the file uses these markers to locate and
isolate the app's section.  Nothing outside these markers is ever modified by
the app. (why? because I was having trouble deleting the blocked websites so having a starting point and ending point worked well)

### `clean()`

Removes all lines that belong to this app from the hosts file, leaving every
other line untouched.

A line is considered "owned by this app" if:
- It is blank (I wrote blank separator lines).
- It equals `MARKER_START` or `MARKER_END`.
- Its **second whitespace token** (the hostname) exactly matches an entry in the `blocked_domains` set passed by the caller.

The third rule uses **exact token matching**, not substring matching.  This is a deliberate fix over the original code which used `line.contains(domain)`.
The old approach would accidentally delete a user entry like`127.0.0.1 my-app.com` when the user had blocked `app.com`, because`"my-app.com".contains("app.com")` is `true`.  The new approach splits the line on whitespace and checks only the hostname token:

```
"127.0.0.1 my-app.com"  ->  tokens = ["127.0.0.1", "my-app.com"]
                              hostname = "my-app.com"
                              blocked_domains.contains("my-app.com") -> false  -> KEPT
```

The `blocked_domains` set passed to `clean()` is built by
`all_tracked_domains()` in `blocker.rs`, which includes variants for **both**
permanent and timed sites, so the clean is always complete.

### `append_blocks()`

Reads the current file content, appends the `MARKER_START` section with all
active entries, then writes the whole thing back via `write_hosts`.
Each entry is a `(String, String)` tuple of `("127.0.0.1 domain", "::1 domain")` already formatted by the caller (`blocker::update_hosts`).

### `sanitise_on_startup()`

Older versions of this app (and manual edits) could leave stale content between the standard Windows DNS comment line and the blocker section.  The old
`process_hosts_file()` function "fixed" this by truncating *everything* between those two points i.e  silently deleting any custom entries the user had manuallyadded in that region.

`sanitise_on_startup()` is safer:

1. If `MARKER_START` is not present, there is nothing to do , return immediately.
2. If both markers are present, extract our block verbatim.
3. Find the Windows DNS comment line and keep everything up to and including it.
4. Discard only the gap between the DNS comment and our block (that region is written by Windows, not by the user).
5. Reassemble preserved header + one blank line + our block.
6. Write back via `write_hosts`.

User entries written *above* the DNS comment are never touched.
User entries written *after* our block would need to be above the DNS marker to
be preserved i.e consistent with standard hosts file conventions.

### `backup()`

Before every call to `write_hosts`, `blocker::update_hosts()` calls
`hosts::backup()`, which copies the current hosts file to `hosts.bak` in the
same directory.  This gives the user a one-step recovery path if anything goes
wrong.

---

## Module: `blocker.rs`

### `BlockedSites` —>>  the Persisted Model

See [Section 3](#3-data-model).  Derives `Serialize` + `Deserialize` from
`serde` so the entire struct round-trips cleanly to/from JSON via `serde_json`.
Derives `Default` so a fresh install starts with an empty list and blocking
disabled.

### `BlockerState`  the Runtime Wrapper

`BlockerState` is not serialised.  It is created at startup from the loaded
`BlockedSites` JSON plus the resolved `hosts_path`:

```rust
pub fn load(hosts_path: PathBuf) -> Self {
    let sites = fs::read_to_string("blocked_sites.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    Self { sites, hosts_path }
}
```

All three steps are chained with `Option` combinators.  If the file does not
exist, cannot be read, or contains malformed JSON, `unwrap_or_default()` falls
back to a clean empty state silently, appropriate because this is a first-run
scenario, not a logic error.

###  Domain Variants

When a user blocks `youtube.com` they almost certainly also want to block
`www.youtube.com`, `m.youtube.com`, and `app.youtube.com`.  `domain_variants()`
produces all four automatically.

A `HashSet` is used internally to deduplicate before sorting, which matters if
the user enters a domain that is already a `www.` prefix (e.g. `www.youtube.com`
→ after stripping `www.`, the root becomes `youtube.com`, variants are the same
four regardless).

### `all_tracked_domains()`

Used exclusively as the argument to `hosts::clean()`.  It chains the keys of
both `permanent_sites` and `timed_sites` (even expired ones are included so
their hosts file entries are cleaned up even if `expire_timed_sites()` has not
run yet), expands each to its four variants, and collects into a `HashSet<String>`.

### `update_hosts()`  Write Cycle

This is the central orchestration function called after any state change:

```
1. backup()              — save hosts.bak
2. clean()               — remove all our entries (using all_tracked_domains)
3. expire_timed_sites()  — drop entries whose SystemTime has passed
4. if is_blocking_enabled:
       build entries list for all active domains
       append_blocks()   — write the fresh section
```

Step 2 always runs before step 4.  This means an update is always a
clean-then-rewrite, never an in-place edit.  The file is never in a state where
duplicate entries could accumulate.

The entries list in step 4 filters timed sites:

```rust
.filter(|(_, exp)| exp.map_or(true, |t| t > now))
```

`None` maps to `true` (keep it, permanent within timed list).
`Some(t)` keeps it only if `t > now` (not yet expired).

### `toggle_blocking()`

```
1. check_permissions()   — abort with error if not admin
2. flip is_blocking_enabled
3. if now disabled:
       hosts::clean()    — immediate removal so browsing resumes at once
4. save_and_update()     — persist JSON + rewrite hosts
```

The immediate `clean()` in step 3 is important for user experience: without it,
the user would have to restart the app or wait for the next update cycle for
the hosts file to be cleaned.

### `expire_timed_sites()`

Uses `HashMap::retain` to drop any timed site whose expiry has passed:

```rust
self.sites.timed_sites.retain(|_, exp| match *exp {
    Some(t) => t > now,
    None    => true,
});
```

This is called inside `update_hosts()` so expired sites are cleaned from
memory and from the hosts file in the same operation.

---

## `duration.rs`

### `ParsedDuration` Enum

```rust
pub enum ParsedDuration {
    Permanent,
    Timed(SystemTime),
}
```

Returning a typed enum instead of `Option<SystemTime>` makes the caller's
intent explicit.  `Permanent` is a first-class concept, not "None".

### Parsing Rules and Error Strategy

The original code used `.unwrap_or(0)` when parsing the numeric part of a
duration string.  This meant typing `"10x"` would silently produce a 0 second
block that expired instantly, with no feedback to the user.

`parse_duration()` now returns `Err(String)` for every bad input, and the error
message is surfaced directly in the UI's status bar.  The rules are:

| Input | Result |
|---|---|
| `""` or `"permanent"` | `Ok(Permanent)` |
| `"30s"` | `Ok(Timed(now + 30s))` |
| `"5m"` | `Ok(Timed(now + 5 minutes))` |
| `"2h"` | `Ok(Timed(now + 2 hours))` |
| `"1d"` | `Ok(Timed(now + 1 day))` |
| `"0s"` / `"0m"` etc. | `Err("duration must be > 0")` |
| `"10x"` | `Err("'x' is not a recognised suffix")` |
| `"abcm"` | `Err("'abc' is not a valid number")` |
| `"m"` (single char) | `Err("not a valid duration")` |

Overflow is also handled: `checked_mul` and `checked_add` are used on the
numeric part and the final `SystemTime`, returning errors instead of panicking
or wrapping silently.

---

## `permissions.rs`

The original code ran `whoami /priv` and checked the output for
`SeTakeOwnershipPrivilege`.  This was fragile for two reasons:

1. The privilege name varies slightly across Windows editions and configurations.
2. The *presence* of a privilege in the list does not mean it is *enabled* 
   privileges can be present but disabled.

The replacement is a direct write-access probe:

```rust
OpenOptions::new().append(true).open(hosts_path)
```

Opening the hosts file in append mode (which does not modify the file) is
exactly the permission the rest of the app needs.  If it succeeds, the app can
proceed.  If it fails, the OS error message is forwarded to the UI directly —
it will say exactly why the access was denied, which is more informative than
any custom message.

---

## `app.rs`

### Iced Sandbox Pattern

The app uses `iced::Sandbox` (not `Application`).  `Sandbox` is the simpler of
the two iced entry points: it is single-threaded, has no async runtime, and has
no `Command` type.  This is appropriate because every operation in this app is
synchronous (file I/O is fast on local disk) and there is no background work to
perform.

The Elm architecture enforced by iced:
- `new()`    — create initial state.
- `title()`  — window title string.
- `update()` — receive a `Message`, mutate state.
- `view()`   — render current state to a widget tree.

###  Message Enum

```rust
pub enum Message {
    UrlChanged(String),      
    DurationChanged(String),  
    AddSite,                  
    RemoveSite(String),       
    ToggleBlocking,           
}
```

The `ShowError(String)` variant from the original code has been removed.it was defined but never dispatched. Errors are now set directly on
`status_message` inside each handler rather than being routed through a
separate message variant.

### `update()`
**`UrlChanged` / `DurationChanged`**
Pure field assignments, no I/O.  Iced calls `update()` on every keystroke, so
these must be as cheap as possible.

**`AddSite`**
Three-phase validation before any I/O:
1. `BlockerState::validate_url()` — extract bare hostname or return error.
2. `parse_duration()` — parse the duration field or return error.
3. Insert into `permanent_sites` or `timed_sites` based on the result.

Both inputs are cleared *before* calling `save_and_update()` so the UI
immediately reflects the cleared state even if the file write is slow.

**`RemoveSite(site)`**
Calls `blocker.remove(&site)` which checks **both** `permanent_sites` and
`timed_sites`.  This fixes the original bug where clicking Remove on a timed
site did nothing because the handler only searched `permanent_sites`.

**`ToggleBlocking`**
Delegates entirely to `blocker.toggle_blocking()`, then sets `status_message`
based on the new `is_blocking_enabled` value.



Permanent and timed sites are rendered in the same scrollable column.  Timed
sites show remaining seconds (computed live from `SystemTime::now()` at render
time).  An "Expired" label is shown if the expiry has passed but
`expire_timed_sites()` has not yet run (e.g. between the last `update_hosts`
call and the current render frame).

---

## `main.rs`

Three responsibilities, in order:

**1. Console suppression**
```rust
#![cfg_attr(not(test), windows_subsystem = "windows")]
```
This attribute tells the Windows linker to produce a GUI subsystem binary,
which means no console window appears when the app is launched.  The
`not(test)` guard is critical: without it, `cargo test` would also suppress
the console, making all test output invisible and the test runner appear to
hang.

**2. Startup sanitisation**
```rust
let hosts_path = hosts::default_hosts_path();
if let Err(e) = hosts::sanitise_on_startup(&hosts_path) {
    eprintln!("Warning: startup hosts-file sanitisation failed: {}", e);
}
```
Errors from sanitisation are non-fatal.  If the hosts file is in a state that
cannot be parsed (e.g. missing the DNS marker), the app continues normally.
The user is not blocked from using the app just because the hosts file is
non-standard.

**3. Launch**
```rust
app::run()
```
Delegates to `app::run()` which calls `WebBlocker::run(Settings::default())`.
Keeping `main.rs` at three responsibilities means it never needs to change
unless the startup sequence itself changes.

---

## Persistence —>> `blocked_sites.json`

`BlockedSites` is serialised to `blocked_sites.json` in the **current working
directory** (wherever the `.exe` is launched from , typically the same folder
as the binary).

Format (pretty-printed via `serde_json::to_string_pretty`):

```json
{
  "permanent_sites": [
    "youtube.com",
    "reddit.com"
  ],
  "timed_sites": {
    "twitter.com": {
      "secs_since_epoch": 1720000000,
      "nanos_adjustment": 0
    }
  },
  "is_blocking_enabled": true
}
```

`SystemTime` is serialised by serde as a struct with `secs_since_epoch` and
`nanos_adjustment`.  This is the default serde representation and does not
require any custom serialiser.

The file is written on every Add, Remove, and Toggle action.  It is read once
at startup inside `BlockerState::load()`.

---


##  Dependency Rationale

| Crate | Version | Why |
|---|---|---|
| `iced` | 0.10 | GUI framework (Elm architecture, GPU-accelerated rendering via wgpu) |
| `serde` | 1.0 | Derive macros for `Serialize` / `Deserialize` on `BlockedSites` |
| `serde_json` | 1.0 | JSON serialisation for `blocked_sites.json` persistence |
| `url` | 2.4 | Robust URL parsing and hostname extraction in `validate_url()` |
| `windows-sys` | 0.48 | Direct access to `CreateFileW`, `FILE_SHARE_READ`, `FILE_SHARE_WRITE` |

`windows-sys` is declared under `[target.'cfg(windows)'.dependencies]` so it
is only compiled and linked on Windows builds.  The features requested are the minimum required:

```toml
features = [
    "Win32_Foundation",        # GENERIC_WRITE, INVALID_HANDLE_VALUE, HANDLE
    "Win32_Security",          # SECURITY_ATTRIBUTES (required by CreateFileW signature)
    "Win32_Storage_FileSystem", # CreateFileW, FILE_SHARE_READ/WRITE, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL
]
```
