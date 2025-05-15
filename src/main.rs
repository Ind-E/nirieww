mod icon;

use std::{collections::HashMap, path::PathBuf};

use color_eyre::{
    eyre::{bail, Context},
    Result,
};
use icon::IconCache;
use itertools::Itertools;
use niri_ipc::{Event, Request};
use serde::Serialize;

#[derive(Serialize)]
pub struct Window<'a> {
    title: &'a Option<String>,
    id: &'a u64,
    icon: Option<PathBuf>,
}
#[derive(Serialize)]
pub struct Workspace<'a> {
    output: &'a Option<String>,
    idx: &'a u8,
    is_active: &'a bool,
    windows: Vec<Window<'a>>,
}

#[derive(Default)]
struct State {
    workspaces: HashMap<u64, niri_ipc::Workspace>,
    windows: HashMap<u64, niri_ipc::Window>,
}

impl State {
    pub fn on_event(&mut self, event: Event) {
        match event {
            Event::WorkspacesChanged { workspaces } => {
                self.workspaces = workspaces.into_iter().map(|ws| (ws.id, ws)).collect()
            }
            Event::WindowsChanged { windows } => {
                self.windows = windows.into_iter().map(|w| (w.id, w)).collect()
            }
            Event::WindowOpenedOrChanged { window } => {
                self.windows.insert(window.id, window);
            }
            Event::WindowClosed { id } => {
                self.windows.remove(&id);
            }
            Event::WorkspaceActivated { id, focused } => {
                let output = self.workspaces.iter().find_map(|(wid, ws)| {
                    if wid == &id {
                        ws.output.clone()
                    } else {
                        None
                    }
                });
                for (_, ws) in &mut self.workspaces {
                    if ws.output == output {
                        ws.is_active = false;
                    }
                }
                self.workspaces.get_mut(&id).map(|ws| {
                    ws.is_active = true;
                    ws.is_focused = focused;
                });
            }
            Event::WorkspaceActiveWindowChanged { .. } => {}
            Event::WindowFocusChanged { .. } => {}
            Event::KeyboardLayoutsChanged { .. } => {}
            Event::KeyboardLayoutSwitched { .. } => {}
        }
    }

    pub fn print(&self, icon_cache: &mut IconCache) -> Result<()> {
        let printable = self
            .workspaces
            .iter()
            .sorted_by_key(|(_, ws)| ws.idx)
            .map(|(_, ws)| Workspace {
                output: &ws.output,
                idx: &ws.idx,
                is_active: &ws.is_active,
                windows: self
                    .windows
                    .iter()
                    .filter_map(|(_, w)| {
                        if w.workspace_id == Some(ws.id) {
                            Some(Window {
                                title: &w.title,
                                id: &w.id,
                                icon: w
                                    .app_id
                                    .as_ref()
                                    .and_then(|app_id| icon_cache.get_icon(app_id).clone()),
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>();

        println!(
            "{}",
            serde_json::to_string(&printable).context("Failed to encode as json")?
        );
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut icon_cache = IconCache::new();

    let sock = niri_ipc::socket::Socket::connect().context("Failed to connect to niri socket")?;
    let response = sock
        .send(Request::EventStream)
        .context("Failed to ask for workspaces")?;

    let mut state = State::default();

    match response {
        (Ok(response), mut rest) => {
            match response {
                niri_ipc::Response::Handled => {}
                other => bail!("Niri responsed unexpectedly {other:?}"),
            }
            loop {
                let e = rest()?;
                state.on_event(e);
                state.print(&mut icon_cache)?;
            }
        }
        (Err(e), _) => bail!("Failed to connect to socket {e}"),
    }
}
