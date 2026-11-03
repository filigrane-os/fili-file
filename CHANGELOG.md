# Changelog

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
