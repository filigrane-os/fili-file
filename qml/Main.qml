// Fenêtre de fili-file: barre d'outils (précédent, suivant, chemin), barre latérale, liste, barre d'état.
// Raccourcis: Alt+← / Alt+→ historique, Alt+↑ parent, Ctrl+L chemin, Ctrl+F recherche floue récursive, Ctrl+H fichiers cachés, F5 relire,
// Ctrl+C / Ctrl+X / Ctrl+V copier, couper, coller, F2 renommer, Ctrl+Maj+N nouveau dossier, Suppr ou Ctrl+Retour
// arrière corbeille, Ctrl+Z annuler, Ctrl+A tout sélectionner, Échap annule la copie en cours,
// Ctrl+1 / Ctrl+2 vue en liste / en grille, Ctrl+Maj+Suppr vide la corbeille, Ctrl+W ou Ctrl+Q fermer; boutons précédent/suivant de la souris.
// Mode sélecteur (fili-file --pick, lancé par le portail FileChooser): barre de validation en bas (nom en
// enregistrement, filtres, Annuler, Ouvrir/Enregistrer); les opérations sur les fichiers sont désactivées.
import QtQuick
import QtQuick.Controls.Basic
import Fili.Files

ApplicationWindow {
    id: win
    width: 900
    height: 560
    minimumWidth: 520
    minimumHeight: 300
    visible: true
    color: FiliTheme.nuit
    title: picking ? Fili.pickTitle() : folder.shown === "/" ? "/" : folder.shown.split("/").pop()

    // Mode sélecteur de fichiers
    readonly property string pickKind: Fili.pickKind()
    readonly property bool picking: pickKind !== ""
    readonly property var pickFilters: picking ? JSON.parse(Fili.pickFilters()) : []
    property int pickFilterIndex: Fili.pickFilter()
    // Chemins que validerait le bouton (vide: bouton grisé)
    function pickedPaths() {
        if (!picking)
            return [];
        if (pickKind === "save") {
            const n = nameInput.text.trim();
            return n !== "" && n.indexOf("/") < 0 ? [folder.path + "/" + n] : [];
        }
        const sel = Object.keys(files.selected);
        if (Fili.pickDirectory()) {
            const dirs = sel.filter(p => folder.pathIsDir(p));
            if (dirs.length !== sel.length || (!Fili.pickMultiple() && dirs.length > 1))
                return [];
            return dirs.length > 0 ? dirs : [folder.path];
        }
        if (sel.length === 0 || sel.some(p => folder.pathIsDir(p)) || (!Fili.pickMultiple() && sel.length > 1))
            return [];
        return sel;
    }
    readonly property var picked: pickedPaths()
    function pickAccept() {
        if (picked.length === 0)
            return;
        if (pickKind === "save" && Fili.pathExists(picked[0])) {
            const name = picked[0].split("/").pop();
            win.ask(Fili.tr("Replace “%1”?").replace("%1", name), Fili.tr("A file with this name already exists. Replacing it overwrites its content."), Fili.tr("Replace"), () => Fili.pickFinish(picked.join("\n"), pickFilterIndex));
            return;
        }
        Fili.pickFinish(picked.join("\n"), pickFilterIndex);
    }
    // Double-clic ou Entrée sur une ligne: un dossier s'ouvre; un fichier est choisi en mode sélecteur
    function openRow(row) {
        if (picking && !folder.pathIsDir(folder.pathAt(row))) {
            if (pickKind === "save")
                nameInput.text = folder.pathAt(row).split("/").pop();
            else
                files.select(row);
            pickAccept();
            return;
        }
        show(folder.activate(row));
    }
    Connections {
        target: files
        function onSelectedChanged() {
            if (win.pickKind !== "save")
                return;
            const sel = Object.keys(files.selected);
            if (sel.length === 1 && !folder.pathIsDir(sel[0]))
                nameInput.text = sel[0].split("/").pop();
        }
    }

    FolderModel { id: folder }
    // Icône de la corbeille (pleine ou vide): la barre latérale relit ses lieux quand l'état change
    property bool trashFull: Fili.trashFull()
    function checkTrash() {
        const full = Fili.trashFull();
        if (full !== trashFull) {
            trashFull = full;
            sidebar.reload();
        }
    }
    Connections {
        target: folder
        function onChanged() { win.checkTrash(); }
        function onReloaded() { win.checkTrash(); }
    }

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

    // Contexte pour l'assistant (fili-context): dossier et sélection, écrits à chaque changement (regroupés)
    Timer {
        id: contextTimer
        interval: 150
        onTriggered: Fili.publishContext(folder.path, Object.keys(files.selected).join("\n"), folder.results)
    }
    Connections {
        target: files
        function onSelectedChanged() { contextTimer.restart(); }
    }
    Connections {
        target: folder
        function onChanged() { contextTimer.restart(); }
    }
    // Commandes de l'assistant (fili-file ctl): aller à un endroit, afficher des résultats
    Connections {
        target: Fili
        function onCommand(verb, arg, body) {
            if (verb === "goto") {
                win.open(arg);
            } else if (verb === "show") {
                folder.showResults(arg, body);
                files.select(0);
                files.focusList();
            }
        }
    }
    Component.onDestruction: Fili.publishContext("", "", "")

    Component.onCompleted: {
        nameInput.text = Fili.pickName();
        show(folder.go(Fili.startPath()));
        files.focusList();
        if (Fili.startInfo())
            showInfo();
    }

    Shortcut { sequence: "Ctrl+["; onActivated: win.show(folder.back()) }
    Shortcut { sequence: "Ctrl+]"; onActivated: win.show(folder.forward()) }


    // Commandes de la barre de menus (exposée au menu global de la fili-bar par AT-SPI); chaque raccourci vit ici, une seule fois
    readonly property bool canAct: !win.typing && !win.picking
    Action { id: aNewFolder; text: Fili.tr("New Folder"); shortcut: "Ctrl+Shift+N"; enabled: win.canAct && !folder.inTrash; onTriggered: win.newFolder() }
    Action { id: aOpen; text: Fili.tr("Open"); enabled: win.canAct && files.targets().length === 1; onTriggered: win.show(folder.activate(files.currentIndex)) }
    Action { id: aInfo; text: Fili.tr("Get Info"); shortcut: "Ctrl+I"; enabled: win.canAct; onTriggered: win.showInfo() }
    Action { id: aTrash; text: Fili.tr("Move to Trash"); shortcut: "Delete"; enabled: win.canAct && !folder.inTrash; onTriggered: win.trashSelection() }
    Action { id: aTrashAlt; shortcut: "Ctrl+Backspace"; enabled: win.canAct; onTriggered: win.trashSelection() }
    Action { id: aEmpty; text: Fili.tr("Empty Trash"); shortcut: "Ctrl+Shift+Delete"; enabled: !win.typing && folder.inTrash; onTriggered: win.askEmpty() }
    Action { id: aClose; text: Fili.tr("Close"); shortcut: "Ctrl+W"; onTriggered: Qt.quit() }
    Action { id: aQuit; text: Fili.tr("Quit"); shortcut: "Ctrl+Q"; onTriggered: Qt.quit() }
    Action { id: aUndo; text: Fili.tr("Undo"); shortcut: "Ctrl+Z"; enabled: win.canAct; onTriggered: folder.undo() }
    Action { id: aCut; text: Fili.tr("Cut"); shortcut: "Ctrl+X"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.cut(files.targetsText()) }
    Action { id: aCopy; text: Fili.tr("Copy"); shortcut: "Ctrl+C"; enabled: win.canAct; onTriggered: folder.copy(files.targetsText()) }
    Action { id: aPaste; text: Fili.tr("Paste"); shortcut: "Ctrl+V"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.paste() }
    Action { id: aDuplicate; text: Fili.tr("Duplicate"); shortcut: "Ctrl+D"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.duplicate(files.targetsText()) }
    Action { id: aRename; text: Fili.tr("Rename"); shortcut: "F2"; enabled: win.canAct && !folder.inTrash; onTriggered: files.startRename() }
    Action { id: aSelectAll; text: Fili.tr("Select All"); shortcut: "Ctrl+A"; enabled: win.canAct; onTriggered: files.selectAll() }
    Action { id: aTerminal; text: Fili.tr("Open Terminal Here"); shortcut: "Ctrl+Alt+T"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.terminal(files.targets().length === 1 ? files.targets()[0] : "") }
    Action { id: aList; text: Fili.tr("View as List"); shortcut: "Ctrl+1"; checkable: true; checked: !files.tiles; onTriggered: files.setViewMode("list") }
    Action { id: aGrid; text: Fili.tr("View as Icons"); shortcut: "Ctrl+2"; checkable: true; checked: files.tiles; onTriggered: files.setViewMode("grid") }
    Action { id: aHidden; text: Fili.tr("Show Hidden Files"); shortcut: "Ctrl+H"; checkable: true; checked: folder.showHidden; onTriggered: win.show(folder.toggleHidden(files.currentIndex)) }
    Action { id: aRefresh; text: Fili.tr("Reload"); shortcut: "F5"; onTriggered: win.show(folder.refresh(files.currentIndex)) }
    Action { id: aRefreshAlt; shortcut: "Ctrl+R"; onTriggered: win.show(folder.refresh(files.currentIndex)) }
    Action { id: aBack; text: Fili.tr("Back"); shortcut: "Alt+Left"; enabled: folder.canBack; onTriggered: win.show(folder.back()) }
    Action { id: aForward; text: Fili.tr("Forward"); shortcut: "Alt+Right"; enabled: folder.canForward; onTriggered: win.show(folder.forward()) }
    Action { id: aUp; text: Fili.tr("Enclosing Folder"); shortcut: "Alt+Up"; onTriggered: win.show(folder.up()) }
    Action { id: aPath; text: Fili.tr("Go to Folder…"); shortcut: "Ctrl+L"; onTriggered: pathBar.edit() }
    Action { id: aSearch; text: Fili.tr("Search"); shortcut: "Ctrl+F"; onTriggered: searchInput.forceActiveFocus() }

    menuBar: MenuBar {
        id: bar
        implicitHeight: 24
        background: Rectangle { color: FiliTheme.nuit; Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: FiliTheme.nuit4 } }
        delegate: MenuBarItem {
            id: mbi
            implicitHeight: 24
            padding: FiliTheme.space2
            contentItem: Text {
                text: mbi.text
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: FiliTheme.brume
                verticalAlignment: Text.AlignVCenter
            }
            background: Rectangle { radius: FiliTheme.radiusControl; color: mbi.highlighted ? FiliTheme.nuit3 : "transparent" }
        }
        component FMenu: Menu {
            id: m
            implicitWidth: 240
            background: Rectangle { color: FiliTheme.nuit2; border.width: 1; border.color: FiliTheme.nuit4; radius: FiliTheme.radiusControl }
            delegate: MenuItem {
                id: it
                implicitHeight: 26
                contentItem: Item {
                    Text {
                        anchors { left: parent.left; verticalCenter: parent.verticalCenter }
                        text: (it.checked ? "✓ " : "") + it.text
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.size
                        color: it.enabled ? FiliTheme.brume : FiliTheme.ardoise
                    }
                    Text {
                        anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                        text: it.action ? it.action.shortcut : ""
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.sizeSmall
                        color: FiliTheme.brume2
                    }
                }
                arrow: Item {}
                indicator: Item {}
                background: Rectangle { radius: 4; color: it.highlighted ? FiliTheme.nuit3 : "transparent" }
            }
        }
        FMenu {
            title: Fili.tr("File")
            MenuItem { action: aNewFolder }
            MenuItem { action: aOpen }
            MenuItem { action: aInfo }
            MenuSeparator {}
            MenuItem { action: aTrash }
            MenuItem { action: aEmpty }
            MenuSeparator {}
            MenuItem { action: aClose }
            MenuItem { action: aQuit }
        }
        FMenu {
            title: Fili.tr("Edit")
            MenuItem { action: aUndo }
            MenuSeparator {}
            MenuItem { action: aCut }
            MenuItem { action: aCopy }
            MenuItem { action: aPaste }
            MenuItem { action: aDuplicate }
            MenuItem { action: aRename }
            MenuSeparator {}
            MenuItem { action: aSelectAll }
        }
        FMenu {
            title: Fili.tr("View")
            MenuItem { action: aList }
            MenuItem { action: aGrid }
            MenuItem { action: aHidden }
            MenuSeparator {}
            MenuItem { action: aRefresh }
        }
        FMenu {
            title: Fili.tr("Go")
            MenuItem { action: aBack }
            MenuItem { action: aForward }
            MenuItem { action: aUp }
            MenuItem { action: aPath }
            MenuItem { action: aSearch }
            MenuSeparator {}
            MenuItem { action: aTerminal }
        }
    }

    // Pas de raccourcis de fichiers pendant une saisie (renommage, chemin) ni sous la confirmation
    readonly property bool typing: menu.visible || nameInput.activeFocus || files.editing || pathBar.editing || searchInput.activeFocus || confirm.visible || infoCard.visible
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
        if (picking)
            return;
        const t = files.targets().filter(path => files.selected[path]);
        folder.refreshPaste();
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
            list.push({ id: "paste", label: Fili.tr("Paste"), hint: "Ctrl+V", enabled: folder.canPaste && folder.results === "" });
        if (t.length > 0) {
            list.push({ separator: true });
            list.push({ id: "copyfilepath", label: Fili.tr("Copy Full Path") });
            if (one && folder.pathIsDir(t[0]) && !Fili.isFavorite(t[0])) {
                menuPath = t[0];
                list.push({ id: "addfav", label: Fili.tr("Add to Favorites") });
            }
            if (!trash)
                list.push({ id: "rename", label: Fili.tr("Rename"), hint: "F2", enabled: one });
            if (!trash)
                list.push({ id: "duplicate", label: Fili.tr("Duplicate"), hint: "Ctrl+D" });
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
            list.push({ separator: true });
            list.push({ id: "info", label: Fili.tr("Get Info"), hint: "Ctrl+I" });
        }
        if (!trash && (t.length === 0 || (one && folder.pathIsDir(t[0])))) {
            menuPath = t.length === 1 ? t[0] : "";
            list.push({ id: "terminal", label: Fili.tr("Open Terminal Here") });
        }
        // Sur le fond du dossier: affichage (liste ou grille, fichiers cachés, tri)
        if (t.length === 0) {
            list.push({ separator: true });
            list.push({ id: "view:list", label: Fili.tr("View as List"), hint: "Ctrl+1", checked: !files.tiles });
            list.push({ id: "view:grid", label: Fili.tr("View as Icons"), hint: "Ctrl+2", checked: files.tiles });
            list.push({ id: "hidden", label: Fili.tr("Show Hidden Files"), hint: "Ctrl+H", checked: folder.showHidden });
            list.push({
                id: "sort", label: Fili.tr("Sort By"),
                children: files.allColumns.map(c => ({ id: "sort:" + c.sort, label: c.label, checked: folder.sortKey === c.sort }))
                    .concat([{ id: "sort:0", label: Fili.tr("Name"), checked: folder.sortKey === 0 }]).sort((a, b) => parseInt(a.id.slice(5)) - parseInt(b.id.slice(5)))
            });
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
    // Colonnes de la liste: clic droit sur un en-tête (le nom reste toujours)
    function openColumnsMenu(p) {
        const list = [{ id: "", label: Fili.tr("Name"), checked: true, enabled: false }];
        files.allColumns.forEach(c => list.push({ id: "col:" + c.key, label: c.label, checked: files.enabledKeys.indexOf(c.key) >= 0 }));
        menu.open(p.x, p.y, list);
    }
    // Menu d'un chemin (segment du fil d'Ariane ou lieu de la barre latérale)
    property string menuPath: ""
    function openPathMenu(path, x, y, crumb, removable) {
        menuPath = path;
        const list = [{ id: "gopath", label: Fili.tr("Open") }, { separator: true }];
        if (removable)
            list.push({ id: "remfav", label: Fili.tr("Remove from Favorites") });
        else if (crumb && !Fili.isFavorite(path))
            list.push({ id: "addfav", label: Fili.tr("Add to Favorites") });
        list.push({ id: "copypath", label: Fili.tr("Copy Full Path") });
        menu.open(x, y, list);
    }
    function menuAction(id) {
        const t = files.targetsText();
        if (id.startsWith("view:"))
            files.setViewMode(id.slice(5));
        else if (id === "hidden")
            win.show(folder.toggleHidden(files.currentIndex));
        else if (id.startsWith("sort:"))
            win.show(folder.sortBy(parseInt(id.slice(5)), files.currentIndex));
        else if (id.startsWith("col:")) {
            const key = id.slice(4);
            const on = files.enabledKeys.indexOf(key) >= 0;
            files.setColumns(files.allColumns.map(c => c.key).filter(k => k === key ? !on : files.enabledKeys.indexOf(k) >= 0));
        } else if (id.startsWith("filter:")) {
            pickFilterIndex = parseInt(id.slice(7));
            folder.setPickFilter(pickFilterIndex);
            files.reloaded();
        } else if (id === "gopath")
            win.open(menuPath);
        else if (id === "copypath")
            Fili.copyText(menuPath);
        else if (id === "copyfilepath")
            Fili.copyText(files.targets().join("\n"));
        else if (id === "addfav")
            Fili.addFavorite(menuPath);
        else if (id === "remfav")
            Fili.removeFavorite(menuPath);
        else if (id === "open")
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
        else if (id === "duplicate")
            folder.duplicate(t);
        else if (id === "info")
            win.showInfo();
        else if (id === "terminal")
            folder.terminal(menuPath);
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
            win.askEmpty();
        files.focusList();
    }
    Shortcut { sequence: "Escape"; enabled: !win.typing && folder.busy; onActivated: folder.cancel() }
    Shortcut { sequence: "Escape"; enabled: win.picking && !win.typing && !folder.busy; onActivated: Fili.pickCancel() }
    Shortcut { sequence: "Ctrl+A"; enabled: win.picking && !win.typing && Fili.pickMultiple(); onActivated: files.selectAll() }

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
            x: FiliTheme.margin
            anchors.verticalCenter: nav.verticalCenter
            width: nav.width + 2 * FiliTheme.space1 + 2
            height: 30
            radius: 8
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
        }
        Row {
            id: nav
            x: FiliTheme.margin + FiliTheme.space1 + 1
            anchors.verticalCenter: parent.verticalCenter
            spacing: FiliTheme.space1
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
        // Recherche floue (Ctrl+F) dans le dossier affiché et tous ses sous-dossiers, meilleurs résultats d'abord, Échap efface
        Rectangle {
            id: search
            anchors { right: parent.right; rightMargin: FiliTheme.margin; verticalCenter: parent.verticalCenter }
            width: 180
            height: 26
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
        ViewSwitch {
            id: viewSwitch
            anchors { right: search.left; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
            mode: files.viewMode
            onChose: m => files.setViewMode(m)
        }
        Row {
            id: trashTools
            anchors { right: viewSwitch.left; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
            spacing: 2
            visible: folder.inTrash
            TextButton {
                label: Fili.tr("Put back")
                onClicked: win.putBack()
            }
            TextButton {
                label: Fili.tr("Empty Trash")
                onClicked: win.askEmpty()
            }
        }
        // Titre: nom du dossier affiché (le chemin complet est dans la barre d'état)
        Text {
            anchors { left: nav.right; leftMargin: FiliTheme.space5; right: trashTools.visible ? trashTools.left : viewSwitch.left; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
            text: win.title
            elide: Text.ElideMiddle
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.size
            font.weight: FiliTheme.weightAccent
            color: FiliTheme.brume
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
        anchors { top: toolbarLine.bottom; bottom: pickbar.top; left: parent.left }
        width: FiliTheme.sidebarWidth
        current: folder.shown
        onOpen: p => win.open(p)
        onDropped: (urls, dest, copy) => folder.dropItems(urls, dest, copy)
        onContext: (path, removable, x, y) => win.openPathMenu(path, x, y, false, removable)
    }
    Rectangle {
        id: sideLine
        anchors { top: toolbarLine.bottom; bottom: pickbar.top; left: sidebar.right }
        width: 1
        color: FiliTheme.nuit4
    }

    // Résultats affichés à la place du dossier (recherche de l'assistant)
    Rectangle {
        id: resultsBar
        anchors { top: toolbarLine.bottom; left: sideLine.right; right: parent.right }
        height: folder.results !== "" ? 28 : 0
        visible: folder.results !== ""
        color: FiliTheme.nuit2
        Text {
            anchors { left: parent.left; leftMargin: FiliTheme.margin; right: closeResults.left; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
            elide: Text.ElideRight
            text: Fili.tr("Results") + ": " + folder.results
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeList
            color: FiliTheme.brume
        }
        TextButton {
            id: closeResults
            anchors { right: parent.right; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
            label: Fili.tr("Close results")
            onClicked: {
                folder.closeResults();
                files.focusList();
            }
        }
    }
    FileList {
        id: files
        anchors { top: resultsBar.bottom; bottom: pickbar.top; left: sideLine.right; right: parent.right }
        folder: folder
        windowActive: win.active
        onOpenRow: row => win.openRow(row)
        onGoUp: win.show(folder.up())
        onDropped: (urls, dest, copy) => folder.dropItems(urls, dest, copy)
        onContext: (x, y) => win.openMenu(files.mapToItem(null, x, y))
        onHeaderContext: (x, y) => win.openColumnsMenu(files.mapToItem(null, x, y))
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

    // Mode sélecteur: nom (enregistrer), filtre, Annuler et validation
    Rectangle {
        id: pickbar
        anchors.bottom: statusbar.top
        width: parent.width
        height: win.picking ? 48 : 0
        visible: win.picking
        color: FiliTheme.nuit
        Rectangle {
            width: parent.width
            height: 1
            color: FiliTheme.nuit4
        }
        Rectangle {
            visible: win.pickKind === "save"
            anchors { left: parent.left; leftMargin: FiliTheme.margin; verticalCenter: parent.verticalCenter }
            width: 300
            height: 26
            radius: FiliTheme.radiusControl
            color: FiliTheme.nuit2
            border.width: 1
            border.color: nameInput.activeFocus ? FiliTheme.voile : FiliTheme.nuit4
            TextInput {
                id: nameInput
                anchors { fill: parent; leftMargin: FiliTheme.space2; rightMargin: FiliTheme.space2 }
                verticalAlignment: TextInput.AlignVCenter
                clip: true
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: FiliTheme.brume
                selectionColor: FiliTheme.nuit4
                selectedTextColor: FiliTheme.brume
                onAccepted: win.pickAccept()
                Keys.onEscapePressed: Fili.pickCancel()
                Component.onCompleted: {
                    if (win.pickKind === "save")
                        forceActiveFocus();
                }
            }
        }
        TextButton {
            visible: win.pickFilters.length > 0
            anchors { left: parent.left; leftMargin: win.pickKind === "save" ? 300 + FiliTheme.margin : FiliTheme.margin - FiliTheme.space3; verticalCenter: parent.verticalCenter }
            height: 26
            label: (win.pickFilters[win.pickFilterIndex] || "") + "  ▾"
            onClicked: menu.open(x + FiliTheme.space2, pickbar.y - win.pickFilters.length * menu.itemHeight - FiliTheme.space4, win.pickFilters.map((n, i) => ({ id: "filter:" + i, label: n })))
        }
        Row {
            anchors { right: parent.right; rightMargin: FiliTheme.margin; verticalCenter: parent.verticalCenter }
            spacing: FiliTheme.space2
            TextButton {
                height: 26
                label: Fili.tr("Cancel")
                onClicked: Fili.pickCancel()
            }
            TextButton {
                height: 26
                primary: true
                opacity: win.picked.length > 0 ? 1 : 0.4
                label: Fili.pickAccept() || (win.pickKind === "save" ? Fili.tr("Save") : Fili.pickDirectory() ? Fili.tr("Choose") : Fili.tr("Open"))
                onClicked: win.pickAccept()
            }
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
        // Fil d'Ariane à gauche (clic: y aller, clic droit: menu), information sur la sélection à droite; une erreur,
        // un message ou une opération en cours prennent toute la barre
        readonly property bool message: folder.busy || folder.error !== "" || win.notice !== ""
        PathBar {
            id: pathBar
            anchors { left: parent.left; leftMargin: FiliTheme.margin - FiliTheme.space2; top: parent.top; topMargin: 1; bottom: parent.bottom }
            width: parent.width * 0.6
            visible: !statusbar.message || editing
            fontSize: FiliTheme.sizeSmall
            path: folder.shown
            onOpen: p => win.open(p)
            onContext: (p, x, y) => win.openPathMenu(p, x, y, true)
            onDone: {
                editing = false;
                files.focusList();
            }
        }
        Text {
            anchors { left: pathBar.right; leftMargin: FiliTheme.space3; right: parent.right; rightMargin: FiliTheme.margin; top: parent.top; topMargin: 1; bottom: parent.bottom }
            visible: !statusbar.message && !pathBar.editing
            verticalAlignment: Text.AlignVCenter
            horizontalAlignment: Text.AlignRight
            elide: Text.ElideMiddle
            text: files.info || folder.status
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeSmall
            color: FiliTheme.brume2
        }
        Text {
            anchors { fill: parent; leftMargin: FiliTheme.space3; rightMargin: FiliTheme.space3 }
            visible: statusbar.message && !folder.busy
            verticalAlignment: Text.AlignVCenter
            horizontalAlignment: Text.AlignHCenter
            elide: Text.ElideMiddle
            text: folder.error || win.notice
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeSmall
            color: FiliTheme.cendre
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

    // Confirmation (vider la corbeille, remplacer un fichier): Entrée valide, Échap annule
    function ask(heading, body, label, action) {
        confirm.heading = heading;
        confirm.body = body;
        confirm.label = label;
        confirm.action = action;
        confirm.visible = true;
    }
    function askEmpty() {
        ask(Fili.tr("Empty the Trash?"), Fili.tr("Its items will be deleted permanently. This cannot be undone."), Fili.tr("Empty Trash"), () => folder.emptyTrash());
    }
    Rectangle {
        id: confirm
        property string heading
        property string body
        property string label
        property var action: () => {}
        anchors.fill: parent
        visible: false
        z: 200
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        focus: visible
        onVisibleChanged: if (visible) forceActiveFocus(); else files.focusList()
        Keys.onPressed: e => {
            if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) {
                visible = false;
                confirm.action();
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
                    text: confirm.heading
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: FiliTheme.weightAccent
                    color: FiliTheme.brume
                }
                Text {
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: confirm.body
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
                        label: confirm.label
                        primary: true
                        onClicked: {
                            confirm.visible = false;
                            confirm.action();
                        }
                    }
                }
            }
        }
    }

    // Informations (Ctrl+I): éléments sélectionnés, à défaut le dossier affiché; la taille des dossiers arrive après
    function showInfo() {
        const t = files.targets().filter(path => files.selected[path]);
        const list = t.length > 0 ? t : [folder.path];
        infoCard.rows = JSON.parse(folder.info(list.join("\n")));
        infoCard.visible = true;
        folder.startInfoSize(list.join("\n"));
    }
    Connections {
        target: folder
        function onInfoSize(text) {
            const r = infoCard.rows.slice();
            const i = r.findIndex(x => x[0] === Fili.tr("Size"));
            if (i >= 0 && infoCard.visible) {
                r[i] = [r[i][0], text];
                infoCard.rows = r;
            }
        }
    }
    Rectangle {
        id: infoCard
        property var rows: []
        anchors.fill: parent
        visible: false
        z: 200
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        focus: visible
        onVisibleChanged: if (visible) forceActiveFocus(); else files.focusList()
        Keys.onPressed: e => {
            if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter || e.key === Qt.Key_Escape || (e.key === Qt.Key_I && (e.modifiers & Qt.ControlModifier)))
                visible = false;
            else
                return;
            e.accepted = true;
        }
        MouseArea {
            anchors.fill: parent
            onClicked: infoCard.visible = false
        }
        Rectangle {
            anchors.centerIn: parent
            width: 420
            height: grid.implicitHeight + okRow.height + 3 * FiliTheme.space4
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
            Grid {
                id: grid
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space4 }
                columns: 2
                columnSpacing: FiliTheme.space3
                rowSpacing: FiliTheme.space2
                Repeater {
                    model: infoCard.rows.length * 2
                    Text {
                        readonly property bool label: index % 2 === 0
                        readonly property string content: infoCard.rows[Math.floor(index / 2)][label ? 0 : 1]
                        width: label ? 110 : grid.width - 110 - FiliTheme.space3
                        horizontalAlignment: label ? Text.AlignRight : Text.AlignLeft
                        wrapMode: Text.WrapAnywhere
                        text: content
                        textFormat: Text.PlainText
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.sizeList
                        font.weight: label ? FiliTheme.weightAccent : Font.Normal
                        color: label ? FiliTheme.brume2 : FiliTheme.brume
                    }
                }
            }
            Row {
                id: okRow
                anchors { right: parent.right; bottom: parent.bottom; margins: FiliTheme.space4 }
                TextButton {
                    label: Fili.tr("OK")
                    primary: true
                    onClicked: infoCard.visible = false
                }
            }
        }
    }
}
