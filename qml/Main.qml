// Fenêtre de fili-file: barre d'outils (précédent, suivant, chemin), barre latérale, liste, barre d'état.
// Raccourcis: Alt+← / Alt+→ historique, Alt+↑ parent, Ctrl+L chemin, Ctrl+F recherche dans le dossier, Ctrl+H fichiers cachés, F5 relire,
// Ctrl+C / Ctrl+X / Ctrl+V copier, couper, coller, F2 renommer, Ctrl+Maj+N nouveau dossier, Suppr ou Ctrl+Retour
// arrière corbeille, Ctrl+Z annuler, Ctrl+A tout sélectionner, Échap annule la copie en cours,
// Ctrl+Maj+Suppr vide la corbeille, Ctrl+W ou Ctrl+Q fermer; boutons précédent/suivant de la souris.
import QtQuick
import Fili.Files

Window {
    id: win
    width: 900
    height: 560
    minimumWidth: 520
    minimumHeight: 300
    visible: true
    color: FiliTheme.nuit
    title: folder.path === "/" ? "/" : folder.path.split("/").pop()

    FolderModel { id: folder }

    // Message passager (volumes): effacé après 6 s
    property string notice: ""
    Timer { id: noticeTimer; interval: 6000; onTriggered: win.notice = "" }
    Connections {
        target: Fili
        function onMounted(path) {
            win.open(path);
        }
        function onVolumeError(message) {
            win.notice = message;
            noticeTimer.restart();
        }
    }
    // Un volume branché ou une corbeille vidée en dehors: relire les lieux au retour dans la fenêtre
    onActiveChanged: if (active) sidebar.reload()

    function show(row) {
        if (row >= 0)
            files.select(row);
    }
    function open(path) {
        const row = folder.go(path);
        if (row >= 0 || folder.error === "") {
            pathBar.editing = false;
            show(row);
            files.focusList();
        }
    }

    Component.onCompleted: {
        show(folder.go(Fili.startPath()));
        files.focusList();
    }

    Shortcut { sequence: "Alt+Left"; onActivated: win.show(folder.back()) }
    Shortcut { sequence: "Alt+Right"; onActivated: win.show(folder.forward()) }
    Shortcut { sequence: "Ctrl+["; onActivated: win.show(folder.back()) }
    Shortcut { sequence: "Ctrl+]"; onActivated: win.show(folder.forward()) }
    Shortcut { sequence: "Ctrl+L"; onActivated: pathBar.edit() }
    Shortcut { sequence: "Ctrl+F"; onActivated: searchInput.forceActiveFocus() }
    Shortcut { sequence: "Ctrl+H"; onActivated: win.show(folder.toggleHidden(files.currentIndex)) }
    Shortcut { sequence: "F5"; onActivated: win.show(folder.refresh(files.currentIndex)) }
    Shortcut { sequence: "Ctrl+R"; onActivated: win.show(folder.refresh(files.currentIndex)) }
    Shortcut { sequences: ["Ctrl+W", "Ctrl+Q"]; onActivated: Qt.quit() }

    // Pas de raccourcis de fichiers pendant une saisie (renommage, chemin) ni sous la confirmation
    readonly property bool typing: menu.visible || files.editing || pathBar.editing || searchInput.activeFocus || confirm.visible
    function trashSelection() {
        const row = folder.trash(files.targetsText(), files.currentIndex);
        files.reloaded();
        win.show(row);
    }
    function newFolder() {
        const row = folder.newFolder();
        if (row >= 0) {
            files.select(row);
            files.startRename();
        }
    }
    function putBack() {
        win.show(folder.putBack(files.targetsText(), files.currentIndex));
    }
    // Menu contextuel: entrées selon la sélection (aucune: le dossier lui-même)
    function openMenu(p) {
        const t = files.targets().filter(path => files.selected[path]);
        const one = t.length === 1;
        const trash = folder.inTrash;
        const archives = t.length > 0 && folder.allArchives(t.join("\n"));
        const list = [];
        if (t.length > 0) {
            list.push({ id: "open", label: Fili.tr("Open"), enabled: one });
            if (one && !folder.pathIsDir(t[0])) {
                const apps = JSON.parse(folder.openWithList(t[0]));
                list.push({
                    id: "with", label: Fili.tr("Open With"),
                    children: apps.length > 0 ? apps.map(a => ({ id: "app:" + a.id, label: a.name })) : [{ id: "", label: Fili.tr("No application"), enabled: false }]
                });
            }
            list.push({ id: "quicklook", label: Fili.tr("Quick Look"), hint: "Space", enabled: one });
            list.push({ separator: true });
            list.push({ id: "copy", label: Fili.tr("Copy"), hint: "Ctrl+C" });
            if (!trash)
                list.push({ id: "cut", label: Fili.tr("Cut"), hint: "Ctrl+X" });
        }
        if (!trash)
            list.push({ id: "paste", label: Fili.tr("Paste"), hint: "Ctrl+V", enabled: folder.canPaste });
        if (t.length > 0) {
            list.push({ separator: true });
            if (!trash)
                list.push({ id: "rename", label: Fili.tr("Rename"), hint: "F2", enabled: one });
            if (trash)
                list.push({ id: "putback", label: Fili.tr("Put Back") });
            else
                list.push({ id: "trash", label: Fili.tr("Move to Trash"), hint: "Del" });
            if (!trash) {
                list.push({ separator: true });
                if (archives)
                    list.push({ id: "extract", label: Fili.tr("Extract Here") });
                list.push({ id: "compress", label: Fili.tr("Compress") });
            }
        }
        if (!trash) {
            list.push({ separator: true });
            list.push({ id: "newfolder", label: Fili.tr("New Folder"), hint: "Ctrl+Shift+N" });
        } else {
            list.push({ separator: true });
            list.push({ id: "empty", label: Fili.tr("Empty Trash") });
        }
        menu.open(p.x, p.y, list);
    }
    function menuAction(id) {
        const t = files.targetsText();
        if (id === "open")
            win.show(folder.activate(files.currentIndex));
        else if (id.startsWith("app:"))
            folder.openWith(id.slice(4), files.targets()[0]);
        else if (id === "quicklook")
            folder.quicklook(files.currentIndex);
        else if (id === "copy")
            folder.copy(t);
        else if (id === "cut")
            folder.cut(t);
        else if (id === "paste")
            folder.paste();
        else if (id === "rename")
            files.startRename();
        else if (id === "trash")
            win.trashSelection();
        else if (id === "putback")
            win.putBack();
        else if (id === "extract")
            folder.extract(t);
        else if (id === "compress")
            folder.compress(t);
        else if (id === "newfolder")
            win.newFolder();
        else if (id === "empty")
            confirm.visible = true;
        files.focusList();
    }
    Shortcut { sequence: "Ctrl+C"; enabled: !win.typing; onActivated: folder.copy(files.targetsText()) }
    Shortcut { sequence: "Ctrl+X"; enabled: !win.typing; onActivated: folder.cut(files.targetsText()) }
    Shortcut { sequence: "Ctrl+V"; enabled: !win.typing; onActivated: folder.paste() }
    Shortcut { sequence: "Ctrl+A"; enabled: !win.typing; onActivated: files.selectAll() }
    Shortcut { sequence: "Ctrl+Z"; enabled: !win.typing; onActivated: folder.undo() }
    Shortcut { sequence: "F2"; enabled: !win.typing; onActivated: files.startRename() }
    Shortcut { sequence: "Ctrl+Shift+N"; enabled: !win.typing; onActivated: win.newFolder() }
    Shortcut { sequences: ["Delete", "Ctrl+Backspace"]; enabled: !win.typing; onActivated: win.trashSelection() }
    Shortcut { sequence: "Ctrl+Shift+Delete"; enabled: !win.typing && folder.inTrash; onActivated: confirm.visible = true }
    Shortcut { sequence: "Escape"; enabled: !win.typing && folder.busy; onActivated: folder.cancel() }

    // Boutons précédent/suivant de la souris, partout dans la fenêtre
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.BackButton | Qt.ForwardButton
        onClicked: e => win.show(e.button === Qt.BackButton ? folder.back() : folder.forward())
    }

    Rectangle {
        id: toolbar
        width: parent.width
        height: FiliTheme.toolbarHeight
        color: FiliTheme.nuit
        // Boutons précédent/suivant réunis dans une capsule
        Rectangle {
            x: nav.x - 4
            anchors.verticalCenter: nav.verticalCenter
            width: nav.width + 8
            height: 28
            radius: 8
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
        }
        Row {
            id: nav
            x: FiliTheme.space2
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2
            IconButton {
                glyph: ""
                enabledState: folder.canBack
                onClicked: win.show(folder.back())
            }
            IconButton {
                glyph: ""
                enabledState: folder.canForward
                onClicked: win.show(folder.forward())
            }
        }
        // Recherche dans le dossier affiché (Ctrl+F): filtre les noms, Échap efface
        Rectangle {
            id: search
            anchors { right: parent.right; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
            width: 170
            height: 24
            radius: FiliTheme.radiusControl
            color: FiliTheme.nuit2
            border.width: 1
            border.color: searchInput.activeFocus ? FiliTheme.voile : FiliTheme.nuit4
            Text {
                anchors { left: parent.left; leftMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                visible: searchInput.text === ""
                text: Fili.tr("Search")
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: FiliTheme.ardoise
            }
            TextInput {
                id: searchInput
                anchors { fill: parent; leftMargin: FiliTheme.space2; rightMargin: FiliTheme.space2 }
                verticalAlignment: TextInput.AlignVCenter
                clip: true
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: FiliTheme.brume
                selectionColor: FiliTheme.nuit4
                selectedTextColor: FiliTheme.brume
                onTextChanged: if (text !== folder.filter) folder.setFilter(text)
                Keys.onEscapePressed: {
                    text = "";
                    files.focusList();
                }
                Keys.onReturnPressed: files.focusList()
                Keys.onDownPressed: files.focusList()
            }
            Connections {
                target: folder
                function onChanged() {
                    if (folder.filter === "" && searchInput.text !== "")
                        searchInput.text = "";
                }
            }
        }
        Row {
            id: trashTools
            anchors { right: search.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
            spacing: 2
            visible: folder.inTrash
            TextButton {
                label: Fili.tr("Put back")
                onClicked: win.putBack()
            }
            TextButton {
                label: Fili.tr("Empty Trash")
                onClicked: confirm.visible = true
            }
        }
        PathBar {
            id: pathBar
            anchors { left: nav.right; leftMargin: FiliTheme.space3; right: trashTools.visible ? trashTools.left : search.left; rightMargin: FiliTheme.space3; top: parent.top; bottom: parent.bottom }
            path: folder.path
            onOpen: p => win.open(p)
            onDone: {
                editing = false;
                files.focusList();
            }
        }
    }
    Rectangle {
        id: toolbarLine
        anchors.top: toolbar.bottom
        width: parent.width
        height: 1
        color: FiliTheme.nuit4
    }

    Sidebar {
        id: sidebar
        anchors { top: toolbarLine.bottom; bottom: statusbar.top; left: parent.left }
        width: FiliTheme.sidebarWidth
        current: folder.path
        onOpen: p => win.open(p)
        onDropped: (urls, dest, copy) => folder.dropItems(urls, dest, copy)
    }
    Rectangle {
        id: sideLine
        anchors { top: toolbarLine.bottom; bottom: statusbar.top; left: sidebar.right }
        width: 1
        color: FiliTheme.nuit4
    }

    FileList {
        id: files
        anchors { top: toolbarLine.bottom; bottom: statusbar.top; left: sideLine.right; right: parent.right }
        folder: folder
        windowActive: win.active
        onOpenRow: row => win.show(folder.activate(row))
        onGoUp: win.show(folder.up())
        onDropped: (urls, dest, copy) => folder.dropItems(urls, dest, copy)
        onContext: (x, y) => win.openMenu(files.mapToItem(null, x, y))
    }
    ContextMenu {
        id: menu
        onActivated: id => win.menuAction(id)
    }
    Connections {
        target: folder
        function onReloaded() {
            files.reloaded();
        }
    }

    // Barre d'état: nombre d'éléments et espace libre, ou l'erreur de la dernière action
    Rectangle {
        id: statusbar
        anchors.bottom: parent.bottom
        width: parent.width
        height: FiliTheme.statusHeight
        color: FiliTheme.nuit
        Rectangle {
            width: parent.width
            height: 1
            color: FiliTheme.nuit4
        }
        Text {
            anchors { fill: parent; leftMargin: FiliTheme.space3; rightMargin: FiliTheme.space3 }
            visible: !folder.busy
            verticalAlignment: Text.AlignVCenter
            horizontalAlignment: Text.AlignHCenter
            elide: Text.ElideMiddle
            text: folder.error || win.notice || folder.status
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeSmall
            color: folder.error || win.notice ? FiliTheme.cendre : FiliTheme.brume2
        }
        // Opération en cours: nom, avancement, annulation
        Row {
            anchors.centerIn: parent
            visible: folder.busy
            spacing: FiliTheme.space3
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: folder.job
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeSmall
                color: FiliTheme.brume2
            }
            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: 140
                height: 4
                radius: 2
                color: FiliTheme.nuit3
                Rectangle {
                    width: parent.width * Math.max(0.05, folder.progress)
                    height: parent.height
                    radius: 2
                    color: FiliTheme.voile
                }
            }
            TextButton {
                label: Fili.tr("Cancel")
                height: statusbar.height - 2
                onClicked: folder.cancel()
            }
        }
    }

    // Confirmation avant de vider la corbeille (suppression définitive): Entrée vide, Échap annule
    Rectangle {
        id: confirm
        anchors.fill: parent
        visible: false
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        focus: visible
        onVisibleChanged: if (visible) forceActiveFocus(); else files.focusList()
        Keys.onPressed: e => {
            if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) {
                folder.emptyTrash();
                visible = false;
            } else if (e.key === Qt.Key_Escape)
                visible = false;
            else
                return;
            e.accepted = true;
        }
        MouseArea {
            anchors.fill: parent
            onClicked: confirm.visible = false
        }
        Rectangle {
            anchors.centerIn: parent
            width: 340
            height: card.implicitHeight + 2 * FiliTheme.space4
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
            Column {
                id: card
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space4 }
                spacing: FiliTheme.space3
                Text {
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: Fili.tr("Empty the Trash?")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: FiliTheme.weightAccent
                    color: FiliTheme.brume
                }
                Text {
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: Fili.tr("Its items will be deleted permanently. This cannot be undone.")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
                Row {
                    anchors.right: parent.right
                    spacing: FiliTheme.space2
                    TextButton {
                        label: Fili.tr("Cancel")
                        onClicked: confirm.visible = false
                    }
                    TextButton {
                        label: Fili.tr("Empty Trash")
                        primary: true
                        onClicked: {
                            folder.emptyTrash();
                            confirm.visible = false;
                        }
                    }
                }
            }
        }
    }
}
