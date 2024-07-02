mod icon;

use std::{
    io::{stdout, Write},
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{anyhow, bail, Context},
    Result,
};
use icon::{IconCache, DEFAULT_ICON};
use niri_ipc::{Request, Response};
use serde::Serialize;

#[derive(Serialize)]
pub struct Window {
    title: Option<String>,
    icon: Option<PathBuf>,
}
#[derive(Serialize)]
pub struct Workspace {
    output: Option<String>,
    idx: u8,
    is_active: bool,
    windows: Vec<Window>,
}

fn tick(icon_cache: &mut IconCache) -> Result<()> {
    let sock = niri_ipc::Socket::connect().context("Failed to connect to niri socket")?;
    let response = sock
        .send(Request::Workspaces)
        .context("Failed to ask for workspaces")?;

    match response {
        Ok(Response::Workspaces(workspaces)) => {
            let result = workspaces
                .iter()
                .map(|ws| {
                    Ok(Workspace {
                        output: ws.output.clone(),
                        idx: ws.idx,
                        is_active: ws.is_active,
                        windows: ws
                            .windows
                            .iter()
                            .map(|w| {
                                Ok(Window {
                                    title: w.title.clone(),
                                    icon: w
                                        .app_id
                                        .as_ref()
                                        .and_then(|app_id| icon_cache.get_icon(&app_id).clone()),
                                })
                            })
                            .collect::<Result<Vec<_>>>()?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            println!(
                "{}",
                serde_json::to_string(&result).context("Failed to encode as json")?
            );

            Ok(())
        }
        Ok(_) => bail!("Got unexpected response from workspace request"),
        Err(e) => bail!("Got error {e} from workspace request"),
    }
}

fn main() -> Result<()> {
    let mut icon_cache = IconCache::new();

    loop {
        if let Err(e) = tick(&mut icon_cache) {
            eprint!("{e:#?}")
        }

        std::thread::sleep(Duration::from_millis(100))
    }
}
