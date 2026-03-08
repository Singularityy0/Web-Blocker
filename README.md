# Website Blocker

A Rust-based Windows desktop application that blocks websites by redirecting
their domains to `127.0.0.1` in the system hosts file.

---

## Features

- Block websites permanently or for a timed duration (e.g. `30m`, `2h`, `1d`)
- Remove sites from the block list at any time — hosts file cleaned immediately
- Toggle all blocking on / off with a single button
- Automatically blocks common domain variants (`www.`, `m.`, `app.`)
- Persists the block list across restarts (`blocked_sites.json`)
- Creates a `hosts.bak` backup before every modification
- Safe concurrent writes via `CreateFileW` with shared file access
- Precise hostname matching — blocking `app.com` never touches `my-app.com`

---

## Technical Details

| Item | Detail |
|---|---|
| Language | Rust 2021 |
| UI | [iced](https://github.com/iced-rs/iced) 0.10 |
| Blocking mechanism | Modifies `C:\Windows\System32\drivers\etc\hosts` |
| Persistence | `blocked_sites.json` (next to the executable) |
| Write strategy | `CreateFileW` with `FILE_SHARE_READ \| FILE_SHARE_WRITE` |

---

## Module Structure

```
src/
  main.rs        — entry point: startup sanitisation, launches the GUI
  app.rs         — Iced Sandbox: Message enum, view() layout, update() handler
  blocker.rs     — BlockedSites model + all business logic (add/remove/toggle/expire)
  hosts.rs       — all raw hosts file I/O (backup, clean, append, sanitise, write)
  duration.rs    — parse_duration(): human-readable suffix parsing with clear errors
  permissions.rs — check_permissions(): probes write access instead of parsing whoami
```

---

## Dependencies

```toml
iced         = "0.10"    # GUI framework
serde        = "1.0"     # serialisation derive macros
serde_json   = "1.0"     # JSON persistence
url          = "2.4"     # URL validation and host extraction
windows-sys  = "0.48"    # CreateFileW, FILE_SHARE_READ/WRITE (Windows only)
```

---

## Usage

1. Run `webblocker.exe` **as Administrator** (right-click -> Run as administrator).
2. Type a website URL in the left input field (e.g. `youtube.com` or `https://www.reddit.com`).
3. Optionally type a duration in the right field:
   - `30s` — 30 seconds
   - `5m`  — 5 minutes
   - `2h`  — 2 hours
   - `1d`  — 1 day
   - Leave blank for a **permanent** block.
4. Click **Add Website**.
5. Click **Enable Blocking** to activate. The hosts file is updated immediately.
6. Click **Disable Blocking** to deactivate — all blocker entries are removed from the hosts file at once.

> **Tip:** If a site is still reachable after blocking, restart your browser and/or run
> `ipconfig /flushdns` in an Administrator terminal.

---

## Removing a site

1. Click the **Remove** button next to the site in the list.
2. The hosts file is cleaned **immediately** — no need to disable blocking first
   and no restart required.

> **Tip:** If the site is still reachable after removal, flush DNS:
> `ipconfig /flushdns`

---

## Installation (from source)

```sh
# requires Rust — https://rustup.rs
cargo build --release
# binary is at target/release/webblocker.exe
```

The binary statically links the MSVC C runtime (configured in `.cargo/config.toml`)
so it runs on any Windows machine without extra redistributables.

---

## Notes

- Administrator privileges are required to write to the hosts file. The app
  detects this by probing write access directly — no fragile `whoami /priv` parsing.
- A backup of the hosts file (`hosts.bak`) is created before every modification.
- Blocking `example.com` automatically also blocks `www.example.com`,
  `m.example.com`, and `app.example.com`.
- Timed blocks are expired automatically — they are not written back to the
  hosts file once their time has passed.
- For full technical detail on every design and implementation decision,
  see [`DESIGN.md`](DESIGN.md).
