# NiriEww

A small program that bridges niri-ipc and eww to render icons for all open windows
on all active workspaces.

> NOTE: As far as I know, there is no good way to 100% accurately determine what icon
> an application uses, instead, this relies on the "app id" of each window and
> uses that to look up a desktop file and then an icon. This seems to work for
> most apps, but quite a few programs end up with the default icon

## Installation

Install nirieww

```
cargo install https://gitlab.com/TheZoq2/nirieww.git
```

The default icon is currently hard-coded to be
`/usr/share/icons/Adwaita/16x16/apps/help-contents-symbolic.symbolic.png` which
means that Adwaita needs to be installed for it to show up.

Create a new workspace widget that listens on `nirieww`, mine looks like

```
(deflisten workspaces :intial [] "nirieww")
(defwidget workspaces [monitor monitorName]
  (eventbox :class "workspaces" :vexpand true
    (box :space-evenly false :orientation "v" :vexpand true
      (for workspace in workspaces
        (eventbox
            ; :onclick "hyprctl dispatch workspace ${workspace.id}"
            :onclick "niri msg action focus-workspace ${workspace.idx}"
            :visible {workspace.output == monitorName ? true : false}
            :class "workspace-button ${workspace.is_active ? "current" : ""}"
            :vexpand true
          (box
              :class "workspace-entry"
              :orientation "v"
              :space-evenly false
            (label :text "${workspace.idx}")
            (for window in {workspace.windows}
                (image
                  :tooltip {window.title}
                  :path {window.icon}
                  :image-width 14))))))))
```

Since I have a multi-monitor setup and like workspaces to be separated per
monitor, this takes `monitor` and `monitorName` to hide irrelevant workspaces
per window. `monitor` is the eww mointor ID, and `monitorName` is the name of
the monitor, for example `HDMI-A-1` or whatever `wlr-randr` calls your monitor.

```
(defwidget top
  (workspaces :monitor 0 :monitorName "HDMI-A-1"))
```


