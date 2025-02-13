#![windows_subsystem = "windows"]
use std::fs::{self, OpenOptions, read_to_string, write};
use std::io::{self,Read, Write};
use std::collections::HashSet;
use std::path::PathBuf;
use url::Url;
use serde::{Deserialize, Serialize};
use iced::widget::{button, text_input, scrollable, Column, Row, Text, Container};
use iced::{Element, Length, Sandbox, Settings};
use std::process::Command;
use std::collections::HashMap;
#[derive(Serialize, Deserialize, Default, Clone)]
struct BlockedSites {
    permanent_sites: HashSet<String>,
    timed_sites: HashMap<String, Option<std::time::SystemTime>>,
    is_blocking_enabled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    AddSite,
    RemoveSite(String),
    ToggleBlocking,
    ShowError(String),
    DurationChanged(String),

}

struct WebBlocker {
    blocked_sites: BlockedSites,
    input_value: String,
    status_message: String,
    hosts_file_path: PathBuf,
    duration_value: String,

}

impl Default for WebBlocker {
    fn default() -> Self {
        let hosts_file_path = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\\Windows\\System32\\drivers\\etc\\hosts")
        } else {
            PathBuf::from("/etc/hosts")
        };

        let mut blocker = Self {
            blocked_sites: BlockedSites {
                permanent_sites: HashSet::new(),
                timed_sites: HashMap::new(),
                is_blocking_enabled: false,
            },
            input_value: String::new(),
            status_message: String::new(),
            duration_value: String::new(),

            hosts_file_path,
        };


        if let Ok(content) = read_to_string("blocked_sites.json") {
            if let Ok(sites) = serde_json::from_str(&content) {
                blocker.blocked_sites = sites;
            }
        }

        blocker
    }
}

impl WebBlocker {
    fn validate_url(url: &str) -> Result<String, String> {
        let cleaned_url = url.trim()
            .strip_prefix("http://").unwrap_or(url)
            .strip_prefix("https://").unwrap_or(url)
            .strip_prefix("www.").unwrap_or(url);

        match Url::parse(&format!("http://{}", cleaned_url)) {
            Ok(parsed_url) => {
                let host = parsed_url.host_str()
                    .ok_or_else(|| "Invalid host".to_string())?;
                Ok(host.to_string())
            },
            Err(_) => Err("Invalid URL format".to_string())
        }
    }

    fn get_domain_variants(domain: &str) -> Vec<String> {
        let mut variants = HashSet::new();
        variants.insert(domain.to_string());
        variants.insert(format!("www.{}", domain));
        variants.insert(format!("m.{}", domain));
        variants.insert(format!("app.{}", domain));
        let mut result: Vec<String> = variants.into_iter().collect();
        result.sort();
        result

    }

    fn save_blocked_sites(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.blocked_sites)?;
        write("blocked_sites.json", json)
    }

    fn backup_hosts_file(&self) -> io::Result<()> {
        let backup_path = self.hosts_file_path.with_extension("bak");
        fs::copy(&self.hosts_file_path, backup_path)?;
        Ok(())
    }

    fn clean_hosts_file(&self) -> io::Result<()> {
        let mut hosts_content = String::new();
        OpenOptions::new()
            .read(true)
            .open(&self.hosts_file_path)?
            .read_to_string(&mut hosts_content)?;

        let blocked_domains: HashSet<String> = self.blocked_sites.permanent_sites
            .iter()
            .flat_map(|site| Self::get_domain_variants(site))
            .flat_map(|domain| {
                vec![
                    format!("127.0.0.1 {}", domain),
                    format!("::1 {}", domain),
                    domain.clone(),
                ]
            })
            .collect();

        let cleaned_lines: Vec<&str> = hosts_content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.contains("# Website Blocker") {
                    return false;
                }
                for blocked in &blocked_domains {
                    if trimmed.contains(blocked) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.hosts_file_path)?;

        for line in cleaned_lines {
            writeln!(file, "{}", line.trim())?;
        }

        Ok(())
    }

    fn update_hosts_file(&mut self) -> io::Result<()> {
        self.backup_hosts_file()?;
        self.clean_hosts_file()?;
        self.clean_expired_sites();

        if self.blocked_sites.is_blocking_enabled {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&self.hosts_file_path)?;

            writeln!(file, "\n# Website Blocker - Start")?;

            // Add permanent sites
            for site in &self.blocked_sites.permanent_sites {
                for variant in Self::get_domain_variants(site) {
                    writeln!(file, "127.0.0.1 {}", variant)?;
                    writeln!(file, "::1 {}", variant)?;
                }
            }

            // Add timed sites
            for (site, expiry) in &self.blocked_sites.timed_sites {
                if expiry.is_none() || expiry.unwrap() > std::time::SystemTime::now() {
                    for variant in Self::get_domain_variants(site) {
                        writeln!(file, "127.0.0.1 {}", variant)?;
                        writeln!(file, "::1 {}", variant)?;
                    }
                }
            }

            writeln!(file, "# Website Blocker - End")?;
        }

        Ok(())
    }


    fn save_and_update_hosts(&mut self) -> Result<(), String> {
        self.save_blocked_sites()
            .map_err(|e| format!("Failed to save blocked sites: {}", e))?;
        self.update_hosts_file()
            .map_err(|e| format!("Failed to update hosts file: {}", e))
    }


    fn check_permissions(&self) -> Result<(), String> {
        if cfg!(target_os = "windows") {
            let result = Command::new("whoami").arg("/priv").output();
            if let Ok(output) = result {
                if String::from_utf8_lossy(&output.stdout).contains("SeTakeOwnershipPrivilege") {
                    return Ok(());
                }
            }
            Err("Insufficient permissions. Please run as administrator.".to_string())

        } else {
            let result = Command::new("id").arg("-u").output();
            if let Ok(output) = result {
                if String::from_utf8_lossy(&output.stdout).trim() == "0" {
                    return Ok(());
                }
            }
            Err("Insufficient permissions. Please run as administrator.".to_string())

        }
    }

    fn toggle_blocking(&mut self) -> Result<(), String> {
        self.check_permissions()?;
        self.blocked_sites.is_blocking_enabled = !self.blocked_sites.is_blocking_enabled;
        if !self.blocked_sites.is_blocking_enabled {
            self.clean_hosts_file()
                .map_err(|e| format!("Failed to clean hosts file: {}", e))?;
        }
        self.save_and_update_hosts()
    }



    fn clean_expired_sites(&mut self) {
        let now = std::time::SystemTime::now();
        self.blocked_sites.timed_sites.retain(|_, &mut expiry| {
            match expiry {
                Some(expiry_time) => expiry_time > now,
                None => true, // Keep permanent blocks
            }
        });
    }

}

impl Sandbox for WebBlocker {
    type Message = Message;

    fn new() -> Self {
        let mut instance = Self::default();
        if let Err(e) = instance.check_permissions() {
            instance.status_message = e;
        }
        instance
    }

    fn title(&self) -> String {
        String::from("Website Blocker")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::AddSite => {
                if !self.input_value.trim().is_empty() {
                    match Self::validate_url(&self.input_value) {
                        Ok(domain) => {
                            // Split input into domain and duration
                            let domain_clone = domain.clone();
                            let parts: Vec<&str> = self.input_value.split_whitespace().collect();
                            let _duration = if parts.len() > 1 {
                                parts[1]
                            } else {
                                "permanent" // Default to permanent blocking
                            };

                            fn parse_duration(input: &str) -> Option<std::time::SystemTime> {
                                let input = input.trim().to_lowercase();
                                let now = std::time::SystemTime::now();
                                let duration = match input {
                                    s if s.ends_with("s") => {
                                        let seconds: u64 = s[..s.len()-1].parse().unwrap_or(0);
                                        std::time::Duration::from_secs(seconds)
                                    },
                                    s if s.ends_with("m") => {
                                        let minutes: u64 = s[..s.len()-1].parse().unwrap_or(0);
                                        std::time::Duration::from_secs(minutes * 60)
                                    },
                                    s if s.ends_with("h") => {
                                        let hours: u64 = s[..s.len()-1].parse().unwrap_or(0);
                                        std::time::Duration::from_secs(hours * 3600)
                                    },
                                    s if s.ends_with("d") => {
                                        let days: u64 = s[..s.len()-1].parse().unwrap_or(0);
                                        std::time::Duration::from_secs(days * 86400)
                                    },
                                    _ => return None
                                };
                                Some(now + duration)
                            }


                            // In the AddSite message handling
                            match parse_duration(&self.duration_value) {
                                Some(expiry) => {
                                    // Timed block
                                    self.blocked_sites.timed_sites.insert(domain, Some(expiry));
                                    self.status_message = format!("Site {} blocked until {:?}", domain_clone, expiry);
                                },
                                None => {
                                    // Permanent block if no valid duration
                                    self.blocked_sites.permanent_sites.insert(domain);
                                    self.status_message = format!("Site {} blocked permanently", domain_clone);
                                }
                            }


                            self.input_value.clear();
                            match self.save_and_update_hosts() {
                                Ok(_) => self.status_message = "Site added successfully".to_string(),
                                Err(e) => self.status_message = format!("Error: {}", e),
                            }
                        }
                        Err(e) => self.status_message = format!("Invalid URL: {}", e),
                    }
                }
            }


            Message::RemoveSite(site) => {
                if self.blocked_sites.permanent_sites.remove(&site) {
                    match self.save_and_update_hosts() {
                        Ok(_) => self.status_message = "Site removed successfully".to_string(),
                        Err(e) => self.status_message = format!("Error: {}", e),
                    }
                }
            }
            Message::ToggleBlocking => {
                match self.toggle_blocking() {
                    Ok(_) => {
                        self.status_message = if self.blocked_sites.is_blocking_enabled {
                            "Blocking enabled".to_string()
                        } else {
                            "Blocking disabled and hosts file cleaned".to_string()
                        };
                    }
                    Err(e) => self.status_message = format!("Error: {}", e),
                }
            }
            Message::ShowError(error) => {
                self.status_message = error;
            }

            Message::DurationChanged(value) => {
                self.duration_value = value;
            }



        }
    }

    fn view(&self) -> Element<Message> {
        let input = text_input("Enter website...", &self.input_value)
            .on_input(Message::InputChanged)
            .padding(10);

        let duration_input = text_input("Enter block duration (e.g.,1s, 1m, 1h, 1d)...", &self.duration_value)
            .on_input(Message::DurationChanged)
            .padding(10);

        let add_button = button("Add Website")
            .on_press(Message::AddSite)
            .padding(10);

        let input_row = Row::new()
            .spacing(10)
            .push(input)
            .push(duration_input)
            .push(add_button);


        let toggle_text = if self.blocked_sites.is_blocking_enabled {
            "Disable Blocking"
        } else {
            "Enable Blocking"
        };

        let toggle_button = button(toggle_text)
            .on_press(Message::ToggleBlocking)
            .padding(10);

        let mut content = Column::new()
            .spacing(20)
            .padding(20)
            .push(Text::new("Website Blocker").size(30))
            .push(input_row)
            .push(toggle_button);


        // Permanent sites list
        let sites_list = self.blocked_sites.permanent_sites.iter().fold(Column::new().spacing(10), |column, site| {
            column.push(
                Row::new()
                    .spacing(10)
                    .push(Text::new(format!("{} (Permanent)", site)))
                    .push(
                        button("Remove")
                            .on_press(Message::RemoveSite(site.clone()))
                            .padding(5)
                    )
            )
        });

        // Timed sites list
        let timed_sites_list = self.blocked_sites.timed_sites.iter().fold(sites_list, |column, (site, expiry)| {
            let expiry_text = match expiry {
                Some(time) => {
                    let duration = time.duration_since(std::time::SystemTime::now());
                    if let Ok(dur) = duration {
                        format!("Expires in {} seconds", dur.as_secs())
                    } else {
                        "Expired".to_string()
                    }
                }
                None => "Permanent".to_string(),
            };

            column.push(
                Row::new()
                    .spacing(10)
                    .push(Text::new(format!("{} ({})", site, expiry_text)))
                    .push(
                        button("Remove")
                            .on_press(Message::RemoveSite(site.clone()))
                            .padding(5)
                    )
            )
        });


        if !self.blocked_sites.permanent_sites.is_empty() || !self.blocked_sites.timed_sites.is_empty() {
            content = content.push(Text::new("Blocked Websites:").size(20));
            content = content.push(scrollable(timed_sites_list).height(Length::Fixed(200.0)));
        }


        if !self.status_message.is_empty() {
            content = content.push(
                Text::new(&self.status_message)
                    .size(16)
            );
        }

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }

}
fn process_hosts_file(file_path: &str) -> io::Result<()> {

    let mut file = OpenOptions::new().read(true).write(false).open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let dns_marker = "# localhost name resolution is handled within DNS itself.";
    let blocker_start_marker = "# Website Blocker - Start";

    if let Some(dns_index) = content.find(dns_marker) {
        let mut truncated_content = String::new();
        truncated_content.push_str(&content[..=dns_index + dns_marker.len()]);

        if let Some(blocker_start_index) = content.find(blocker_start_marker) {
            truncated_content.push_str(&content[blocker_start_index..]);
        }

        write(file_path, truncated_content)?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "DNS marker not found in hosts file"
        ));
    }

    Ok(())
}

fn main() -> iced::Result {
    if let Err(e) = process_hosts_file("C:\\Windows\\System32\\drivers\\etc\\hosts") {
        eprintln!("Warning: Hosts file processing failed: {}", e);
    }

    WebBlocker::run(Settings::default())
}

