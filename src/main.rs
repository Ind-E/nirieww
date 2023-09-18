mod icon;

use std::{
    io::{stdout, Write},
    path::PathBuf,
    sync::{Arc, Mutex}, process::Command,
};

use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{anyhow, Context},
    Result,
};
use hyprland::{
    data::{Clients, Monitor, Monitors, Workspaces},
    event_listener::EventListener,
    shared::HyprData,
};
use icon::{IconCache, DEFAULT_ICON};
use serde::Serialize;

#[derive(Serialize)]
struct WorkspaceInformation {
    name: String,
    id: i32,
    icons: Vec<String>,
    monitor: i64,
}

fn get_workspaces_on_monitor(
    monitor: &Monitor,
    icons: &mut IconCache,
) -> Result<Vec<WorkspaceInformation>> {
    let mut workspaces = Workspaces::get()?
        .filter(|w| w.monitor == monitor.name)
        .map(|w| {
            let clients = Clients::get()?
                .filter(|c| c.workspace.id == w.id)
                .map(|c| {
                    let icon = icons.get_icon(&c);
                    icon.as_ref()
                        .map(|i| i.clone())
                        .map_err(|e| eprintln!("Icon lookup error {e:#?}"))
                        .unwrap_or_else(|_| PathBuf::from(DEFAULT_ICON))
                        .to_string_lossy()
                        .to_string()
                })
                .collect();

            Ok(WorkspaceInformation {
                id: w.id,
                name: format!("{}", w.id % 10),
                icons: clients,
                monitor: monitor.id,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let existing_ids = workspaces.iter().map(|w| w.id).collect::<Vec<_>>();
    // Fill out the list with empty workspaces that still fit on this monitor
    workspaces.append(
        &mut (1..10)
            .filter_map(|id| {
                let expected_id = id + monitor.id * 10;
                if !existing_ids.contains(&(expected_id as i32)) {
                    Some(WorkspaceInformation {
                        name: format!("{id}"),
                        id: expected_id as i32,
                        icons: vec![],
                        monitor: monitor.id,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    );

    workspaces.sort_by_key(|w| w.id);

    Ok(workspaces)
}

fn print_workspaces(icons: &mut IconCache) -> Result<()> {
    let monitors = Monitors::get()?;

    let result = monitors
        .iter()
        .map(|m| get_workspaces_on_monitor(&m, icons))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    println!(
        "{}",
        serde_json::to_string(&result).with_context(|| "Failed to encode workspace information")?
    );

    stdout().flush().ok();

    Ok(())
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Cli,
}

#[derive(Subcommand)]
enum Cli {
    Workspaces,
    ActiveWorkspace,
    CreateBars,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut event_listener = EventListener::new();
    let icon_cache = Arc::new(Mutex::new(IconCache::new()));

    macro_rules! listen {
        ($event:ident, $listener:tt) => {
            event_listener.$event($listener)
        };
    }

    macro_rules! listen_all {
        ($listener:tt) => {{
            listen!(add_active_monitor_change_handler, $listener);
            listen!(add_active_window_change_handler, $listener);
            listen!(add_layer_closed_handler, $listener);
            listen!(add_layer_open_handler, $listener);
            listen!(add_monitor_added_handler, $listener);
            listen!(add_monitor_removed_handler, $listener);
            listen!(add_urgent_state_handler, $listener);
            listen!(add_window_close_handler, $listener);
            listen!(add_window_moved_handler, $listener);
            listen!(add_window_open_handler, $listener);
            listen!(add_workspace_added_handler, $listener);
            listen!(add_workspace_change_handler, $listener);
            listen!(add_workspace_destroy_handler, $listener);
        }};
    }

    match args.command {
        Cli::Workspaces => {
            listen_all!({
                let icon_cache = icon_cache.clone();
                move |_| {
                    let mut icon_cache = icon_cache.lock().unwrap();
                    print_workspaces(&mut icon_cache)
                        .map_err(|e| eprintln!("{e:#?}"))
                        .ok();
                }
            });
        }
        Cli::ActiveWorkspace => {
            let listener = || if let Ok(mut monitors) = Monitors::get() {
                eprintln!("Updating active workspace");
                if let Some(ws) = monitors.find(|m| m.focused).map(|m| m.active_workspace) {
                    println!("{}", ws.id);
                    stdout().flush().ok();
                }
            };

            listener();
            event_listener.add_workspace_change_handler(move |_| listener());
            event_listener.add_active_monitor_change_handler(move |_| listener());
        }
        Cli::CreateBars => {
            for m in Monitors::get()? {
                let status = Command::new("eww").arg("open").arg(format!("bar_{}", m.id)).status()?;

                if !status.success() {
                    eprintln!("Failed to open bar_{}", m.id)
                }
            }

            return Ok(())
        }
    }

    event_listener.start_listener().unwrap();

    Ok(())
}
