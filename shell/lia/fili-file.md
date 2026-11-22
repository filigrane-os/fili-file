# Casier (`fili-file`)

Casier is the file manager. Its window is commanded with `fili-file ctl [--pid PID]` (the window in front by default):
- `state` (folder, selection, view), `actions` (every action of its menu), `action NAME [ARG]` (the same action as the menu or the shortcut, with Casier's own undo and confirmations), `goto PATH`, `reveal PATH`, `select [--add|--none|--pattern GLOB] < paths`, `tab new|close|switch|move|detach`, `view list|icons`, `hidden on|off`, `preview on|off`, `sort COLUMN [asc|desc]`.
- "This", "these files", "here" = the selection and folder of the window: `fili-file ctl state`, or the `selection` and `location` fields of `~/.local/state/filigrane/context.json`.
- Fast path: for a named file or folder, go straight to `fili-tx`, no need to open the window or read its state. `ctl actions` only when you do not know an action name.

Files
- A change to files (rename, move, trash, copy, new folder, batch) goes through `fili-tx` (`mv -` for a batch on standard input, `trash`, `mkdir`, `cp`), never `mv`/`rm`/`cp` alone. Compress and extract with `fili-archive pack|unpack`. When the user watches the window, prefer `fili-file ctl action NAME` for a single action: Casier shows the result and can undo it itself.
- Searching and reading stay in the shell (`fd`, `rg`, `find`, `file`, `du`), read-only, and bounded: never `find /` or a recursive listing of the whole home.
- Never empty the Trash, never touch `~/.ssh`, `/etc`, other volumes or hidden folders unless the user names them.

Code (repository `filigrane-os/fili-file`, clone `~/Projects/fili-file`, pinned by `apps/fili-file/upstream`): a change is made there (`make check`), tested live by the user before any tag; the fallback menu `dotfiles/config/filigrane/appmenu/fili-file.json` stays identical to the application's menu (`tests/fili-shell/check-fili-file-menu.py`).
