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
5. If you don't have Rust installed or prefer not to install it, a release build is included in the repository.

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

```

## HOW TO USE

- run the .exe file as adminsitrator
- it should look something like this.

![image](https://github.com/user-attachments/assets/ba9ab2e3-8f73-4c27-bff3-d30e4c68373e)

- follow the steps in order <span style="color:red">RED</span> <span style="color:yellow">YELLOW</span> <span style="color:green">GREEN</span>



  ![Screenshot (4)](https://github.com/user-attachments/assets/6e44cd4a-8b16-47e7-918d-cf89136119da)

- should look something like this
  ![Screenshot (5)](https://github.com/user-attachments/assets/4006f749-da3f-44fa-9672-e8ac197612dd)

- CHECK!! , if does not work , close and restart your browser and/or perform the following code on terminal (admin)
      
  ```sh
  ipconfig /flushdns
  ```
- __TO DELETE__

- follow the steps in order <span style="color:red">RED</span> <span style="color:yellow">YELLOW</span> <span style="color:green">GREEN</span>
![image](https://github.com/user-attachments/assets/7bea229a-3bc4-4501-8a94-c8caec1199c2)

- **IT IS ABSOLUTELY NECESSARY TO CLOSE AND RESTART THE SCRIPT IN ORDER FOR UNBLOCKING TO TAKE AFFECT !!!**
- - CHECK!! , if does not work , close and restart your browser and/or perform the following code on terminal (admin)
      
  ```sh
  ipconfig /flushdns
  ```

