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


## License

[Include your chosen license here]

## Contributing

[Include guidelines for contributing to the project]

<div style="text-align: center">⁂</div>

[^1]: https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/51247489/b016605b-684c-4397-ab14-0c99b1ba71fc/Cargo.toml

[^2]: https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/51247489/f00216f3-ce53-4bfd-9377-c70d0a09a5b7/main.rs

[^3]: https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/51247489/97477de9-8c12-48db-ae4e-bfcb3ac01338/config.toml

