// Fenêtre de fili-file: barre d'outils (précédent, suivant, chemin), barre latérale, liste, barre d'état.
// Raccourcis: Alt+← / Alt+→ historique, Alt+↑ parent, Ctrl+L chemin (propositions façon fzf: Haut/Bas, Tab, Entrée), Ctrl+F ou / recherche floue récursive, Ctrl+H fichiers cachés, Ctrl+Maj+P panneau d'aperçu, F5 relire,
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
    title: picking ? Fili.pickTitle() : win.searchLabel !== "" ? win.searchLabel : folder.shown === "/" ? "/" : folder.shown.split("/").pop()

    // Nom de la recherche en cours (saisie ou résultats de l'assistant), vide hors recherche
    readonly property string searchLabel: folder.filter.trim() !== "" ? Fili.tr("Search “%1”").replace("%1", folder.filter.trim()) : folder.results !== "" && folder.results !== "…" ? folder.results : folder.results !== "" ? Fili.tr("Search results") : ""

    // Mode sélecteur de fichiers
    readonly property string pickKind: Fili.pickKind()
    readonly property bool picking: pickKind !== ""
    // Entrée 0: tous les fichiers (défaut); entrée i: filtre i - 1 de l'application
    readonly property var pickFilters: picking ? [Fili.tr("All files")].concat(JSON.parse(Fili.pickFilters())) : []
    property int pickFilterIndex: 0
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
            win.ask(Fili.tr("Replace “%1”?").replace("%1", name), Fili.tr("A file with this name already exists. Replacing it overwrites its content."), Fili.tr("Replace"), () => Fili.pickFinish(picked.join("\n"), pickFilterIndex - 1));
            return;
        }
        Fili.pickFinish(picked.join("\n"), pickFilterIndex - 1);
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
            } else if (verb === "action") {
                win.runAction(arg);
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
    Action { id: aAbout; text: Fili.tr("About Casier"); onTriggered: win.showAbout() }
    // Ctrl+W ferme l'onglet, et la fenêtre quand il n'y en a plus qu'un
    Action { id: aClose; text: Fili.tr("Close"); shortcut: "Ctrl+W"; onTriggered: win.tabs.length > 1 ? win.closeTab(win.tabIndex) : Qt.quit() }
    Action { id: aNewTab; text: Fili.tr("New Tab"); shortcut: "Ctrl+T"; enabled: !win.picking; onTriggered: win.newTab() }
    Action { id: aNextTab; text: Fili.tr("Show Next Tab"); shortcut: "Ctrl+Tab"; enabled: win.tabs.length > 1; onTriggered: win.switchTab((win.tabIndex + 1) % win.tabs.length) }
    Action { id: aPrevTab; text: Fili.tr("Show Previous Tab"); shortcut: "Ctrl+Shift+Tab"; enabled: win.tabs.length > 1; onTriggered: win.switchTab((win.tabIndex + win.tabs.length - 1) % win.tabs.length) }
    Action { id: aQuit; text: Fili.tr("Quit"); shortcut: "Ctrl+Q"; onTriggered: Qt.quit() }
    Action { id: aUndo; text: Fili.tr("Undo"); shortcut: "Ctrl+Z"; enabled: win.canAct; onTriggered: folder.undo() }
    Action { id: aCut; text: Fili.tr("Cut"); shortcut: "Ctrl+X"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.cut(files.targetsText()) }
    Action { id: aCopy; text: Fili.tr("Copy"); shortcut: "Ctrl+C"; enabled: win.canAct; onTriggered: folder.copy(files.targetsText()) }
    Action { id: aPaste; text: Fili.tr("Paste"); shortcut: "Ctrl+V"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.paste() }
    Action { id: aDuplicate; text: Fili.tr("Duplicate"); shortcut: "Ctrl+D"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.duplicate(files.targetsText()) }
    // F2: renommer sur place; avec plusieurs éléments sélectionnés, renommer en lot
    Action { id: aRename; text: Fili.tr("Rename"); shortcut: "F2"; enabled: win.canAct && !folder.inTrash; onTriggered: Object.keys(files.selected).length > 1 ? bulkBox.open() : files.startRename() }
    Action { id: aBulk; text: Fili.tr("Rename Items…"); enabled: win.canAct && !folder.inTrash && Object.keys(files.selected).length > 1; onTriggered: bulkBox.open() }
    Action { id: aSelectAll; text: Fili.tr("Select All"); shortcut: "Ctrl+A"; enabled: win.canAct; onTriggered: files.selectAll() }
    Action { id: aInvert; text: Fili.tr("Invert Selection"); shortcut: "Ctrl+Shift+I"; enabled: win.canAct; onTriggered: files.invertSelection() }
    Action { id: aMatch; text: Fili.tr("Select Matching…"); shortcut: "Ctrl+Shift+M"; enabled: win.canAct && !win.picking; onTriggered: matchBox.open() }
    Action { id: aSameKind; text: Fili.tr("Select Same Kind"); enabled: win.canAct && files.targets().length > 0; onTriggered: files.selectPaths(folder.sameKind(files.targetsText()).split("\n")) }
    Action { id: aTerminal; text: Fili.tr("Open Terminal Here"); shortcut: "Ctrl+Alt+T"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.terminal(files.targets().length === 1 ? files.targets()[0] : "") }
    Action { id: aList; text: Fili.tr("View as List"); shortcut: "Ctrl+1"; checkable: true; checked: !files.tiles; onTriggered: files.setViewMode("list") }
    Action { id: aGrid; text: Fili.tr("View as Icons"); shortcut: "Ctrl+2"; checkable: true; checked: files.tiles; onTriggered: files.setViewMode("grid") }
    Action { id: aCopyPath; text: Fili.tr("Copy Full Path"); shortcut: "Ctrl+Shift+C"; enabled: win.canAct && files.targets().length > 0; onTriggered: win.copyAs("path") }
    Action { id: aCopyName; text: Fili.tr("Copy Name"); shortcut: "Ctrl+Alt+C"; enabled: win.canAct && files.targets().length > 0; onTriggered: win.copyAs("name") }
    Action { id: aCopyStem; text: Fili.tr("Copy Name Without Extension"); enabled: win.canAct && files.targets().length > 0; onTriggered: win.copyAs("stem") }
    Action { id: aCopyParent; text: Fili.tr("Copy Parent Folder"); enabled: win.canAct && files.targets().length > 0; onTriggered: win.copyAs("parent") }
    Action { id: aPreview; text: Fili.tr("Show Preview"); shortcut: "Ctrl+Shift+P"; checkable: true; checked: win.previewOn; enabled: !win.picking; onTriggered: win.togglePreview() }
    Action { id: aHidden; text: Fili.tr("Show Hidden Files"); shortcut: "Ctrl+H"; checkable: true; checked: folder.showHidden; onTriggered: win.show(folder.toggleHidden(files.currentIndex)) }
    Action { id: aRefresh; text: Fili.tr("Reload"); shortcut: "F5"; onTriggered: win.show(folder.refresh(files.currentIndex)) }
    Action { id: aRefreshAlt; shortcut: "Ctrl+R"; onTriggered: win.show(folder.refresh(files.currentIndex)) }
    Action { id: aBack; text: Fili.tr("Back"); shortcut: "Alt+Left"; enabled: folder.canBack; onTriggered: win.show(folder.back()) }
    Action { id: aForward; text: Fili.tr("Forward"); shortcut: "Alt+Right"; enabled: folder.canForward; onTriggered: win.show(folder.forward()) }
    Action { id: aUp; text: Fili.tr("Enclosing Folder"); shortcut: "Alt+Up"; onTriggered: win.show(folder.up()) }
    Action { id: aPath; text: Fili.tr("Go to Folder…"); shortcut: "Ctrl+L"; onTriggered: pathBar.edit() }
    Action { id: aConnect; text: Fili.tr("Connect to Server…"); shortcut: "Ctrl+K"; onTriggered: connectBox.open() }
    // « / » comme Ctrl+F (façon vim, yazi); sans libellé: la barre de menus ne montre que Ctrl+F
    Action { id: aSearchSlash; shortcut: "/"; enabled: win.canAct; onTriggered: searchInput.forceActiveFocus() }
    Action { id: aSearch; text: Fili.tr("Search"); shortcut: "Ctrl+F"; onTriggered: searchInput.forceActiveFocus() }

    // `fili-file ctl action NOM`: la fili-bar affiche ces commandes dans son menu (dotfiles/config/filigrane/appmenu/fili-file.json,
    // test tests/fili-shell/check-fili-file-menu.py: mêmes noms, mêmes raccourcis). Une commande grisée ne fait rien.
    readonly property var menuActions: ({
        "new-folder": aNewFolder, "open": aOpen, "info": aInfo, "trash": aTrash, "empty-trash": aEmpty, "close": aClose, "new-tab": aNewTab, "next-tab": aNextTab, "prev-tab": aPrevTab, "about": aAbout,
        "undo": aUndo, "cut": aCut, "copy": aCopy, "paste": aPaste, "duplicate": aDuplicate, "rename": aRename, "bulk-rename": aBulk,
        "select-all": aSelectAll, "invert-selection": aInvert, "select-matching": aMatch, "select-same-kind": aSameKind, "terminal": aTerminal, "view-list": aList, "view-icons": aGrid, "toggle-hidden": aHidden, "toggle-preview": aPreview, "copy-path": aCopyPath, "copy-name": aCopyName, "copy-stem": aCopyStem, "copy-parent": aCopyParent,
        "reload": aRefresh, "back": aBack, "forward": aForward, "up": aUp, "path": aPath, "connect-server": aConnect, "search": aSearch
    })
    // Copie dans le presse-papiers (texte) le chemin, le nom, le nom sans extension ou le dossier parent des éléments
    // sélectionnés, un par ligne
    function copyAs(kind) {
        const name = p => p.split("/").pop();
        const out = files.targets().map(p => {
            if (kind === "name")
                return name(p);
            if (kind === "stem") {
                const n = name(p), i = n.lastIndexOf(".");
                return i > 0 && !folder.pathIsDir(p) ? n.slice(0, i) : n;
            }
            if (kind === "parent")
                return p.slice(0, p.lastIndexOf("/")) || "/";
            return p;
        });
        Fili.copyText(out.join("\n"));
    }
    // Onglets: chaque onglet garde son dossier (l'historique est celui de la fenêtre). La barre d'onglets n'apparaît
    // qu'à partir de deux onglets.
    property var tabs: [{ path: "" }]
    property int tabIndex: 0
    function newTab() {
        const t = tabs.slice();
        t[tabIndex] = { path: folder.shown };
        t.splice(tabIndex + 1, 0, { path: folder.shown });
        tabs = t;
        tabIndex = tabIndex + 1;
        files.focusList();
    }
    function switchTab(i) {
        if (i === tabIndex || i < 0 || i >= tabs.length)
            return;
        const t = tabs.slice();
        t[tabIndex] = { path: folder.shown };
        tabs = t;
        tabIndex = i;
        open(t[i].path);
    }
    function closeTab(i) {
        if (tabs.length < 2)
            return;
        const t = tabs.slice();
        t[tabIndex] = { path: folder.shown };
        t.splice(i, 1);
        const next = Math.min(i === tabIndex ? i : (i < tabIndex ? tabIndex - 1 : tabIndex), t.length - 1);
        tabs = t;
        const moved = i === tabIndex;
        tabIndex = next;
        if (moved)
            open(t[next].path);
        files.focusList();
    }
    // Panneau d'aperçu (choix gardé dans l'état de l'utilisateur)
    property bool previewOn: Fili.previewShown()
    function togglePreview() {
        previewOn = !previewOn;
        Fili.setPreviewShown(previewOn);
    }
    function runAction(name) {
        const a = menuActions[name];
        if (a && a.enabled)
            a.trigger();
    }

    // Pas de barre de menus dans la fenêtre: le menu vit dans la fili-bar (repli appmenu/fili-file.json, commandes ci-dessus)

    // Pas de raccourcis de fichiers pendant une saisie (renommage, chemin) ni sous la confirmation
    readonly property bool typing: menu.visible || nameInput.activeFocus || files.editing || pathBar.editing || searchInput.activeFocus || confirm.visible || infoCard.visible || matchBox.visible || bulkBox.visible || conflictBox.visible
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
            list.push({
                id: "copyas", label: Fili.tr("Copy Path As"),
                children: [
                    { id: "copyas:path", label: Fili.tr("Copy Full Path"), hint: "Ctrl+Shift+C" },
                    { id: "copyas:name", label: Fili.tr("Copy Name"), hint: "Ctrl+Alt+C" },
                    { id: "copyas:stem", label: Fili.tr("Copy Name Without Extension") },
                    { id: "copyas:parent", label: Fili.tr("Copy Parent Folder") }
                ]
            });
            if (one && folder.pathIsDir(t[0]) && !Fili.isFavorite(t[0])) {
                menuPath = t[0];
                list.push({ id: "addfav", label: Fili.tr("Add to Favorites") });
            }
            if (!trash)
                list.push({ id: "rename", label: one ? Fili.tr("Rename") : Fili.tr("Rename Items…"), hint: "F2" });
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
            folder.setPickFilter(pickFilterIndex - 1);
            files.reloaded();
        } else if (id === "gopath")
            win.open(menuPath);
        else if (id === "copypath")
            Fili.copyText(menuPath);
        else if (id.startsWith("copyas:"))
            win.copyAs(id.slice(7));
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
            aRename.trigger();
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
        PreviewButton {
            id: previewButton
            visible: !win.picking
            anchors { right: search.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
            on: win.previewOn
            onClicked: win.togglePreview()
        }
        ViewSwitch {
            id: viewSwitch
            anchors { right: win.picking ? search.left : previewButton.left; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
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

    // Onglets (à partir de deux): nom du dossier, × au survol; clic du milieu ferme
    Rectangle {
        id: tabBar
        anchors { top: toolbarLine.bottom; left: sideLine.right; right: previewLine.left }
        height: visible ? 30 : 0
        visible: win.tabs.length > 1 && !win.picking
        color: FiliTheme.nuit
        Row {
            anchors { fill: parent; leftMargin: FiliTheme.space2; topMargin: FiliTheme.space1; bottomMargin: FiliTheme.space1 }
            spacing: FiliTheme.space1
            Repeater {
                model: win.tabs
                Rectangle {
                    required property var modelData
                    required property int index
                    readonly property bool current: index === win.tabIndex
                    readonly property string path: current ? folder.shown : modelData.path
                    width: Math.max(80, Math.min(180, (tabBar.width - 2 * FiliTheme.space2) / win.tabs.length - FiliTheme.space1))
                    height: parent.height
                    radius: FiliTheme.radiusControl
                    color: current ? FiliTheme.nuit3 : tabArea.containsMouse ? FiliTheme.nuit2 : "transparent"
                    Text {
                        anchors { left: parent.left; leftMargin: FiliTheme.space3; right: closeX.left; rightMargin: FiliTheme.space1; verticalCenter: parent.verticalCenter }
                        text: path === "/" ? "/" : path === Fili.home() ? Fili.home().split("/").pop() : path.split("/").pop()
                        elide: Text.ElideRight
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.sizeList
                        font.weight: current ? FiliTheme.weightAccent : Font.Normal
                        color: current ? FiliTheme.brume : FiliTheme.brume2
                    }
                    MouseArea {
                        id: tabArea
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.LeftButton | Qt.MiddleButton
                        onClicked: e => e.button === Qt.MiddleButton ? win.closeTab(index) : win.switchTab(index)
                    }
                    Text {
                        id: closeX
                        visible: tabArea.containsMouse || xHit.containsMouse || current
                        anchors { right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                        text: "×"
                        font.pixelSize: 16
                        color: xHit.containsMouse ? FiliTheme.cendre : FiliTheme.ardoise
                        MouseArea {
                            id: xHit
                            anchors { fill: parent; margins: -5 }
                            hoverEnabled: true
                            onClicked: win.closeTab(index)
                        }
                    }
                }
            }
        }
    }
    // Résultats affichés à la place du dossier (recherche de l'assistant)
    Rectangle {
        id: resultsBar
        anchors { top: tabBar.bottom; left: sideLine.right; right: parent.right }
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
        anchors { top: resultsBar.bottom; bottom: pickbar.top; left: sideLine.right; right: previewLine.left }
        folder: folder
        windowActive: win.active
        onOpenRow: row => win.openRow(row)
        onGoUp: win.show(folder.up())
        onDropped: (urls, dest, copy) => folder.dropItems(urls, dest, copy)
        onContext: (x, y) => win.openMenu(files.mapToItem(null, x, y))
        onHeaderContext: (x, y) => win.openColumnsMenu(files.mapToItem(null, x, y))
    }
    Rectangle {
        id: previewLine
        anchors { top: toolbarLine.bottom; bottom: pickbar.top; right: preview.left }
        width: preview.visible ? 1 : 0
        color: FiliTheme.nuit4
    }
    Preview {
        id: preview
        // Largeur 300 au plus; la liste garde au moins 320 (en dessous de 160 le panneau disparaît)
        readonly property real room: win.width - FiliTheme.sidebarWidth - 1 - 320
        visible: win.previewOn && !win.picking && room >= 160
        anchors { top: toolbarLine.bottom; bottom: pickbar.top; right: parent.right }
        width: visible ? Math.min(300, room) : 0
        path: {
            const keys = Object.keys(files.selected);
            return keys.length === 1 ? keys[0] : keys.length === 0 && files.currentIndex >= 0 ? folder.pathAt(files.currentIndex) : "";
        }
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
            visible: win.pickFilters.length > 1
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
        // Au-dessus de la liste pendant la saisie d'un chemin: ses propositions débordent vers le haut
        z: pathBar.editing || tasksList.visible ? 150 : 0
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
            visible: (!statusbar.message && win.searchLabel === "") || editing
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
            anchors { left: parent.left; leftMargin: FiliTheme.margin; right: parent.horizontalCenter; top: parent.top; topMargin: 1; bottom: parent.bottom }
            visible: !statusbar.message && win.searchLabel !== "" && !pathBar.editing
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
            text: win.searchLabel
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeSmall
            font.bold: true
            color: FiliTheme.brume
        }
        Text {
            anchors { left: pathBar.right; leftMargin: FiliTheme.space3; right: parent.right; rightMargin: FiliTheme.margin; top: parent.top; topMargin: 1; bottom: parent.bottom }
            visible: !statusbar.message && !pathBar.editing
            verticalAlignment: Text.AlignVCenter
            horizontalAlignment: Text.AlignRight
            elide: Text.ElideMiddle
            text: win.searchLabel !== "" ? folder.status : files.info || folder.status
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
            // Nom de l'opération en cours, avec le nombre de celles en attente; un clic ouvre la liste des tâches
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: folder.job + (win.tasks.length > 1 ? "  +" + (win.tasks.length - 1) : "")
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeSmall
                color: taskHit.containsMouse ? FiliTheme.brume : FiliTheme.brume2
                MouseArea {
                    id: taskHit
                    anchors { fill: parent; margins: -4 }
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: tasksList.shown = !tasksList.shown
                }
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
        // Liste des tâches: la tâche en cours (avancement), puis la file; × retire ou arrête
        Rectangle {
            id: tasksList
            property bool shown: false
            visible: shown && win.tasks.length > 0
            anchors { horizontalCenter: parent.horizontalCenter; bottom: parent.top; bottomMargin: FiliTheme.space2 }
            width: 360
            height: win.tasks.length * 30 + 2 * FiliTheme.space1
            radius: FiliTheme.radiusControl
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
            Column {
                anchors { fill: parent; margins: FiliTheme.space1 }
                Repeater {
                    model: win.tasks
                    Item {
                        required property var modelData
                        required property int index
                        width: parent.width
                        height: 30
                        Text {
                            anchors { left: parent.left; leftMargin: FiliTheme.space2; right: bar.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                            text: modelData.label
                            elide: Text.ElideRight
                            font.family: FiliTheme.font
                            font.pixelSize: FiliTheme.sizeList
                            color: modelData.progress === -2 ? FiliTheme.ardoise : FiliTheme.brume
                        }
                        Rectangle {
                            id: bar
                            visible: modelData.progress !== -2
                            anchors { right: cancelTask.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                            width: 70
                            height: 4
                            radius: 2
                            color: FiliTheme.nuit3
                            Rectangle {
                                width: parent.width * Math.max(0.05, modelData.progress)
                                height: parent.height
                                radius: 2
                                color: FiliTheme.voile
                            }
                        }
                        Text {
                            id: cancelTask
                            anchors { right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                            text: "×"
                            font.pixelSize: 18
                            color: xArea.containsMouse ? FiliTheme.cendre : FiliTheme.ardoise
                            MouseArea {
                                id: xArea
                                anchors { fill: parent; margins: -6 }
                                hoverEnabled: true
                                onClicked: folder.cancelTask(index)
                            }
                        }
                    }
                }
            }
        }
    }

    // Tâches en cours et en attente (copies, déplacements, archives): rafraîchies à chaque changement de l'opération
    property var tasks: []
    Connections {
        target: folder
        function onChanged() { win.tasks = JSON.parse(folder.tasks()); }
    }
    // Confirmation (vider la corbeille, remplacer un fichier): Entrée valide, Échap annule
    // « Sélectionner selon un motif… »: *.pdf, photo*, a?.txt (sans joker: le nom contient le texte)
    Rectangle {
        id: matchBox
        function open() {
            patternInput.text = "";
            visible = true;
        }
        function apply() {
            visible = false;
            if (patternInput.text.trim() !== "")
                files.selectPaths(folder.matching(patternInput.text).split("\n"));
        }
        anchors.fill: parent
        visible: false
        z: 200
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        onVisibleChanged: if (visible) patternInput.forceActiveFocus(); else files.focusList()
        MouseArea {
            anchors.fill: parent
            onClicked: matchBox.visible = false
        }
        Rectangle {
            anchors.centerIn: parent
            width: 340
            height: matchCard.implicitHeight + 2 * FiliTheme.space4
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
            Column {
                id: matchCard
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space4 }
                spacing: FiliTheme.space3
                Text {
                    width: parent.width
                    text: Fili.tr("Select items matching")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: FiliTheme.weightAccent
                    color: FiliTheme.brume
                }
                Rectangle {
                    width: parent.width
                    height: 26
                    radius: FiliTheme.radiusControl
                    color: FiliTheme.nuit
                    border.width: 1
                    border.color: FiliTheme.voile
                    TextInput {
                        id: patternInput
                        anchors { fill: parent; leftMargin: FiliTheme.space2; rightMargin: FiliTheme.space2 }
                        verticalAlignment: TextInput.AlignVCenter
                        clip: true
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.size
                        color: FiliTheme.brume
                        onAccepted: matchBox.apply()
                        Keys.onEscapePressed: matchBox.visible = false
                    }
                }
                Text {
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: Fili.tr("* stands for any text, ? for one character: *.pdf, photo*, a?.txt")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
            }
        }
    }
    // « Se connecter au serveur… » (Ctrl+K): lecteur réseau par gvfs (smb://, sftp://, davs://, ftp://, nfs://); le dossier monté s'ouvre
    // à la réponse (Fili.mounted), une erreur s'affiche dans le message passager (Fili.volumeError)
    Rectangle {
        id: connectBox
        property bool busy: false
        function open() {
            busy = false;
            serverField.text = "";
            userField.text = "";
            passField.text = "";
            visible = true;
        }
        function apply() {
            if (busy || serverField.text.trim() === "")
                return;
            busy = true;
            Fili.connectServer(serverField.text, userField.text, passField.text);
        }
        anchors.fill: parent
        visible: false
        z: 200
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        onVisibleChanged: if (visible) serverField.field.forceActiveFocus(); else files.focusList()
        Connections {
            target: Fili
            function onMounted() { connectBox.busy = false; connectBox.visible = false; }
            function onVolumeError() { connectBox.busy = false; }
        }
        MouseArea {
            anchors.fill: parent
            onClicked: connectBox.visible = false
        }
        component ConnectField: Rectangle {
            property alias text: input.text
            property alias field: input
            property string label
            property bool secret: false
            property Item next
            width: parent ? parent.width : 100
            height: 26
            radius: FiliTheme.radiusControl
            color: FiliTheme.nuit
            border.width: 1
            border.color: input.activeFocus ? FiliTheme.voile : FiliTheme.nuit4
            Text {
                visible: input.text === ""
                anchors { left: parent.left; leftMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                text: parent.label
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: FiliTheme.ardoise
            }
            TextInput {
                id: input
                anchors { fill: parent; leftMargin: FiliTheme.space2; rightMargin: FiliTheme.space2 }
                verticalAlignment: TextInput.AlignVCenter
                clip: true
                echoMode: parent.secret ? TextInput.Password : TextInput.Normal
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: FiliTheme.brume
                selectByMouse: true
                onAccepted: connectBox.apply()
                KeyNavigation.tab: parent.next ? parent.next.field : null
                Keys.onEscapePressed: connectBox.visible = false
            }
        }
        Rectangle {
            anchors.centerIn: parent
            width: 380
            height: connectCard.implicitHeight + 2 * FiliTheme.space4
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
            Column {
                id: connectCard
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space4 }
                spacing: FiliTheme.space3
                Text {
                    width: parent.width
                    text: Fili.tr("Connect to Server")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: FiliTheme.weightAccent
                    color: FiliTheme.brume
                }
                ConnectField { id: serverField; label: Fili.tr("smb://server/share"); next: userField }
                ConnectField { id: userField; label: Fili.tr("User (optional)"); next: passField }
                ConnectField { id: passField; label: Fili.tr("Password (optional)"); secret: true; next: serverField }
                Text {
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: connectBox.busy ? Fili.tr("Connecting…") : Fili.tr("smb://, sftp://, davs://, ftp://, nfs://. The connection stays in the sidebar until you eject it.")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
                Row {
                    spacing: FiliTheme.space2
                    anchors.right: parent.right
                    TextButton { label: Fili.tr("Cancel"); onClicked: connectBox.visible = false }
                    TextButton { label: Fili.tr("Connect"); primary: true; onClicked: connectBox.apply() }
                }
            }
        }
    }
    // Renommage en lot (F2 avec plusieurs éléments): remplacer du texte, numéroter, changer la casse; aperçu avant/après
    component BulkField: Rectangle {
        property alias text: input.text
        property string label
        signal edited()
        width: parent ? parent.width : 100
        height: 26
        radius: FiliTheme.radiusControl
        color: FiliTheme.nuit
        border.width: 1
        border.color: input.activeFocus ? FiliTheme.voile : FiliTheme.nuit4
        Text {
            visible: input.text === ""
            anchors { left: parent.left; leftMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
            text: parent.label
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.size
            color: FiliTheme.ardoise
        }
        TextInput {
            id: input
            anchors { fill: parent; leftMargin: FiliTheme.space2; rightMargin: FiliTheme.space2 }
            verticalAlignment: TextInput.AlignVCenter
            clip: true
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.size
            color: FiliTheme.brume
            selectByMouse: true
            onTextEdited: parent.edited()
            Keys.onEscapePressed: bulkBox.visible = false
            Keys.onReturnPressed: bulkBox.apply()
            Keys.onEnterPressed: bulkBox.apply()
        }
    }
    Rectangle {
        id: bulkBox
        property string paths: ""
        property string mode: "replace"
        property string caseMode: "lower"
        property var plan: []
        readonly property bool changed: plan.some(r => !r.same)
        readonly property bool valid: plan.length > 0 && plan.every(r => r.ok)
        function open() {
            paths = Object.keys(files.selected).join("\n");
            findField.text = "";
            replaceField.text = "";
            patternField.text = "{name} {n}";
            padField.text = "2";
            startField.text = "1";
            mode = "replace";
            caseMode = "lower";
            refresh();
            visible = true;
            findField.forceFocus();
        }
        function refresh() {
            const a = mode === "replace" ? findField.text : mode === "number" ? patternField.text : caseMode;
            const b = mode === "replace" ? replaceField.text : padField.text;
            plan = JSON.parse(folder.bulkPreview(paths, mode, a, b, parseInt(startField.text) || 1));
        }
        function apply() {
            if (!changed || !valid)
                return;
            const a = mode === "replace" ? findField.text : mode === "number" ? patternField.text : caseMode;
            const b = mode === "replace" ? replaceField.text : padField.text;
            if (folder.bulkApply(paths, mode, a, b, parseInt(startField.text) || 1))
                visible = false;
        }
        anchors.fill: parent
        visible: false
        z: 200
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        focus: visible
        onVisibleChanged: if (!visible) files.focusList()
        Keys.onEscapePressed: visible = false
        onModeChanged: refresh()
        MouseArea {
            anchors.fill: parent
            onClicked: bulkBox.visible = false
        }
        Rectangle {
            anchors.centerIn: parent
            width: 460
            height: bulkCard.implicitHeight + 2 * FiliTheme.space4
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
            MouseArea { anchors.fill: parent }
            Column {
                id: bulkCard
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space4 }
                spacing: FiliTheme.space3
                Text {
                    width: parent.width
                    text: Fili.tr("Rename %1 items").replace("%1", bulkBox.plan.length)
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: FiliTheme.weightAccent
                    color: FiliTheme.brume
                }
                Row {
                    spacing: FiliTheme.space2
                    TextButton { label: Fili.tr("Replace Text"); primary: bulkBox.mode === "replace"; onClicked: { bulkBox.mode = "replace"; findField.forceFocus(); } }
                    TextButton { label: Fili.tr("Number"); primary: bulkBox.mode === "number"; onClicked: { bulkBox.mode = "number"; patternField.forceFocus(); } }
                    TextButton { label: Fili.tr("Change Case"); primary: bulkBox.mode === "case"; onClicked: bulkBox.mode = "case" }
                }
                Column {
                    width: parent.width
                    spacing: FiliTheme.space2
                    visible: bulkBox.mode === "replace"
                    BulkField { id: findField; label: Fili.tr("Find"); onEdited: bulkBox.refresh(); function forceFocus() { forceActiveFocus(); } }
                    BulkField { id: replaceField; label: Fili.tr("Replace with"); onEdited: bulkBox.refresh() }
                }
                Column {
                    width: parent.width
                    spacing: FiliTheme.space2
                    visible: bulkBox.mode === "number"
                    BulkField { id: patternField; label: Fili.tr("Name, with {name} and {n}"); onEdited: bulkBox.refresh(); function forceFocus() { forceActiveFocus(); } }
                    Row {
                        spacing: FiliTheme.space2
                        BulkField { id: startField; width: 120; label: Fili.tr("Start at"); onEdited: bulkBox.refresh() }
                        BulkField { id: padField; width: 160; label: Fili.tr("Digits"); onEdited: bulkBox.refresh() }
                    }
                }
                Row {
                    visible: bulkBox.mode === "case"
                    spacing: FiliTheme.space2
                    TextButton { label: Fili.tr("lowercase"); primary: bulkBox.caseMode === "lower"; onClicked: { bulkBox.caseMode = "lower"; bulkBox.refresh(); } }
                    TextButton { label: Fili.tr("UPPERCASE"); primary: bulkBox.caseMode === "upper"; onClicked: { bulkBox.caseMode = "upper"; bulkBox.refresh(); } }
                    TextButton { label: Fili.tr("Capitalize Words"); primary: bulkBox.caseMode === "title"; onClicked: { bulkBox.caseMode = "title"; bulkBox.refresh(); } }
                }
                Rectangle {
                    width: parent.width
                    height: Math.min(bulkBox.plan.length, 8) * 22 + 2 * FiliTheme.space1
                    radius: FiliTheme.radiusControl
                    color: FiliTheme.nuit
                    border.width: 1
                    border.color: FiliTheme.nuit4
                    ListView {
                        anchors { fill: parent; margins: FiliTheme.space1 }
                        clip: true
                        model: bulkBox.plan
                        boundsBehavior: Flickable.StopAtBounds
                        delegate: Item {
                            required property var modelData
                            width: ListView.view.width
                            height: 22
                            Text {
                                anchors { left: parent.left; leftMargin: FiliTheme.space2; right: arrow.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                                text: modelData.from
                                elide: Text.ElideMiddle
                                font.family: FiliTheme.font
                                font.pixelSize: FiliTheme.sizeList
                                color: FiliTheme.brume2
                            }
                            Text {
                                id: arrow
                                anchors { horizontalCenter: parent.horizontalCenter; verticalCenter: parent.verticalCenter }
                                text: "→"
                                font.pixelSize: FiliTheme.sizeList
                                color: FiliTheme.ardoise
                            }
                            Text {
                                anchors { left: arrow.right; leftMargin: FiliTheme.space2; right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                                text: modelData.to === "" ? "—" : modelData.to
                                elide: Text.ElideMiddle
                                font.family: FiliTheme.font
                                font.pixelSize: FiliTheme.sizeList
                                color: !modelData.ok ? FiliTheme.cendre : modelData.same ? FiliTheme.ardoise : FiliTheme.brume
                            }
                        }
                    }
                }
                Text {
                    width: parent.width
                    visible: bulkBox.changed && !bulkBox.valid
                    wrapMode: Text.Wrap
                    text: Fili.tr("Some new names are not valid or already exist")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.cendre
                }
                Row {
                    anchors.right: parent.right
                    spacing: FiliTheme.space2
                    TextButton { label: Fili.tr("Cancel"); onClicked: bulkBox.visible = false }
                    TextButton {
                        label: Fili.tr("Rename")
                        primary: true
                        opacity: bulkBox.changed && bulkBox.valid ? 1 : 0.4
                        onClicked: bulkBox.apply()
                    }
                }
            }
        }
    }
    // Un nom existe déjà dans la destination d'une copie ou d'un déplacement (Rust: signal conflict)
    Connections {
        target: folder
        function onConflict(names, dest) {
            const list = names.split("\n");
            const where = dest.split("/").pop() || "/";
            conflictBox.heading = list.length === 1 ? Fili.tr("“%1” already exists in “%2”.").replace("%1", list[0]).replace("%2", where) : Fili.tr("%1 items already exist in “%2”.").replace("%1", list.length).replace("%2", where);
            conflictBox.body = Fili.tr("Replacing moves the existing item to the Trash (Ctrl+Z brings it back). Keep Both adds “copy” to the name.");
            conflictBox.many = list.length > 1;
            conflictBox.visible = true;
        }
    }
    Rectangle {
        id: conflictBox
        property string heading
        property string body
        property bool many: false
        function choose(c) {
            visible = false;
            folder.resolveConflict(c);
        }
        anchors.fill: parent
        visible: false
        z: 200
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        focus: visible
        onVisibleChanged: if (visible) forceActiveFocus(); else files.focusList()
        Keys.onPressed: e => {
            if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter)
                choose("both");
            else if (e.key === Qt.Key_Escape)
                choose("cancel");
            else
                return;
            e.accepted = true;
        }
        MouseArea {
            anchors.fill: parent
            onClicked: conflictBox.choose("cancel")
        }
        Rectangle {
            anchors.centerIn: parent
            width: 380
            height: conflictCard.implicitHeight + 2 * FiliTheme.space4
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
            Column {
                id: conflictCard
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space4 }
                spacing: FiliTheme.space3
                Text {
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: conflictBox.heading
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: FiliTheme.weightAccent
                    color: FiliTheme.brume
                }
                Text {
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: conflictBox.body
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
                Row {
                    anchors.right: parent.right
                    spacing: FiliTheme.space2
                    TextButton {
                        label: Fili.tr("Cancel")
                        onClicked: conflictBox.choose("cancel")
                    }
                    TextButton {
                        label: conflictBox.many ? Fili.tr("Skip These") : Fili.tr("Skip")
                        onClicked: conflictBox.choose("skip")
                    }
                    TextButton {
                        label: Fili.tr("Replace")
                        onClicked: conflictBox.choose("replace")
                    }
                    TextButton {
                        label: Fili.tr("Keep Both")
                        primary: true
                        onClicked: conflictBox.choose("both")
                    }
                }
            }
        }
    }
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
        infoCard.mode = list.length === 1 ? folder.permMode(list[0]) : -1;
        infoCard.path = list[0];
        infoCard.visible = true;
        folder.startInfoSize(list.join("\n"));
        if (infoCard.rows.some(r => r[0] === "SHA-256"))
            folder.startInfoHash(list[0]);
    }
    Connections {
        target: folder
        function onInfoHash(text) {
            const r = infoCard.rows.slice();
            const i = r.findIndex(x => x[0] === "SHA-256");
            if (i >= 0 && infoCard.visible) {
                r[i] = [r[i][0], text];
                infoCard.rows = r;
            }
        }
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
        // Droits modifiables (0..511) de l'élément seul affiché, -1 si on ne peut pas les changer
        property int mode: -1
        property string path: ""
        function toggleBit(bit) {
            const text = folder.setPerm(path, mode ^ bit);
            if (text === "")
                return;
            mode = mode ^ bit;
            const r = rows.slice();
            const i = r.findIndex(x => x[0] === Fili.tr("Permissions"));
            if (i >= 0) {
                r[i] = [r[i][0], text];
                rows = r;
            }
        }
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
            height: grid.implicitHeight + okRow.height + 3 * FiliTheme.space4 + (infoCard.mode >= 0 ? permBox.height + FiliTheme.space3 : 0)
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
            // Droits: trois lignes (propriétaire, groupe, autres) de trois cases lecture / écriture / exécution
            Column {
                id: permBox
                visible: infoCard.mode >= 0
                anchors { left: parent.left; top: grid.bottom; leftMargin: FiliTheme.space4 + 110 + FiliTheme.space3; topMargin: FiliTheme.space3 }
                spacing: FiliTheme.space1
                Repeater {
                    model: [{ label: Fili.tr("Owner"), shift: 6 }, { label: Fili.tr("Group"), shift: 3 }, { label: Fili.tr("Others"), shift: 0 }]
                    Row {
                        required property var modelData
                        spacing: FiliTheme.space1
                        Text {
                            width: 64
                            anchors.verticalCenter: parent.verticalCenter
                            text: modelData.label
                            font.family: FiliTheme.font
                            font.pixelSize: FiliTheme.sizeList
                            color: FiliTheme.brume2
                        }
                        Repeater {
                            model: [{ ch: "r", bit: 4 }, { ch: "w", bit: 2 }, { ch: "x", bit: 1 }]
                            Rectangle {
                                required property var modelData
                                readonly property int bit: modelData.bit << parent.modelData.shift
                                readonly property bool on: (infoCard.mode & bit) !== 0
                                width: 28
                                height: 22
                                radius: FiliTheme.radiusControl
                                color: on ? FiliTheme.nuit4 : permArea.containsMouse ? FiliTheme.nuit3 : FiliTheme.nuit
                                border.width: 1
                                border.color: on ? FiliTheme.voile : FiliTheme.nuit4
                                Text {
                                    anchors.centerIn: parent
                                    text: modelData.ch
                                    font.family: FiliTheme.mono
                                    font.pixelSize: FiliTheme.sizeList
                                    color: on ? FiliTheme.brume : FiliTheme.ardoise
                                }
                                MouseArea {
                                    id: permArea
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    onClicked: infoCard.toggleBit(parent.bit)
                                }
                            }
                        }
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

    // À propos de Casier (nom affiché de fili-file): icône, nom, alias, version et le reste de la fiche
    function showAbout() {
        aboutCard.rows = JSON.parse(Fili.aboutRows());
        aboutCard.visible = true;
    }
    Rectangle {
        id: aboutCard
        property var rows: []
        anchors.fill: parent
        visible: false
        z: 210
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        focus: visible
        onVisibleChanged: if (visible) forceActiveFocus(); else files.focusList()
        Keys.onPressed: e => {
            if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter || e.key === Qt.Key_Escape)
                visible = false;
            else
                return;
            e.accepted = true;
        }
        MouseArea {
            anchors.fill: parent
            onClicked: aboutCard.visible = false
        }
        Rectangle {
            anchors.centerIn: parent
            width: 380
            height: aboutColumn.implicitHeight + 2 * FiliTheme.space5
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: 1
            border.color: FiliTheme.nuit4
            MouseArea { anchors.fill: parent }
            Column {
                id: aboutColumn
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space5 }
                spacing: FiliTheme.space3
                // Dossier du thème par défaut (clair ou sombre selon le thème) portant le logo de Filigrane
                Item {
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 96
                    height: 96
                    Image {
                        anchors.fill: parent
                        source: Fili.aboutFolder(FiliTheme.light)
                        sourceSize: Qt.size(192, 192)
                    }
                    Image {
                        width: parent.width * 0.62
                        height: width * 0.24
                        x: (parent.width - width) / 2
                        y: parent.height * 0.52
                        source: Fili.aboutLogo(FiliTheme.light)
                        sourceSize.width: 192
                        fillMode: Image.PreserveAspectFit
                    }
                }
                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: "Casier"
                    textFormat: Text.PlainText
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeTitle
                    font.weight: FiliTheme.weightAccent
                    color: FiliTheme.brume
                }
                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: Fili.tr("The file manager of Filigrane")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: Font.Light
                    color: FiliTheme.brume2
                }
                Rectangle { width: parent.width; height: 1; color: FiliTheme.nuit4 }
                Grid {
                    id: aboutGrid
                    width: parent.width
                    columns: 2
                    columnSpacing: FiliTheme.space3
                    rowSpacing: FiliTheme.space2
                    Repeater {
                        model: aboutCard.rows.length * 2
                        Text {
                            readonly property bool label: index % 2 === 0
                            width: label ? 110 : aboutGrid.width - 110 - FiliTheme.space3
                            horizontalAlignment: label ? Text.AlignRight : Text.AlignLeft
                            wrapMode: Text.WrapAnywhere
                            text: aboutCard.rows[Math.floor(index / 2)][label ? 0 : 1]
                            textFormat: Text.PlainText
                            font.family: FiliTheme.font
                            font.pixelSize: FiliTheme.sizeList
                            font.weight: label ? FiliTheme.weightAccent : Font.Normal
                            color: label ? FiliTheme.brume2 : FiliTheme.brume
                        }
                    }
                }
                Item {
                    width: parent.width
                    height: aboutOk.height
                    TextButton {
                        id: aboutOk
                        anchors.right: parent.right
                        label: Fili.tr("OK")
                        primary: true
                        onClicked: aboutCard.visible = false
                    }
                }
            }
        }
    }
}
