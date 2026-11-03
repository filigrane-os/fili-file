# Changelog

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
