use iced::{Element, Length, Sandbox, Settings};
use iced::widget::{button, scrollable, text_input, Column, Container, Row, Text};

use crate::blocker::BlockerState;
use crate::duration::{parse_duration, ParsedDuration};
use crate::hosts;

#[derive(Debug, Clone)]
pub enum Message {
    UrlChanged(String),
    DurationChanged(String),
    AddSite,
    RemoveSite(String),
    ToggleBlocking,
}

pub struct WebBlocker {
    blocker:        BlockerState,
    url_input:      String,
    duration_input: String,
    status_message: String,
}

impl Sandbox for WebBlocker {
    type Message = Message;

    fn new() -> Self {
        let hosts_path = hosts::default_hosts_path();
        let blocker    = BlockerState::load(hosts_path);
        let status     = blocker.check_permissions().err().unwrap_or_default();
        Self {
            blocker,
            url_input:      String::new(),
            duration_input: String::new(),
            status_message: status,
        }
    }

    fn title(&self) -> String {
        "Website Blocker".to_string()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::UrlChanged(v)      => self.url_input      = v,
            Message::DurationChanged(v) => self.duration_input = v,

            Message::AddSite => {
                let raw = self.url_input.trim().to_string();
                if raw.is_empty() {
                    self.status_message = "Please enter a website address.".to_string();
                    return;
                }

                let domain = match BlockerState::validate_url(&raw) {
                    Ok(d)  => d,
                    Err(e) => { self.status_message = format!("Invalid URL: {}", e); return; }
                };

                let parsed = match parse_duration(&self.duration_input) {
                    Ok(p)  => p,
                    Err(e) => { self.status_message = e; return; }
                };

                match parsed {
                    ParsedDuration::Permanent      => self.blocker.add_permanent(domain.clone()),
                    ParsedDuration::Timed(expiry)  => self.blocker.add_timed(domain.clone(), expiry),
                }

                self.url_input.clear();
                self.duration_input.clear();

                self.status_message = match self.blocker.save_and_update() {
                    Ok(_)  => format!("'{}' added successfully.", domain),
                    Err(e) => format!("Error: {}", e),
                };
            }

            Message::RemoveSite(site) => {
                // remove_and_update cleans the hosts file BEFORE removing the
                // domain from the in-memory set, so clean() still knows which
                // lines to strip.
                self.status_message = match self.blocker.remove_and_update(&site) {
                    Ok(true)  => format!("'{}' removed.", site),
                    Ok(false) => format!("'{}' was not in the block list.", site),
                    Err(e)    => format!("Error: {}", e),
                };
            }

            Message::ToggleBlocking => {
                self.status_message = match self.blocker.toggle_blocking() {
                    Ok(_) => if self.blocker.sites.is_blocking_enabled {
                        "Blocking enabled.".to_string()
                    } else {
                        "Blocking disabled. Hosts file cleaned.".to_string()
                    },
                    Err(e) => format!("Error: {}", e),
                };
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let url_field = text_input("Website (e.g. youtube.com)", &self.url_input)
            .on_input(Message::UrlChanged)
            .padding(10);

        let duration_field =
            text_input("Duration (e.g. 1h, 30m) — blank = permanent", &self.duration_input)
                .on_input(Message::DurationChanged)
                .padding(10);

        let add_btn = button("Add Website")
            .on_press(Message::AddSite)
            .padding(10);

        let input_row = Row::new()
            .spacing(10)
            .push(url_field)
            .push(duration_field)
            .push(add_btn);

        let toggle_label = if self.blocker.sites.is_blocking_enabled {
            "Disable Blocking"
        } else {
            "Enable Blocking"
        };
        let toggle_btn = button(toggle_label)
            .on_press(Message::ToggleBlocking)
            .padding(10);

        let list = self
            .blocker.sites.permanent_sites.iter()
            .fold(Column::new().spacing(8), |col, site| {
                col.push(site_row(site, "(Permanent)", Message::RemoveSite(site.clone())))
            });

        let list = self
            .blocker.sites.timed_sites.iter()
            .fold(list, |col, (site, expiry)| {
                let label = match expiry {
                    Some(t) => match t.duration_since(SystemTime::now()) {
                        Ok(d)  => format!("Expires in {}s", d.as_secs()),
                        Err(_) => "Expired".to_string(),
                    },
                    None => "Permanent".to_string(),
                };
                col.push(site_row(site, &label, Message::RemoveSite(site.clone())))
            });

        let mut content = Column::new()
            .spacing(20)
            .padding(20)
            .push(Text::new("Website Blocker").size(30))
            .push(input_row)
            .push(toggle_btn);

        let has_sites = !self.blocker.sites.permanent_sites.is_empty()
            || !self.blocker.sites.timed_sites.is_empty();

        if has_sites {
            content = content
                .push(Text::new("Blocked Websites:").size(20))
                .push(scrollable(list).height(Length::Fixed(200.0)));
        }

        if !self.status_message.is_empty() {
            content = content.push(Text::new(&self.status_message).size(16));
        }

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}

use std::time::SystemTime;

fn site_row<'a>(domain: &str, label: &str, on_remove: Message) -> Row<'a, Message> {
    Row::new()
        .spacing(10)
        .push(Text::new(format!("{} ({})", domain, label)))
        .push(button("Remove").on_press(on_remove).padding(5))
}

pub fn run() -> iced::Result {
    WebBlocker::run(Settings::default())
}
