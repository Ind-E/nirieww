use freedesktop_desktop_entry::default_paths;
use hyprland::data::Client;

fn client_icon(client: Client) -> PathBuf {
    let paths = default_paths();

    let desktop_file = paths.iter().find(|p| {
        let file = p.join(client.class).with_extension("desktop");
        file.exists()
    });


}
