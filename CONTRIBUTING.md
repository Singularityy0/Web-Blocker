# Contributing to Web-Blocker

Thanks for taking the time to contribute. This is a short guide to get you moving quickly.

---

## Prerequisites

Install Rust via rustup:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The project targets the `x86_64-pc-windows-msvc` toolchain. Make sure it is installed:

```sh
rustup target add x86_64-pc-windows-msvc
```

---

## Getting Started

```sh
git clone https://github.com/Singularityy0/Web-Blocker.git
cd Web-Blocker
cargo build
cargo test
```

Run the app as Administrator (required to write to the hosts file):

```sh
cargo run
```

---

## Project Layout

```
src/
  main.rs        entry point
  app.rs         UI layer (iced Sandbox)
  blocker.rs     business logic
  hosts.rs       hosts file I/O
  duration.rs    duration string parsing
  permissions.rs admin privilege check
```

Read `DESIGN.md` before touching `hosts.rs` or `blocker.rs`. Those two modules
have non-obvious ordering constraints that are documented there.

---

## Making Changes

1. Fork the repository and create a branch named after what you are fixing or adding.
2. Make your changes. Run `cargo check` and `cargo test` before committing.
3. Keep commits focused. One logical change per commit.
4. Write a clear commit message in the imperative mood, for example:
   `Fix timed site entries not removed from hosts file`
5. Open a pull request against `main`. Describe what the change does and why.

---

## Code Conventions

Follow the patterns already in the codebase:

**Errors** are `String` at the boundary between business logic and the UI,
and `io::Result` inside `hosts.rs`. Do not use `unwrap` or `expect` in
non-test code.

**Hosts file writes** must always go through `hosts::write_hosts`. Never open
the hosts file directly from `blocker.rs` or `app.rs`.

**Operation order in `blocker.rs`** when removing a site: update the hosts
file first, then mutate the in-memory state, then save JSON. Reversing this
order leaves stale entries in the hosts file. See section 6.6 in `DESIGN.md`.

**Tests** live in `#[cfg(test)]` modules inside the relevant source file.
`duration.rs` has a full suite as a reference example.

---

## Reporting Bugs

Open a GitHub issue and include:

1. What you did.
2. What you expected to happen.
3. What actually happened.
4. The contents of your hosts file between the `# Website Blocker - Start`
   and `# Website Blocker - End` markers, if relevant.

---

## What Needs Work

These are good areas to contribute to:

- Auto-refresh the timed sites list so the countdown updates without restarting the app
- A confirmation dialog before removing a site
- Support for blocking by category (social media, gaming, etc.)
- Linux and macOS testing (the code supports both platforms but is only regularly tested on Windows)