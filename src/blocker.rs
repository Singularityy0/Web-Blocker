use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::hosts;
use crate::permissions;
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct BlockedSites {
    pub permanent_sites: HashSet<String>,
    pub timed_sites: HashMap<String, Option<SystemTime>>,
    pub is_blocking_enabled: bool,
}
pub struct BlockerState {
    pub sites: BlockedSites,
    pub hosts_path: PathBuf,
}

impl BlockerState {
    pub fn load(hosts_path: PathBuf) -> Self {
        let sites = std::fs::read_to_string("blocked_sites.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Self { sites, hosts_path }
    }
    pub fn validate_url(input: &str) -> Result<String, String> {
        let cleaned = input
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("www.");

        match url::Url::parse(&format!("http://{}", cleaned)) {
            Ok(parsed) => parsed
                .host_str()
                .map(str::to_string)
                .ok_or_else(|| "Could not extract host from URL.".to_string()),
            Err(_) => Err("Invalid URL format.".to_string()),
        }
    }
    pub fn domain_variants(domain: &str) -> Vec<String> {
        let mut set = HashSet::new();
        set.insert(domain.to_string());
        set.insert(format!("www.{}", domain));
        set.insert(format!("m.{}", domain));
        set.insert(format!("app.{}", domain));
        let mut v: Vec<String> = set.into_iter().collect();
        v.sort();
        v
    }
    fn all_tracked_domains(&self) -> HashSet<String> {
        self.sites
            .permanent_sites
            .iter()
            .chain(self.sites.timed_sites.keys())
            .flat_map(|d| Self::domain_variants(d))
            .collect()
    }
    pub fn save(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.sites)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        std::fs::write("blocked_sites.json", json)
    }
    pub fn update_hosts(&mut self) -> io::Result<()> {
        hosts::backup(&self.hosts_path)?;
        hosts::clean(&self.hosts_path, &self.all_tracked_domains())?;
        self.expire_timed_sites();

        if self.sites.is_blocking_enabled {
            let now = SystemTime::now();

            let entries: Vec<(String, String)> = self
                .sites
                .permanent_sites
                .iter()
                .flat_map(|d| Self::domain_variants(d))
                .chain(
                    self.sites
                        .timed_sites
                        .iter()
                        .filter(|(_, exp)| exp.map_or(true, |t| t > now))
                        .flat_map(|(d, _)| Self::domain_variants(d)),
                )
                .map(|variant| (format!("127.0.0.1 {}", variant), format!("::1 {}", variant)))
                .collect();

            hosts::append_blocks(&self.hosts_path, &entries)?;
        }

        Ok(())
    }
    pub fn save_and_update(&mut self) -> Result<(), String> {
        self.save()
            .map_err(|e| format!("Failed to save site list: {}", e))?;
        self.update_hosts()
            .map_err(|e| format!("Failed to update hosts file: {}", e))
    }
    pub fn add_permanent(&mut self, domain: String) {
        self.sites.permanent_sites.insert(domain);
    }
    pub fn add_timed(&mut self, domain: String, expiry: SystemTime) {
        self.sites.timed_sites.insert(domain, Some(expiry));
    }
    pub fn remove(&mut self, domain: &str) -> bool {
        let removed_permanent = self.sites.permanent_sites.remove(domain);
        let removed_timed = self.sites.timed_sites.remove(domain).is_some();
        removed_permanent || removed_timed
    }
    pub fn toggle_blocking(&mut self) -> Result<(), String> {
        permissions::check_permissions(&self.hosts_path)?;

        self.sites.is_blocking_enabled = !self.sites.is_blocking_enabled;
        if !self.sites.is_blocking_enabled {
            hosts::clean(&self.hosts_path, &self.all_tracked_domains())
                .map_err(|e| format!("Failed to clean hosts file: {}", e))?;
        }

        self.save_and_update()
    }
    pub fn expire_timed_sites(&mut self) {
        let now = SystemTime::now();
        self.sites.timed_sites.retain(|_, exp| match *exp {
            Some(t) => t > now,
            None => true,
        });
    }
    pub fn check_permissions(&self) -> Result<(), String> {
        permissions::check_permissions(&self.hosts_path)
    }
}
