# Website Blocker

---

A Rust-based application with a user interface for Windows that allows users to block specific websites.

## Features

- Add websites to a block list
- Remove websites from the block list
- Toggle blocking on and off
- Set timed blocks for websites
- Persistent storage of blocked sites
- Automatic blocking of common domain variants (www, m, app)
- Hosts file backup before modifications


## Technical Details

- **Language:** Rust
- **UI Library:** Iced
- **Blocking Mechanism:** Modifies the Windows hosts file
- **File Path:** `C:\Windows\System32\drivers\etc\hosts`


## Dependencies

- iced = "0.10"
- serde = { version = "1.0", features = ["derive"] }
- serde_json = "1.0"
- url = "2.4"
- chrono = "0.4.39"


## Usage

1. Run the application with administrator privileges
2. Enter a website URL in the input field
3. Optionally, specify a block duration (e.g., 1s, 1m, 1h, 1d)
4. Click "Add Website" to block the site
5. Use the "Enable Blocking" / "Disable Blocking" button to toggle the blocker

## Installation

1. Ensure you have Rust installed on your system
2. Clone this repository
3. Run `cargo build --release` in the project directory
4. The executable will be available in `target/release/webblocker.exe`

## Notes

- The application requires administrator privileges to modify the hosts file
- A backup of the hosts file is created before any modifications
- Blocked websites are saved to a JSON file for persistence across app restarts
- it is advised to first select disble blocking then removing the sites.
- hence user is advised to resart the app once for unblocking to take affect 

## Installing Rust

To install Rust, follow these steps:

### 1. Install Rust using `rustup`
Rust uses `rustup` for installation and version management. Run the following command in your terminal:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

