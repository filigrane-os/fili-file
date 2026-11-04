# Changelog

## Unreleased

- Mouse wheel: the inertia handler of the lists took the wheel events of a mouse and ignored them, so nothing
  scrolled. It now scrolls by steps for a wheel and keeps the momentum for a touchpad.
- Open With: when the chosen application is already running (it opens the file in its existing window, which Wayland
  does not let it focus), Casier brings that window to the front under niri.

## 1.4.2

- Window panel: keyboard navigation. The first tile is selected when it opens; arrows move (left/right through the
  tiles, up/down by rows), Home/End, Tab, Enter or Space runs the tile, Esc closes. A pointer resting under the panel
  does not take the selection.

## 1.4.1

- Window buttons: the right margin of the Close button equals the margin above it, and the Window button is a plain
  square. "Show Window Buttons" and "Window Actions…" moved from the View menu to a new Window menu of the bar.

## 1.4.0

- Two buttons at the far right of the toolbar: Close (a cross), and Window, which opens a panel of window actions, one
  clickable tile with its own pictogram each. Under niri: fullscreen, maximize, fill the width, preset widths and
  heights, center, floating, move or join columns, tabbed column, overview, send to another workspace or monitor,
  screenshot. On any other desktop: fullscreen, maximize, minimize.
- Hide them with `window_buttons = false` in `fili-file.toml`, or with View > Show Window Buttons (the menu choice
  wins). View > Window Actions… (Ctrl+Shift+W) opens the panel from the keyboard.

## 1.3.2

- A network share opened from its mount folder is named "share on Server" (for example "sauvegarde on Nas") in the window
  title, the path bar and the Recents list, instead of the technical `smb-share:server=…,share=…,user=…`. The path bar
  starts at the share and no longer shows the `gvfs` folders.

## 1.3.1

- Default accent: sand in dark mode, slate in light mode (it was mist / black). Folder icons follow. A chosen accent is unchanged.
- Type scale +1 px (14 / 13 / 12 / 11): Jost has a small x-height, so the sizes copied from Apple's Lucida Grande looked small.

## 1.3.0

- Servers in the sidebar: one row per server (`Nas`), no longer one per share. A click lists the server's shares as
  folders; opening one connects it and opens it. The cross forgets the server and its password.
- Remember password: a check box in Connect to Server keeps the password in the user's keyring (Secret Service, through
  `secret-tool`: gnome-keyring, KeePassXC...), never in a file of ours. A kept password is used by every later connection
  to that server, so a click asks for nothing.
- Favorites and recent folders inside a network share stay in the sidebar when the share is not mounted; a click
  reconnects the share (with the kept password) and opens the folder. When there is no kept password or it was refused,
  Connect to Server opens with the address filled in.
- A network address typed in the path bar opens the server (`smb://host/`) or connects the share (`smb://host/share`).
- Shares and other network entries without a date show a dash instead of 1970.

## 1.2.3

- Fixed: Connect to Server (Ctrl+K, and a server clicked in the sidebar) did nothing: the button showed "Connecting…"
  forever because `connectServer` was never exposed to the interface. It has not worked since it was added. A new test
  makes the build fail when the QML calls a function that is not exposed.

## 1.2.2

- Connecting to a server writes a journal, `~/.cache/fili-file/connect.log`: the questions gio asked, which kind of answer
  was sent (never a password) and how it ended. Read it when a server will not connect.

## 1.2.1

- Connecting to an SMB server given without a share (`smb://host/`, what a discovered server fills in) now works: an SMB
  server has no folder of its own, so Casier lists its shares with the credentials you typed and connects each of them (up
  to 8). The first one opens, the others appear in the sidebar.
- A refused password fails at once with "Wrong user name or password" instead of waiting 60 seconds.

## 1.2.0

- Servers on the local network are discovered: those that announce `_smb._tcp` or `_sftp-ssh._tcp` (multicast DNS: NAS
  boxes, Macs, Linux machines running Samba or SSH) appear in the sidebar under Locations. A click opens Connect to Server
  with the address already filled in, so only the user name and the password are left to type. The scan runs when the
  window opens and every two minutes, without any extra daemon (avahi is not needed). Windows PCs usually do not announce
  themselves this way: Connect to Server still reaches them.

## 1.1.0

- Compress and extract no longer need the Filigrane `fili-archive` tool: Casier has its own archive code
  (`fili-file archive pack|unpack`) that drives `tar`, `zip` and `7z`. `fili-archive` is still used when it is installed.
- Light mode: a theme can ship `<name>-light.svg` next to an icon, used instead of it on a light background. The pale file
  icon of the Filigrane theme was nearly invisible on a light background.

## 1.0.1

- Fixed: dragging an item onto a folder inside Casier copied it instead of moving it. Ctrl now asks for a copy; a drag from
  another application keeps the action it proposes.

## 1.0.0

First stable release.

- Works on any distribution: any freedesktop icon theme, fonts embedded in the binary, own translation catalog, optional
  programs detected at run time (terminal, `fili-archive`, `fili-share`), light and dark mode from the desktop.
- `casier` command alongside `fili-file`.
- `make install`, GPL-3.0-or-later license.

### Not re-verified by hand before this release

The automated checks pass (unit tests and the headless scenarios of `tests/`). These manual checks of the acceptance
list (`ACCEPTANCE.md`) were not repeated for 1.0.0, because they need a pointer, other applications or real devices:
Delete and Ctrl+Z, double-click on folders, files and archives, rubber-band selection, context menus, wheel and
touchpad scrolling, in-place rename and rename conflicts, drag and drop (several items, sidebar places, tabs, from and
to other applications, Esc to cancel, name conflicts), the menu entries of the fili-bar, path entry and suggestions,
hidden files, connecting to a server (Ctrl+K), closing the window during a copy.
