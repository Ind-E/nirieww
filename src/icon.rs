use std::{path::{PathBuf, Path}, collections::BTreeMap};

use color_eyre::{eyre::Context, Result};
use freedesktop_desktop_entry::{default_paths, DesktopEntry};
use hyprland::data::Client;
use xdgkit::icon_finder;

pub const DEFAULT_ICON: &str = "/usr/share/icons/Adwaita/16x16/apps/help-contents-symbolic.symbolic.png";

pub fn client_icon(client: &Client) -> Result<PathBuf> {
    let paths = default_paths();

    let desktop_file = paths
        .iter()
        .find_map(|p| {
            let file = p.join(&format!("{}.desktop", client.class));
            if file.exists() {
                Some(file)
            }
            else {
                None
            }
        })
        .map(|df| -> Result<_> {
            eprintln!("{df:?}");
            let content = std::fs::read_to_string(&df)
                .with_context(|| format!("Failed to read desktop entry {df:?}"))?;

            let entry = DesktopEntry::decode(&df, &content)?;

            Ok(entry
                .desktop_entry("Icon")
                .and_then(|icon_name| icon_finder::find_icon(icon_name.to_string(), 128, 1)))
        })
        .transpose()?
        .unwrap_or_else(|| icon_finder::find_icon("default-application".to_string(), 128, 1))
        .unwrap_or_else(|| {
            PathBuf::from(DEFAULT_ICON)
        });

    Ok(desktop_file)
}

pub struct IconCache {
    inner: BTreeMap<String, Result<PathBuf>>
}

impl IconCache {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn get_icon(&mut self, client: &Client) -> &Result<PathBuf> {
        if !self.inner.contains_key(&client.class) {
            let result = client_icon(client);

            self.inner.insert(client.class.clone(), result);
        }

        &self.inner[&client.class]
    }
}
