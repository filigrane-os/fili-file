# Acceptance checklist of Casier (fili-file) before 1.0

What cannot be automated: mouse, real keyboard, devices, other applications. The rest is guarded by
`tests/fili-file/run.sh` (headless scenarios, see AGENTS.md) and `cargo test`: navigation, selections, tabs,
copy/cut/paste, name conflicts, rename, trash, archives, dialogs. Do not redo them here.

**Preparation** (once):

1. `cd apps/fili-file && CARGO_TARGET_DIR=$HOME/.cache/filigrane-apps make`, then `./bootstrap.sh --apps`
   (deploys without sudo).
2. `tests/fili-file/acceptance-fixture.sh` creates `/tmp/casier-acceptance` (names with spaces, accents, emoji, a very long
   name, 5000 files, broken links, archives including `protected.zip` with the password `secret`, image, PDF, read-only
   folder, two different `report.txt` in `conflict/source` and `conflict/target`). Open Casier on it:
   `fili-file /tmp/casier-acceptance` (or Mod+E then Ctrl+L).
3. Tick as you go. For each failure: the line, what happened, what was expected (tell the assistant, who fixes it and
   reruns `tests/fili-file/run.sh`).

Shortcuts found in `qml/Main.qml`: Ctrl+C/X/V, Ctrl+D (duplicate), F2, Delete (trash), Ctrl+Z, Ctrl+A, Ctrl+Shift+I
(invert), Ctrl+Shift+M (pattern), Ctrl+F or `/` (search), Ctrl+L (path), Ctrl+H (hidden), Ctrl+1/2 (list/icons),
Ctrl+Shift+P (preview), Ctrl+I (info), Ctrl+Shift+N (new folder), Ctrl+T/W/Tab/Shift+Tab (tabs), Alt+←/→/↑, Ctrl+[ /
Ctrl+], F5 or Ctrl+R, Ctrl+Alt+T (terminal here), Ctrl+Shift+C (path), Ctrl+Alt+C (name), Ctrl+K (server),
Ctrl+Shift+R (nearby), Ctrl+Shift+Delete (empty the trash), Ctrl+Q.

## 1. Keyboard (French keyboard)

- [x] **`/`** opens the search on an AZERTY keyboard (Shift+: or the dedicated key depending on the layout), without typing a "/" in the field.
- [x] Up/down arrows move the current row; Shift+arrows extend the selection; Ctrl+A selects everything.
- [x] Backspace/Enter opens the folder or the file; Alt+↑ goes up.
- [x] Typing letters in the list jumps to the first name that starts with them; a pause resets the input; with accented names and capitals.
- [x] **Space** opens the quick look (floating `quicklook` window) of the selected item: image, PDF, text, spreadsheet, folder, video; Space, Esc or `q` closes it; Space on the same file closes it too. While typing the beginning of a name ("my f…"), the space is a letter and opens nothing. "Quick Look" entry in the menu bar.
- [ ] Delete moves to the trash; Ctrl+Z undoes. _(fixed after the first checklist run, to be checked again)_
- [x] Shortcuts do **nothing** while a name, the path or the search is being typed, and behind an open dialog (Compress, password, About, Server, Info).
- [x] Esc closes dialog, menu, rename, search, path field, in this order, without closing the window.
- [x] The focus outline is visible and follows the keyboard.

## 2. Mouse

- [x] Click selects; Ctrl+click adds or removes; Shift+click extends; click on the empty space deselects.
- [ ] Double click opens a folder, a file (default application through xdg-open), an archive (extract here). _(fixed after the first checklist run, to be checked again)_
- [ ] **Rubber band**: dragging from the empty space selects the area crossed, in list and icons; Ctrl adds. _(fixed after the first checklist run, to be checked again)_
- [ ] Right click on an item, on several items, on the empty space, on an item of the trash: suitable entries (Open with…, Rename, Duplicate, Compress/Extract, Copy path, Information…). _(fixed after the first checklist run, to be checked again)_
- [x] Right click on the header: choice of columns; a click on a column sorts, a second click reverses; **dragging the left edge of a column header** (↔ cursor) widens or narrows the column (56 to 600 px), the name takes the rest; double click on this edge = default width; the width is kept when reopening.
- [ ] Wheel and touchpad scrolling: natural inertia, no jump; fast scrolling of the `big` folder (5000 files) is smooth. _(fixed after the first checklist run, to be checked again)_
- [x] Hover: no flicker; correct cursor on the edges and the buttons.

## 3. Rename

- [ ] F2 renames in place: the name without extension is selected; Enter validates; Esc cancels; a click elsewhere validates. _(fixed after the first checklist run, to be checked again)_
- [ ] Rename to an existing name: readable error message, nothing is overwritten.
- [x] Hard names: `été à la plage.txt`, `emoji 🎉.txt`, `it's.txt`, name of 200+ characters (truncated in the middle, full name in a tooltip or in the info).
- [x] **Bulk rename** (select several, F2): live preview for Replace, Number, Case; **Apply** renames everything; an invalid or colliding name is reported and blocks; Ctrl+Z undoes the batch in one go.

## 4. Drag and drop

- [x] Dragging a file onto a folder of the list: moves; with Ctrl: copies; the cursor and the highlight of the target announce it.
- [ ] Dragging several selected items. _(fixed after the first checklist run, to be checked again)_
- [ ] Dragging onto a place of the sidebar (Documents, trash). _(fixed after the first checklist run, to be checked again)_
- [ ] Dragging onto another tab (hovering the tab) and reordering the tabs by dragging them. _(fixed after the first checklist run, to be checked again)_
- [ ] Dragging from another application (browser: image or downloaded file; terminal) onto Casier.
- [ ] Dragging from Casier to the browser (file upload area of a site) and to another application.
- [ ] Esc during the drag cancels; dropping into the original folder does nothing.
- [ ] Name conflict on drop: `conflict/source/report.txt` dragged into `conflict/target`: the dialog offers Replace / Keep both / Skip / Cancel, **each one clicked with the mouse** and validated with Enter (= Keep both), Esc (= Cancel); the final content matches the choice.

## 5. Menu of the fili-bar and context menu

- [ ] For each entry of the Casier menu in the fili-bar (File, Edit, View, Go): click = the command is done; the shortcut shown is the one of the keyboard; entries greyed out when the command is impossible (nothing selected, trash). _(fixed after the first checklist run, to be checked again)_
- [x] "About Casier": card with icon, "Also known as fili-file", version (must show 1.0.0 after the acceptance run).
- [x] The menu of the bar and the window stay in agreement after changing tab, opening the trash, showing results.

## 6. Folder, path and search

- [ ] Ctrl+L: typing a path, `~`, `..`, relative path; fzf-like suggestions appear and Tab/arrows choose them; a word without "/" jumps to an already visited folder. _(fixed after the first checklist run, to be checked again)_
- [x] Clickable breadcrumb (each segment); a very long path folds without breaking the bar.
- [x] Search (Ctrl+F): live filter; Esc clears it; the filter is cleared when changing folder.
- [x] Empty folder (`empty`): readable empty state. Folder without read permission: clear "permission denied" message, not a silent empty window.
- [x] `read-only`: New folder, paste, rename, drop give a readable error; nothing is created.
- [x] `links`: valid link opens on its target; broken link is shown (distinct icon) and does not crash; link to a folder opens; Info shows the target.
- [ ] Hidden files (Ctrl+H) appear and disappear; the state is kept when reopening. _(fixed after the first checklist run, to be checked again)_

## 7. Preview, thumbnails, info

- [x] Ctrl+Shift+P: preview of an image (`image.png`), of a PDF (`document.pdf`), of a text (`notes.txt`), of a folder; **dragging its left edge** changes its width (160 to 900 px, reduced if the window lacks room), double click = 300 px, width kept when reopening; empty when nothing is selected.
- [x] Thumbnails in icon mode (`media`): image and PDF; first opening a bit slower, the second immediate (cache `~/.cache/filigrane/thumbs`).
- [x] Info (Ctrl+I): folder size computed in the background (bar or value that updates), SHA-256 of a file, permissions as check boxes (change then undo), several items.
- [x] A folder of 5000 files opens without freezing the window; sorting by each column stays smooth.

## 8. Archives

- [x] Compress (dialog): zip, 7z, tar.gz/xz/zst; level; without hidden files; **with a password** (zip, 7z): the archive asks for this password when opened elsewhere.
- [x] Extract `plain.zip` / `plain.7z`: result next to it, free name on collision.
- [x] Double click on `protected.zip`: password prompt; a wrong password is reported and asked again; `secret` extracts.
- [x] Cancel a compression or an extraction in progress (Esc or task list): nothing partial is left.

## 9. Trash and volumes

- [x] Trash of the home folder: put, open "Trash", put back, delete permanently, empty (confirmation, cancellable).
- [x] **Trash of another volume** (USB stick or second disk): a file deleted there goes into the volume's `.Trash-UID` and shows up in Casier's Trash.
- [x] **Real USB stick**: plugging in = appears in the sidebar without restarting; a click mounts it (or automatic mount); the content is shown; the eject button unmounts it and the row disappears; an abrupt unplug does not crash.
- [x] Copy of big files to the stick: progress, cancel, queue of several copies, task list.
- [x] **MTP phone**: plugged in, appears; mounting asks gio for the address if needed; browsing, copy in both directions; ejected, disappears until it is plugged in again.
- [ ] **Server** (Ctrl+K): address `smb://…` or `sftp://…`, credentials, readable address error. _(fixed after the first checklist run, to be checked again)_
- [x] "Nearby" (Ctrl+Shift+R) and Share: list of devices, sending a file, receiving.

## 10. System integration

- [x] **Mod+E** opens Casier; a "Show in folder" link of a browser (Chrome: downloads) opens Casier with the file selected (FileManager1 service).
- [x] **File chooser portal in Chrome**: "Upload a file" (one file, several), "Save as…" (proposed name, overwrite asked), Cancel; the window is floating, at the right starting folder, type filter respected.
- [x] `xdg-open /tmp/casier-acceptance` and `inode/directory` open Casier (not Thunar).
- [x] "Open with…": list of applications, the preferred one first; choosing one launches the file.
- [x] Open a terminal here (Ctrl+Alt+T) in the right folder; copy the path/the name (Ctrl+Shift+C, Ctrl+Alt+C) then paste elsewhere.
- [x] Copy in Casier then paste in another manager (or the reverse): the files follow; cut only erases after the paste.
- [x] Multiple windows: two independent Casier windows; the assistant (Mod+Y) sees the folder and the selection of the active window.
- [x] Light and dark theme, system font size, high-density screen: no cut text, readable contrasts, sharp icons.
- [ ] Closing the window during a copy: warns or ends cleanly, without a half-copied file. _(fixed after the first checklist run, to be checked again)_

## 11. Measurements (to take on the machine, `budget`)

| Measure | Value | Goal |
|---|---|---|
| Opening (window visible) | 240 ms (3 identical tries, 2026-11-02) | < 350 ms |
| Private memory, window open on a folder | 43 MB (PSS 84 MB, RSS 155 MB) | < 40 MB |
| After opening `big` (5000 files) and the preview | to be measured | no continuous growth |
| After 20 openings/closings of folders | to be measured | back to ±5 MB |

The memory is 3 MB over the goal (V0: ≈ 30 MB). Breakdown measured on 2026-11-02: 24 MB of anonymous memory and 10 MB of heap (Qt Quick, scene graph, Mesa/LLVM driver), 1 MB of QML JavaScript heap, the rest in libraries; the Rust code of the application is not the visible cause. Decision to take: accept 43 MB (3 MB over the goal for tabs, preview, grid, dialogs) or investigate (disable useless Qt Quick layers, `QSG_RHI_BACKEND`, avoid `MultiEffect`).

## 12. Exit of the acceptance run

When everything is ticked (or the gaps accepted, noted here):

- [ ] Known gaps accepted: …
- [ ] `tests/fili-shell/run.sh` green.
- [x] `version = "1.0.0"` in `Cargo.toml`, shown in the "About" card.
- [ ] Removal of Thunar from the Filigrane distribution (packages `thunar`, `thunar-volman`, `thunar-archive-plugin` of `packages/desktop.txt`, `.thunar` blocks of the GTK3 CSS, `accels.scm` block of `bootstrap.sh`, i18n string `Thunar: Space = quick look`, `references/resources.md`), then `budget diff`.
