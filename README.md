# Website Blocker

A Rust-based Windows desktop application that blocks websites by redirecting
their domains to `127.0.0.1` in the system hosts file.

---

## Features

- Block websites permanently or for a timed duration (e.g. `30m`, `2h`, `1d`)
- Remove sites from the block list at any time
- Toggle all blocking on / off with a single button
- Automatically blocks common domain variants (`www.`, `m.`, `app.`)
- Persists the block list across restarts (`blocked_sites.json`)
- Creates a `hosts.bak` backup before every modification
- Exclusive file-locking on every write — safe against concurrent access
- Precise hostname matching — blocking `app.com` never touches `my-app.com`

---

## Technical Details

| Item | Detail |
|---|---|
| Language | Rust 2021 |
| UI | [iced](https://github.com/iced-rs/iced) 0.10 |
| Blocking mechanism | Modifies `C:\Windows\System32\drivers\etc\hosts` |
| Persistence | `blocked_sites.json` (next to the executable) |
| File locking | `fs2` exclusive lock on every write |

---

## Module Structure

```
src/
  main.rs       
  app.rs         
  blocker.rs     
  hosts.rs      
  duration.rs   
  permissions.rs 
```

---

## Dependencies

```toml
iced       = "0.10"     
serde      = "1.0"        
serde_json = "1.0"        
url        = "2.4"       
fs2        = "0.4"        
```

> `chrono` has been removed , it was listed as a dependency but was never used.

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
6. Click **Disable Blocking** to stop all blocker entries are removed from the hosts file at once.

> **Tip:** If a site is still reachable after blocking, restart your browser and/or run
> `ipconfig /flushdns` in an Administrator terminal.

---

## Removing a site

1. Make sure **blocking is disabled** first (click "Disable Blocking").
2. Click the **Remove** button next to the site in the list.
3. Re-enable blocking if desired.

> The app must be **restarted once** after removal for the unblock to fully take effect.

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

- Administrator privileges are required to write to the hosts file.  The app
  detects this by probing write access directly — no fragile `whoami /priv` parsing.
- A backup of the hosts file (`hosts.bak`) is created before every modification.
- Blocking `example.com` automatically also blocks `www.example.com`,
  `m.example.com`, and `app.example.com`.
- Timed blocks are cleaned up automatically on the next app launch (expired entries
  are not written back to the hosts file).
