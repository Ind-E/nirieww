use std::{
    collections::BTreeMap,
    path::PathBuf,
};

use color_eyre::{eyre::Context, Result};
use freedesktop_desktop_entry::{default_paths, DesktopEntry};
use xdgkit::icon_finder;

pub const DEFAULT_ICON: &str =
    "/usr/share/icons/Adwaita/16x16/apps/help-contents-symbolic.symbolic.png";

pub fn client_icon(app_id: &str) -> Result<PathBuf> {
    let paths = default_paths();

    let desktop_file = paths
        .iter()
        .find_map(|p| {
            let file = p.join(&format!("{}.desktop", app_id));
            if file.exists() {
                Some(file)
            } else {
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
        .unwrap_or_else(|| PathBuf::from(DEFAULT_ICON));

    Ok(desktop_file)
}

pub struct IconCache {
    inner: BTreeMap<String, Option<PathBuf>>,
}

impl IconCache {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn get_icon(&mut self, app_id: &str) -> &Option<PathBuf> {
        if !self.inner.contains_key(app_id) {
            let result = client_icon(app_id);

            self.inner.insert(
                app_id.to_string(),
                result
                    .map_err(|e| eprintln!("Failed to get icon for {app_id}. {e}"))
                    .ok(),
            );
        }

        &self.inner[app_id]
    }
}
