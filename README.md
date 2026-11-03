# Casier (`fili-file`)

A fast, keyboard-friendly file manager for Wayland desktops. Written in Rust, with a Qt Quick interface compiled into a
single binary. Part of [Filigrane](https://github.com/filigrane-os/filigrane), but it does not need it: it runs on any
Linux distribution with Qt 6 and a Wayland compositor.

The program is `fili-file`; **Casier** is the name you see. Both commands do the same thing:

```
casier ~/Documents
fili-file ~/Documents
```

## Features

- List and icon views, tabs (drag a tab out to detach it), live updates of the folder shown (inotify)
- Copy, cut, paste, drag and drop, bulk rename, undo, trash (including the trash of other volumes)
- Quick look (Space), preview panel, thumbnails for images and PDF, file info with SHA-256
- Compress and extract (zip, 7z, tar.gz/xz/zst, with passwords) when `fili-archive` is installed
- Volumes, USB drives, MTP phones and network places (SMB, SFTP) through udisks and gvfs
- File chooser portal backend (`org.freedesktop.impl.portal.FileChooser`) so browsers use it for "Open" and "Save as"
- Controllable from a terminal or an assistant: `fili-file ctl actions`, `fili-file ctl state`, `fili-file ctl action NAME`
- Any freedesktop icon theme, light and dark mode that follows the desktop, fully themable colors and fonts

## Install

```
make                  # build
sudo make install     # installs under /usr/local; PREFIX=/usr for a system package, DESTDIR for packagers
```

The build needs a Rust toolchain (edition 2024, Rust 1.85 or newer), a C++17 compiler, `pkg-config` and the Qt 6
development files (base, declarative, svg). `make` finds `qmake` by itself; set `QMAKE=/path/to/qmake6` if yours is
elsewhere. A linker such as `mold` or `lld` makes linking much faster.

| | Build | Run |
|---|---|---|
| Void | `qt6-base-devel qt6-declarative-devel qt6-svg-devel` | `qt6-declarative qt6-svg qt6-wayland wl-clipboard glib shared-mime-info adwaita-icon-theme util-linux` |
| Arch | `qt6-base qt6-declarative qt6-svg` | `qt6-wayland wl-clipboard glib2 shared-mime-info adwaita-icon-theme util-linux` |
| Debian / Ubuntu | `qt6-base-dev qt6-declarative-dev libqt6svg6-dev` | `qt6-wayland wl-clipboard libglib2.0-bin shared-mime-info adwaita-icon-theme` |
| Fedora | `qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qtsvg-devel` | `qt6-qtwayland wl-clipboard glib2 shared-mime-info adwaita-icon-theme` |

Only the Void names were checked against a package repository; the others are the usual ones and may differ slightly.

Optional programs: `poppler` / `poppler-utils` (`pdftoppm`, PDF thumbnails), `udisks2` (mount and eject), `gvfs`
(network places, phones), `fili-archive` (compress and extract), `fili-share` (Nearby and Share). A feature whose
program is missing is switched off; nothing else depends on them.

The fonts (Jost, JetBrains Mono, Fili Icons) are inside the binary: no font to install.

## Configuration

`~/.config/filigrane/fili-file.toml`, optional. Flat settings:

```toml
view = "list"             # "list" or "grid"
show_hidden = false
sort = "name"             # name, size, date, kind, created, permissions, owner
sort_desc = false
columns = "size,modified"
start = "~/Documents"     # folder opened when started without an argument
icon_theme = "Papirus"    # default: the desktop's icon theme
folder_icon = "folder"    # icon name for every folder (default: the theme's, else "folder")
file_icon = ""            # icon name for every regular file (default: by type)
```

Colors, fonts, sizes and radii live in the `[theme]`, `[theme.dark]`, `[theme.light]` and `[theme.accents]` sections.
`~/.config/filigrane/theme.toml` (same sections) applies to every Filigrane application, and the defaults are in
[`theme.toml`](theme.toml). The first one found wins, in this order: `fili-file.toml`, `theme.toml`, built-in defaults.

**Icon themes.** Casier follows the freedesktop icon theme specification (`index.theme`, `Inherits`, hicolor last) and
uses the theme set in `icon_theme`, else the desktop's (gsettings, then GTK `settings.ini`), else Adwaita. A theme can
add three optional hints to its `index.theme`, ignored by other applications: `X-Fili-FolderIcon`, `X-Fili-FileIcon`
and `X-Fili-GlyphDir`.

**Light and dark.** Followed from the desktop (settings portal, `org.freedesktop.appearance`), live. Filigrane's own
state file takes precedence when it exists.

**Languages.** English, plus a French catalog built into the binary. To add one without recompiling, put
`<language>.json` (same format as [`i18n/fr.json`](i18n/fr.json)) in `$XDG_DATA_DIRS/fili-file/i18n/`.

## File chooser portal

`make install` also installs the backend description for `xdg-desktop-portal`. It is only used on the desktops listed in
`filigrane.portal` (`UseIn=`, niri by default); to use it elsewhere, add it to your `portals.conf`:

```
[preferred]
org.freedesktop.impl.portal.FileChooser=filigrane
```

## Development

```
make test                      # unit tests
scripts/check-i18n.py          # the French catalog matches the texts of the code
```

The `tests/` directory drives a headless instance (`QT_QPA_PLATFORM=offscreen`) through `fili-file ctl`, in a throwaway
`HOME`, and checks the result on the file system.

## License

GPL-3.0-or-later, see [`LICENSE`](LICENSE). The embedded fonts keep their own licenses (SIL OFL for Jost and JetBrains
Mono, ISC for the Lucide icons behind Fili Icons) in [`licenses/`](licenses/). The application icon is derived from
Nordic-Slot-Dark-Icons (GPL-3.0).
