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

    // Pas de fermeture pendant une opération (copie, déplacement, archive): attendre ou annuler (Échap), car une copie
    // coupée en route laisse des fichiers à moitié écrits
    readonly property bool working: folder.busy || win.tasks.length > 0
    function refuseClose() {
        win.notice = Fili.tr("An operation is in progress: wait for it to finish, or cancel it (Esc), before closing.");
        noticeTimer.restart();
    }
    function quit() {
        if (working)
            refuseClose();
        else
            Qt.quit();
    }
    onClosing: close => {
        if (working) {
            close.accepted = false;
            refuseClose();
        }
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
    // « À proximité » (partage de proximité) remplace la liste des fichiers; tout changement de dossier la ferme
    // Le lieu « Réseau » de la barre latérale est affiché (vue de résultats)
    readonly property bool networkView: folder.results === Fili.tr("Network")
    property bool nearby: false
    property string nearbyFrom: ""
    function open(path) {
        if (path === "nearby:") {
            if (picking)
                return;
            nearbyFrom = folder.shown;
            nearby = true;
            nearbyView.forceActiveFocus();
            return;
        }
        nearby = false;
        if (path === "network:") {
            folder.showNetwork(Fili.tr("Network"));
            files.select(0);
            files.focusList();
            return;
        }
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
        onTriggered: Fili.publishContext(folder.path, Object.keys(files.selected).join("\n"), folder.results, win.disabledNow)
    }
    // Commandes de menu impossibles maintenant (rien de sélectionné, dialogue ouvert...): publiées pour que la fili-bar les grise
    readonly property string disabledNow: Object.keys(menuActions).filter(k => !menuActions[k].enabled).sort().join(",")
    onDisabledNowChanged: contextTimer.restart()
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
        function onCommand(id, verb, arg, body) {
            Fili.reply(id, JSON.stringify(win.runCommand(verb, arg, body)));
        }
    }
    Component.onDestruction: Fili.publishContext("", "", "", "")

    Component.onCompleted: {
        nameInput.text = Fili.pickName();
        show(folder.go(Fili.startPath()));
        files.focusList();
        if (Fili.startInfo())
            showInfo();
    }

    Shortcut { sequence: "Ctrl+["; onActivated: win.show(folder.back()) }
    Shortcut { sequence: "Ctrl+]"; onActivated: win.show(folder.forward()) }


    // Éléments sélectionnés (ceux qu'on voit surlignés): les commandes de fichiers les visent, et sont grisées sans eux.
    // Une ligne seulement « courante » (après un clic dans le vide) n'est pas une cible.
    readonly property int selCount: Object.keys(files.selected).length
    // Commandes de la barre de menus (exposée au menu global de la fili-bar par AT-SPI); chaque raccourci vit ici, une seule fois
    readonly property bool canAct: !win.typing && !win.picking && !win.nearby
    Action { id: aNewFolder; text: Fili.tr("New Folder"); shortcut: "Ctrl+Shift+N"; enabled: win.canAct && !folder.inTrash; onTriggered: win.newFolder() }
    Action { id: aOpen; text: Fili.tr("Open"); enabled: win.canAct && win.selCount === 1; onTriggered: win.show(folder.activate(files.currentIndex)) }
    // Espace: aperçu rapide (script quicklook; relancé sur le même fichier, il se referme). Désactivée pendant la saisie au
    // clavier d'un début de nom (« mon fichier »): l'espace est alors une lettre.
    Action { id: aQuickLook; text: Fili.tr("Quick Look"); shortcut: "Space"; enabled: !win.typing && !win.nearby && files.typed === "" && win.selCount > 0; onTriggered: folder.quicklook(files.targets()[0]) }
    Action { id: aInfo; text: Fili.tr("Get Info"); shortcut: "Ctrl+I"; enabled: win.canAct; onTriggered: win.showInfo() }
    Action { id: aTrash; text: Fili.tr("Move to Trash"); shortcut: "Delete"; enabled: win.canAct && !folder.inTrash && win.selCount > 0; onTriggered: win.trashSelection() }
    Action { id: aTrashAlt; shortcut: "Ctrl+Backspace"; enabled: win.canAct && !folder.inTrash && win.selCount > 0; onTriggered: win.trashSelection() }
    Action { id: aPutBack; text: Fili.tr("Put Back"); enabled: win.canAct && folder.inTrash && win.selCount > 0; onTriggered: win.putBack() }
    Action { id: aEmpty; text: Fili.tr("Empty Trash"); shortcut: "Ctrl+Shift+Delete"; enabled: !win.typing && folder.inTrash; onTriggered: win.askEmpty() }
    Action { id: aAbout; text: Fili.tr("About Casier"); onTriggered: win.showAbout() }
    // Ctrl+W ferme l'onglet, et la fenêtre quand il n'y en a plus qu'un
    Action { id: aClose; text: Fili.tr("Close"); shortcut: "Ctrl+W"; onTriggered: win.tabs.length > 1 ? win.closeTab(win.tabIndex) : win.quit() }
    Action { id: aNewTab; text: Fili.tr("New Tab"); shortcut: "Ctrl+T"; enabled: !win.picking; onTriggered: win.newTab() }
    Action { id: aDetachTab; text: Fili.tr("Move Tab to New Window"); enabled: !win.picking && win.tabs.length > 1; onTriggered: win.detachTab(win.tabIndex) }
    Action { id: aNextTab; text: Fili.tr("Show Next Tab"); shortcut: "Ctrl+Tab"; enabled: win.tabs.length > 1; onTriggered: win.switchTab((win.tabIndex + 1) % win.tabs.length) }
    Action { id: aPrevTab; text: Fili.tr("Show Previous Tab"); shortcut: "Ctrl+Shift+Tab"; enabled: win.tabs.length > 1; onTriggered: win.switchTab((win.tabIndex + win.tabs.length - 1) % win.tabs.length) }
    Action { id: aQuit; text: Fili.tr("Quit"); shortcut: "Ctrl+Q"; onTriggered: win.quit() }
    Action { id: aUndo; text: Fili.tr("Undo"); shortcut: "Ctrl+Z"; enabled: win.canAct; onTriggered: folder.undo() }
    Action { id: aCut; text: Fili.tr("Cut"); shortcut: "Ctrl+X"; enabled: win.canAct && !folder.inTrash && win.selCount > 0; onTriggered: folder.cut(files.targetsText()) }
    Action { id: aCopy; text: Fili.tr("Copy"); shortcut: "Ctrl+C"; enabled: win.canAct && win.selCount > 0; onTriggered: folder.copy(files.targetsText()) }
    Action { id: aPaste; text: Fili.tr("Paste"); shortcut: "Ctrl+V"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.paste() }
    Action { id: aDuplicate; text: Fili.tr("Duplicate"); shortcut: "Ctrl+D"; enabled: win.canAct && !folder.inTrash && win.selCount > 0; onTriggered: folder.duplicate(files.targetsText()) }
    // F2: renommer sur place; avec plusieurs éléments sélectionnés, renommer en lot
    Action { id: aRename; text: Fili.tr("Rename"); shortcut: "F2"; enabled: win.canAct && !folder.inTrash && win.selCount > 0; onTriggered: Object.keys(files.selected).length > 1 ? bulkBox.open() : files.startRename() }
    Action { id: aBulk; text: Fili.tr("Rename Items…"); enabled: win.canAct && !folder.inTrash && Object.keys(files.selected).length > 1; onTriggered: bulkBox.open() }
    Action { id: aSelectAll; text: Fili.tr("Select All"); shortcut: "Ctrl+A"; enabled: win.canAct; onTriggered: files.selectAll() }
    Action { id: aInvert; text: Fili.tr("Invert Selection"); shortcut: "Ctrl+Shift+I"; enabled: win.canAct; onTriggered: files.invertSelection() }
    Action { id: aMatch; text: Fili.tr("Select Matching…"); shortcut: "Ctrl+Shift+M"; enabled: win.canAct && !win.picking; onTriggered: matchBox.open() }
    Action { id: aSameKind; text: Fili.tr("Select Same Kind"); enabled: win.canAct && win.selCount > 0; onTriggered: files.selectPaths(folder.sameKind(files.targetsText()).split("\n")) }
    Action { id: aTerminal; text: Fili.tr("Open Terminal Here"); shortcut: "Ctrl+Alt+T"; enabled: win.canAct && !folder.inTrash; onTriggered: folder.terminal(files.targets().length === 1 ? files.targets()[0] : "") }
    Action { id: aList; text: Fili.tr("View as List"); shortcut: "Ctrl+1"; checkable: true; checked: !files.tiles; onTriggered: files.setViewMode("list") }
    Action { id: aGrid; text: Fili.tr("View as Icons"); shortcut: "Ctrl+2"; checkable: true; checked: files.tiles; onTriggered: files.setViewMode("grid") }
    Action { id: aCopyPath; text: Fili.tr("Copy Full Path"); shortcut: "Ctrl+Shift+C"; enabled: win.canAct && win.selCount > 0; onTriggered: win.copyAs("path") }
    Action { id: aCopyName; text: Fili.tr("Copy Name"); shortcut: "Ctrl+Alt+C"; enabled: win.canAct && win.selCount > 0; onTriggered: win.copyAs("name") }
    Action { id: aCopyStem; text: Fili.tr("Copy Name Without Extension"); enabled: win.canAct && win.selCount > 0; onTriggered: win.copyAs("stem") }
    Action { id: aCopyParent; text: Fili.tr("Copy Parent Folder"); enabled: win.canAct && win.selCount > 0; onTriggered: win.copyAs("parent") }
    Action { id: aPreview; text: Fili.tr("Show Preview"); shortcut: "Ctrl+Shift+P"; checkable: true; checked: win.previewOn; enabled: !win.picking; onTriggered: win.togglePreview() }
    Action { id: aHidden; text: Fili.tr("Show Hidden Files"); shortcut: "Ctrl+H"; checkable: true; checked: folder.showHidden; onTriggered: win.show(folder.toggleHidden(files.currentIndex)) }
    Action { id: aRefresh; text: Fili.tr("Reload"); shortcut: "F5"; onTriggered: win.show(folder.refresh(files.currentIndex)) }
    Action { id: aRefreshAlt; shortcut: "Ctrl+R"; onTriggered: win.show(folder.refresh(files.currentIndex)) }
    Action { id: aBack; text: Fili.tr("Back"); shortcut: "Alt+Left"; enabled: folder.canBack; onTriggered: win.show(folder.back()) }
    Action { id: aForward; text: Fili.tr("Forward"); shortcut: "Alt+Right"; enabled: folder.canForward; onTriggered: win.show(folder.forward()) }
    Action { id: aUp; text: Fili.tr("Enclosing Folder"); shortcut: "Alt+Up"; onTriggered: win.show(folder.up()) }
    Action { id: aShare; text: Fili.tr("Share…"); enabled: win.canAct && !folder.inTrash && !win.picking && win.shareable().length > 0; onTriggered: shareBox.open() }
    Action { id: aNearby; text: Fili.tr("Nearby"); shortcut: "Ctrl+Shift+R"; enabled: !win.picking; onTriggered: win.open("nearby:") }
    Action { id: aPath; text: Fili.tr("Go to Folder…"); shortcut: "Ctrl+L"; onTriggered: pathBar.edit() }
    Action { id: aCompress; text: Fili.tr("Compress…"); enabled: win.canAct && !folder.inTrash && !win.picking && win.selCount > 0; onTriggered: compressBox.open(files.targetsText()) }
    Action { id: aExtract; text: Fili.tr("Extract Here"); enabled: win.canAct && !folder.inTrash && !win.picking && Object.keys(files.selected).length > 0 && folder.allArchives(files.targetsText()); onTriggered: folder.extract(files.targetsText(), "") }
    Action { id: aConnect; text: Fili.tr("Connect to Server…"); shortcut: "Ctrl+K"; onTriggered: connectBox.open() }
    // « / » comme Ctrl+F (façon vim, yazi); sans libellé: la barre de menus ne montre que Ctrl+F
    Action { id: aSearchSlash; shortcut: "/"; enabled: win.canAct; onTriggered: searchInput.forceActiveFocus() }
    Action { id: aSearch; text: Fili.tr("Search"); shortcut: "Ctrl+F"; onTriggered: searchInput.forceActiveFocus() }

    // `fili-file ctl action NOM`: la fili-bar affiche ces commandes dans son menu (dotfiles/config/filigrane/appmenu/fili-file.json,
    // test tests/fili-shell/check-fili-file-menu.py: mêmes noms, mêmes raccourcis). Une commande grisée ne fait rien.
    readonly property var menuActions: ({
        "new-folder": aNewFolder, "open": aOpen, "info": aInfo, "quick-look": aQuickLook, "share": aShare, "nearby": aNearby, "trash": aTrash, "put-back": aPutBack, "empty-trash": aEmpty, "close": aClose, "new-tab": aNewTab, "detach-tab": aDetachTab, "next-tab": aNextTab, "prev-tab": aPrevTab, "about": aAbout,
        "undo": aUndo, "cut": aCut, "copy": aCopy, "paste": aPaste, "duplicate": aDuplicate, "compress": aCompress, "extract": aExtract, "rename": aRename, "bulk-rename": aBulk,
        "select-all": aSelectAll, "invert-selection": aInvert, "select-matching": aMatch, "select-same-kind": aSameKind, "terminal": aTerminal, "view-list": aList, "view-icons": aGrid, "toggle-hidden": aHidden, "toggle-preview": aPreview, "copy-path": aCopyPath, "copy-name": aCopyName, "copy-stem": aCopyStem, "copy-parent": aCopyParent,
        "reload": aRefresh, "back": aBack, "forward": aForward, "up": aUp, "path": aPath, "connect-server": aConnect, "search": aSearch
    })
    // Fichiers de la sélection qu'on peut envoyer (les dossiers ne sont pas envoyés tels quels)
    function shareable() {
        return files.targets().filter(p => !folder.pathIsDir(p));
    }
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
    // Déplace l'onglet from à la place to (glisser un onglet); l'onglet affiché reste le même
    function moveTab(from, to) {
        if (from === to || from < 0 || to < 0 || from >= tabs.length || to >= tabs.length)
            return;
        const t = tabs.slice();
        t[tabIndex] = { path: folder.shown };
        const current = t[tabIndex];
        const moved = t.splice(from, 1)[0];
        t.splice(to, 0, moved);
        tabs = t;
        tabIndex = t.indexOf(current);
    }
    // Détache l'onglet i dans une nouvelle fenêtre de Casier (même dossier) et le ferme ici. Rend false s'il est le seul.
    function detachTab(i) {
        if (tabs.length < 2 || i < 0 || i >= tabs.length)
            return false;
        Fili.openWindow(i === tabIndex ? folder.shown : tabs[i].path);
        closeTab(i);
        return true;
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
    // Largeur choisie du panneau d'aperçu (glisser son bord gauche; double-clic: 300), gardée dans l'état. Le panneau
    // la réduit si la fenêtre manque de place.
    readonly property int previewDefault: 300
    property real previewWidth: {
        const v = parseInt(Fili.uiState("preview"));
        return v >= 160 ? Math.min(v, 900) : previewDefault;
    }
    // px <= 0: largeur par défaut; save=false pendant le glisser
    function setPreviewWidth(px, save) {
        previewWidth = px > 0 ? Math.max(160, Math.min(900, Math.round(px))) : previewDefault;
        if (save !== false)
            Fili.setUiState("preview", px > 0 ? String(previewWidth) : "");
    }
    function togglePreview() {
        previewOn = !previewOn;
        Fili.setPreviewShown(previewOn);
    }
    // Déclenche une commande de menu; renvoie {ok} ou {ok:false, error} (inconnue, désactivée). Avec un argument
    // (ctl action NOM ARG), les commandes à dialogue le sautent: rename NOM, select-matching MOTIF, compress NOM[.zip|.7z|
    // .tar.gz|.tar.xz|.tar.zst], extract MOT_DE_PASSE, new-folder NOM, search TEXTE, path CHEMIN
    function runAction(name, arg) {
        const a = menuActions[name];
        if (!a)
            return { ok: false, error: "unknown action: " + name + " (fili-file ctl actions lists them)" };
        if (!a.enabled)
            return { ok: false, error: "action disabled now: " + name + (win.typing ? " (a dialog or a text field is active)" : "") };
        if (arg === undefined || arg === "") {
            a.trigger();
            return { ok: true };
        }
        folder.clearError();
        const failed = () => ({ ok: false, error: folder.error || (name + " failed") });
        const t = files.targets();
        switch (name) {
        case "rename": {
            if (t.length !== 1)
                return { ok: false, error: "rename NAME needs exactly one selected item (" + t.length + " selected)" };
            const r = folder.rename(t[0], arg);
            if (r < 0)
                return failed();
            files.select(r);
            return { ok: true };
        }
        case "new-folder": {
            const row = folder.newFolder();
            if (row < 0)
                return failed();
            const r = folder.rename(folder.pathAt(row), arg);
            if (r < 0)
                return failed();
            files.select(r);
            return { ok: true };
        }
        case "select-matching": {
            const list = folder.matching(arg).split("\n").filter(p => p !== "");
            files.selectPaths(list);
            return { ok: true, selected: list.length };
        }
        case "compress": {
            const m = /^(.*)\.(zip|7z|tar\.gz|tar\.xz|tar\.zst)$/.exec(arg);
            folder.compress(files.targetsText(), m ? m[1] : arg, m ? m[2] : "zip", "normal", false, "");
            return folder.error === "" ? { ok: true } : failed();
        }
        case "extract":
            folder.extract(files.targetsText(), arg);
            return folder.error === "" ? { ok: true } : failed();
        case "search":
            searchInput.text = arg;
            return { ok: true, filter: folder.filter };
        case "path":
            return runCommand("goto", arg, "");
        }
        return { ok: false, error: name + " takes no argument" };
    }
    // Aller à un chemin; un fichier est sélectionné dans son dossier
    function gotoPath(path) {
        win.open(path);
        return folder.error === "" ? { ok: true } : { ok: false, error: folder.error };
    }
    // Montrer un élément (dossier compris) dans son dossier parent, sélectionné
    function reveal(path) {
        const parent = path.replace(/\/[^\/]*\/?$/, "") || "/";
        const r = gotoPath(parent);
        if (!r.ok)
            return r;
        if (!folder.has(path))
            return { ok: false, error: "not found in " + parent + ": " + path };
        files.selectPaths([path]);
        files.focusList();
        return { ok: true };
    }
    // Sélection: set (liste en corps), add, none, pattern (motif en argument); rend le nombre sélectionné et les chemins absents
    function selectCommand(mode, arg, body) {
        const list = body.split("\n").filter(p => p !== "");
        const missing = list.filter(p => !folder.has(p));
        if (mode === "none") {
            files.selectPaths([]);
        } else if (mode === "pattern") {
            files.selectPaths(folder.matching(arg).split("\n"));
        } else if (mode === "add") {
            files.selectPaths(Object.keys(files.selected).concat(list.filter(p => folder.has(p))));
        } else if (mode === "set") {
            files.selectPaths(list.filter(p => folder.has(p)));
        } else {
            return { ok: false, error: "unknown selection mode: " + mode };
        }
        // Comme un clic: la ligne courante est la première sélectionnée (Ouvrir, Aperçu rapide... partent d'elle)
        const first = Object.keys(files.selected)[0];
        if (first !== undefined) {
            const row = folder.rowOf(first.split("/").pop());
            if (row >= 0)
                files.active.currentIndex = row;
        }
        const n = Object.keys(files.selected).length;
        if (missing.length > 0)
            return { ok: false, error: "not in the folder shown (" + folder.shown + "): " + missing.slice(0, 5).join(", ") + (missing.length > 5 ? " …" : ""), selected: n };
        return { ok: true, selected: n };
    }
    // Largeurs: column NOM PX|reset (size, modified, kind, created, permissions, owner), preview PX|reset. Mêmes
    // fonctions que le glisser des bords; PX est borné (colonne 56 à 600, aperçu 160 à 900)
    function widthCommand(arg) {
        const parts = arg.split("\t");
        const usage = { ok: false, error: "usage: width column NAME PX|reset | width preview PX|reset" };
        const value = v => v === "reset" ? 0 : /^\d+$/.test(v) ? parseInt(v) : -1;
        if (parts[0] === "preview" && parts.length === 2 && value(parts[1]) >= 0) {
            win.setPreviewWidth(value(parts[1]));
            return { ok: true, width: win.previewWidth };
        }
        if (parts[0] === "column" && parts.length === 3 && value(parts[2]) >= 0) {
            const col = files.allColumns.find(c => c.key === parts[1]);
            if (!col)
                return { ok: false, error: "unknown column: " + parts[1] + " (" + files.allColumns.map(c => c.key).join(", ") + ")" };
            files.setColWidth(col.key, value(parts[2]));
            return { ok: true, width: files.baseWidth(col) };
        }
        return usage;
    }
    function onOff(arg) {
        return arg === "on" ? true : arg === "off" ? false : undefined;
    }
    readonly property var sortKeys: ({ name: 0, size: 1, modified: 2, kind: 3, created: 4, permissions: 5, owner: 6 })
    function tabCommand(arg) {
        const parts = arg.split("\t");
        const n = parts.length > 1 && /^\d+$/.test(parts[1]) ? parseInt(parts[1]) : -1;
        switch (parts[0]) {
        case "new":
            if (picking)
                return { ok: false, error: "no tabs in the file picker" };
            newTab();
            return parts[1] ? gotoPath(parts[1]) : { ok: true };
        case "close": {
            const i = parts.length > 1 ? n : tabIndex;
            if (i < 0 || i >= tabs.length)
                return { ok: false, error: "no such tab: " + (parts[1] || "") + " (0.." + (tabs.length - 1) + ")" };
            if (tabs.length < 2)
                return { ok: false, error: "last tab (the window stays open)" };
            closeTab(i);
            return { ok: true };
        }
        case "detach": {
            const i = parts.length > 1 ? n : tabIndex;
            if (i < 0 || i >= tabs.length)
                return { ok: false, error: "no such tab: " + (parts[1] || "") + " (0.." + (tabs.length - 1) + ")" };
            if (!detachTab(i))
                return { ok: false, error: "last tab: nothing to detach" };
            return { ok: true };
        }
        case "move": {
            const to = parts.length > 2 && /^\d+$/.test(parts[2]) ? parseInt(parts[2]) : -1;
            if (n < 0 || n >= tabs.length || to < 0 || to >= tabs.length)
                return { ok: false, error: "no such tab (0.." + (tabs.length - 1) + "): tab move FROM TO" };
            moveTab(n, to);
            return { ok: true };
        }
        case "switch":
            if (n < 0 || n >= tabs.length)
                return { ok: false, error: "no such tab: " + (parts[1] || "") + " (0.." + (tabs.length - 1) + ")" };
            switchTab(n);
            return { ok: true };
        }
        return { ok: false, error: "usage: tab new [PATH] | close [N] | switch N | move FROM TO | detach [N]" };
    }
    // Répond au dialogue ouvert (ctl dialog accept|cancel; pour un conflit de nom aussi replace|skip|both): accept = le
    // bouton par défaut, cancel = Échap. Les champs de saisie (renommage, chemin) et le menu contextuel ne se pilotent pas ici.
    function dialogCommand(choice) {
        const name = dialogName();
        if (name === "")
            return { ok: false, error: "no dialog open" };
        if (choice !== "accept" && choice !== "cancel" && !(name === "conflict" && ["replace", "skip", "both"].indexOf(choice) >= 0))
            return { ok: false, error: name === "conflict" ? "usage: dialog accept|cancel|replace|skip|both" : "usage: dialog accept|cancel" };
        const accept = choice !== "cancel";
        switch (name) {
        case "conflict":
            conflictBox.choose(choice === "accept" ? "both" : choice);
            return { ok: true };
        case "confirm":
            confirm.visible = false;
            if (accept)
                confirm.action();
            return { ok: true };
        case "match":
            accept ? matchBox.apply() : (matchBox.visible = false);
            return { ok: true };
        case "bulk":
            accept ? bulkBox.apply() : (bulkBox.visible = false);
            return { ok: true };
        case "compress":
            accept ? compressBox.apply() : (compressBox.visible = false);
            return { ok: true };
        case "info":
        case "about":
        case "password":
        case "share":
        case "connect": {
            if (accept)
                return { ok: false, error: "the " + name + " dialog has no default button for ctl (dialog cancel closes it)" };
            const box = { info: infoCard, about: aboutCard, password: passwordBox, share: shareBox, connect: connectBox }[name];
            box.visible = false;
            return { ok: true };
        }
        }
        return { ok: false, error: "the " + name + " field is answered by the user (or use action rename NAME, path PATH...)" };
    }
    // Dialogue ouvert, sinon ""
    function dialogName() {
        const open = { menu: menu.visible, confirm: confirm.visible, info: infoCard.visible, match: matchBox.visible, bulk: bulkBox.visible, conflict: conflictBox.visible, compress: compressBox.visible, password: passwordBox.visible, share: shareBox.visible, connect: connectBox.visible, about: aboutCard.visible };
        for (const k in open)
            if (open[k])
                return k;
        return files.editing ? "rename" : pathBar.editing ? "path" : "";
    }
    // État de la fenêtre pour ctl state
    function stateObject() {
        const tabs = win.tabs.slice();
        tabs[tabIndex] = { path: folder.shown };
        return {
            ok: true,
            state: {
                pid: Fili.pid(),
                location: folder.shown,
                results: folder.results,
                in_trash: folder.inTrash,
                tab: tabIndex,
                tabs: tabs.map(t => t.path),
                selection: Object.keys(files.selected),
                view: files.tiles ? "icons" : "list",
                hidden_shown: folder.showHidden,
                filter: folder.filter,
                sort: Object.keys(sortKeys).find(k => sortKeys[k] === folder.sortKey) + (folder.sortDesc ? " desc" : " asc"),
                preview: win.previewOn,
                preview_width: win.previewWidth,
                column_widths: files.colWidths,
                tasks: win.tasks,
                dialog: dialogName(),
                notice: win.notice,
                busy: win.working,
                can_act: win.canAct
            }
        };
    }
    // Commandes de menu avec libellé, raccourci et état (activée maintenant ou non)
    function actionList() {
        return {
            ok: true,
            actions: Object.keys(menuActions).map(k => ({ name: k, label: menuActions[k].text, keys: String(menuActions[k].shortcut || ""), enabled: menuActions[k].enabled }))
        };
    }
    function runCommand(verb, arg, body) {
        switch (verb) {
        case "goto":
            return gotoPath(arg);
        case "reveal":
            return reveal(arg);
        case "show":
            folder.showResults(arg, body);
            files.select(0);
            files.focusList();
            return { ok: true };
        case "action": {
            const at = arg.indexOf("\t");
            return at < 0 ? runAction(arg) : runAction(arg.slice(0, at), arg.slice(at + 1));
        }
        case "state":
            return stateObject();
        case "actions":
            return actionList();
        case "select": {
            const at = arg.indexOf("\t");
            return selectCommand(at < 0 ? arg : arg.slice(0, at), at < 0 ? "" : arg.slice(at + 1), body);
        }
        case "tab":
            return tabCommand(arg);
        case "dialog":
            return dialogCommand(arg);
        case "width":
            return widthCommand(arg);
        case "view":
            if (arg !== "list" && arg !== "icons")
                return { ok: false, error: "usage: view list|icons" };
            files.setViewMode(arg === "icons" ? "grid" : "list");
            return { ok: true };
        case "hidden": {
            const want = onOff(arg);
            if (want === undefined)
                return { ok: false, error: "usage: hidden on|off" };
            if (folder.showHidden !== want)
                win.show(folder.toggleHidden(files.currentIndex));
            return { ok: true };
        }
        case "preview": {
            const want = onOff(arg);
            if (want === undefined || picking)
                return { ok: false, error: picking ? "no preview in the file picker" : "usage: preview on|off" };
            if (previewOn !== want)
                togglePreview();
            return { ok: true };
        }
        case "sort": {
            const parts = arg.split("\t");
            const key = sortKeys[parts[0]];
            const dir = parts[1] || "asc";
            if (key === undefined || (dir !== "asc" && dir !== "desc"))
                return { ok: false, error: "usage: sort name|size|modified|kind|created|permissions|owner [asc|desc]" };
            // sortBy bascule le sens si la colonne est déjà celle du tri, sinon trie en croissant
            if (folder.sortKey !== key || folder.sortDesc !== (dir === "desc"))
                win.show(folder.sortBy(key, files.currentIndex));
            if (folder.sortDesc !== (dir === "desc"))
                win.show(folder.sortBy(key, files.currentIndex));
            return { ok: true };
        }
        }
        return { ok: false, error: "unknown command: " + verb };
    }

    // Pas de barre de menus dans la fenêtre: le menu vit dans la fili-bar (repli appmenu/fili-file.json, commandes ci-dessus)

    // Pas de raccourcis de fichiers pendant une saisie (renommage, chemin) ni sous la confirmation
    readonly property bool typing: menu.visible || nameInput.activeFocus || files.editing || pathBar.editing || searchInput.activeFocus || confirm.visible || infoCard.visible || matchBox.visible || bulkBox.visible || conflictBox.visible || compressBox.visible || passwordBox.visible || shareBox.visible || connectBox.visible || aboutCard.visible
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
                list.push({ id: "compress", label: Fili.tr("Compress…") });
                list.push({ id: "share", label: Fili.tr("Share…"), enabled: t.some(p => !folder.pathIsDir(p)) });
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
    function openPathMenu(path, x, y, crumb, removable, frequent) {
        menuPath = path;
        const list = [{ id: "gopath", label: Fili.tr("Open") }, { separator: true }];
        if (removable)
            list.push({ id: "remfav", label: Fili.tr("Remove from Favorites") });
        else if (frequent)
            list.push({ id: "remfreq", label: Fili.tr("Remove from Recents") });
        else if (crumb && !Fili.isFavorite(path))
            list.push({ id: "addfav", label: Fili.tr("Add to Favorites") });
        list.push({ id: "copypath", label: Fili.tr("Copy Full Path") });
        // Le lieu « Corbeille » de la barre latérale: vider sans y entrer
        if (path === Fili.trashPath() && !crumb)
            list.push({ separator: true }, { id: "empty", label: Fili.tr("Empty Trash"), enabled: trashFull });
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
        else if (id === "remfreq")
            Fili.forgetVisit(menuPath);
        else if (id === "open")
            win.show(folder.activate(files.currentIndex));
        else if (id.startsWith("app:"))
            folder.openWith(id.slice(4), files.targets()[0]);
        else if (id === "quicklook")
            folder.quicklook(files.targets()[0]);
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
            aExtract.trigger();
        else if (id === "compress")
            aCompress.trigger();
        else if (id === "share")
            aShare.trigger();
        else if (id === "newfolder")
            win.newFolder();
        else if (id === "empty")
            win.askEmpty();
        files.focusList();
    }
    Shortcut { sequence: "Escape"; enabled: !win.typing && folder.busy; onActivated: folder.cancel() }
    Shortcut { sequence: "Escape"; enabled: win.picking && !win.typing && !folder.busy; onActivated: Fili.pickCancel() }
    Shortcut { sequence: "Escape"; enabled: win.nearby && !win.picking && !win.typing; onActivated: win.nearby = false }
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
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
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
            border.width: FiliTheme.borderWidth
            border.color: searchInput.activeFocus ? FiliTheme.voile : FiliTheme.border
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
        color: FiliTheme.border
    }

    Sidebar {
        id: sidebar
        anchors { top: toolbarLine.bottom; bottom: pickbar.top; left: parent.left }
        width: FiliTheme.sidebarWidth
        current: win.nearby && !win.picking ? "nearby:" : win.networkView ? "network:" : folder.shown
        onOpen: p => win.open(p)
        onDropped: (urls, dest, copy) => folder.dropItems(urls, dest, copy)
        onContext: (path, removable, frequent, x, y) => win.openPathMenu(path, x, y, false, removable, frequent)
    }
    Rectangle {
        id: sideLine
        anchors { top: toolbarLine.bottom; bottom: pickbar.top; left: sidebar.right }
        width: 1
        color: FiliTheme.border
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
                    color: current ? FiliTheme.nuit3 : tabArea.containsMouse || tabDrop.containsDrag ? FiliTheme.nuit2 : "transparent"
                    border.width: tabDrop.containsDrag ? 1 : 0
                    border.color: FiliTheme.voile
                    // Glisser l'onglet pour le déplacer: il suit le curseur, la place se décide au relâchement
                    property real dragDx: 0
                    property bool moving: false
                    property bool leaving: false
                    z: moving ? 10 : 0
                    opacity: leaving ? 0.45 : moving ? 0.85 : 1
                    transform: Translate { x: dragDx }
                    // Fichiers déposés sur un onglet: ils vont dans son dossier. Le survol un instant l'affiche (la cible se
                    // choisit alors dans la liste). Pas sur l'onglet affiché: ce serait son propre dossier.
                    DropArea {
                        id: tabDrop
                        anchors.fill: parent
                        enabled: !current
                        keys: ["text/uri-list"]
                        onEntered: spring.restart()
                        onExited: spring.stop()
                        onDropped: d => {
                            spring.stop();
                            folder.dropItems(d.urls.map(u => u.toString()).join("\n"), path, d.proposedAction === Qt.CopyAction);
                            d.acceptProposedAction();
                        }
                        Timer {
                            id: spring
                            interval: 700
                            onTriggered: win.switchTab(index)
                        }
                    }
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
                        property real pressX: 0
                        property bool wasDrag: false
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.LeftButton | Qt.MiddleButton
                        onPressed: e => {
                            pressX = mapToItem(tabBar, e.x, 0).x;
                            wasDrag = false;
                        }
                        onPositionChanged: e => {
                            if (!pressed || win.tabs.length < 2)
                                return;
                            const dx = mapToItem(tabBar, e.x, 0).x - pressX;
                            if (!moving && Math.abs(dx) < 8)
                                return;
                            moving = true;
                            wasDrag = true;
                            // Reste dans la barre: du premier au dernier onglet
                            const slot = parent.width + FiliTheme.space1;
                            dragDx = Math.max(-index * slot, Math.min((win.tabs.length - 1 - index) * slot, dx));
                            // Loin de la barre: il deviendra une fenêtre (à demi transparent pour le dire)
                            leaving = outside(e);
                        }
                        // Relâché loin de la barre d'onglets (au-dessus, sous la barre ou hors de la fenêtre): l'onglet devient une fenêtre
                        function outside(e) {
                            const p = mapToItem(null, e.x, e.y);
                            return p.y < -24 || p.y > tabBar.y + tabBar.height + 48 || p.x < -24 || p.x > win.width + 24;
                        }
                        onReleased: e => {
                            if (moving) {
                                const out = outside(e);
                                const to = Math.max(0, Math.min(win.tabs.length - 1, index + Math.round(dragDx / (parent.width + FiliTheme.space1))));
                                moving = false;
                                leaving = false;
                                dragDx = 0;
                                if (out)
                                    win.detachTab(index);
                                else
                                    win.moveTab(index, to);
                            }
                        }
                        onCanceled: {
                            moving = false;
                            leaving = false;
                            dragDx = 0;
                        }
                        onClicked: e => {
                            if (wasDrag)
                                return;
                            e.button === Qt.MiddleButton ? win.closeTab(index) : win.switchTab(index);
                        }
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
            text: win.networkView ? (files.active.count === 0 ? Fili.tr("No server connected. Connect to Server… (Ctrl+K) adds one.") : Fili.tr("Connected servers and shares")) : Fili.tr("Results") + ": " + folder.results
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
    Connections {
        target: folder
        function onChanged() {
            if (win.nearby && folder.shown !== win.nearbyFrom)
                win.nearby = false;
        }
    }
    NearbyView {
        id: nearbyView
        anchors { top: toolbarLine.bottom; bottom: pickbar.top; left: sideLine.right; right: parent.right }
        visible: win.nearby && !win.picking
        z: 5
        onNotice: text => {
            win.notice = text;
            noticeTimer.restart();
        }
        onOpenPath: p => win.open(p)
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
        color: FiliTheme.border
    }
    Preview {
        id: preview
        // Largeur choisie (300 par défaut); la liste garde au moins 320 (en dessous de 160 le panneau disparaît)
        readonly property real room: win.width - FiliTheme.sidebarWidth - 1 - 320
        visible: win.previewOn && !win.picking && room >= 160
        anchors { top: toolbarLine.bottom; bottom: pickbar.top; right: parent.right }
        width: visible ? Math.min(win.previewWidth, room) : 0
        path: {
            const keys = Object.keys(files.selected);
            return keys.length === 1 ? keys[0] : keys.length === 0 && files.currentIndex >= 0 ? folder.pathAt(files.currentIndex) : "";
        }
    }
    // Bord gauche du panneau d'aperçu: le glisser l'élargit (vers la gauche) ou le rétrécit
    MouseArea {
        id: previewGrip
        visible: preview.visible
        property real startX: 0
        property real startWidth: 0
        anchors { top: toolbarLine.bottom; bottom: pickbar.top; right: preview.left; rightMargin: -4 }
        width: 9
        cursorShape: Qt.SplitHCursor
        onPressed: e => {
            startX = mapToItem(null, e.x, 0).x;
            startWidth = preview.width;
        }
        onPositionChanged: e => {
            if (pressed)
                win.setPreviewWidth(startWidth - (mapToItem(null, e.x, 0).x - startX), false);
        }
        onReleased: Fili.setUiState("preview", String(Math.round(win.previewWidth)))
        onDoubleClicked: win.setPreviewWidth(0)
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
            color: FiliTheme.border
        }
        Rectangle {
            visible: win.pickKind === "save"
            anchors { left: parent.left; leftMargin: FiliTheme.margin; verticalCenter: parent.verticalCenter }
            width: 300
            height: 26
            radius: FiliTheme.radiusControl
            color: FiliTheme.nuit2
            border.width: FiliTheme.borderWidth
            border.color: nameInput.activeFocus ? FiliTheme.voile : FiliTheme.border
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
            color: FiliTheme.border
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
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
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
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
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
                    border.width: FiliTheme.borderWidth
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
    // « Partager… »: envoie les fichiers sélectionnés à un appareil Bluetooth appairé ou à un pair LocalSend du réseau local
    // (fili-share). Le script notifie la fin de l'envoi.
    Rectangle {
        id: shareBox
        property var paths: []
        property var protocols: []
        property string proto: ""
        property var peers: []
        property bool loading: false
        readonly property var current: protocols.find(p => p.id === proto) || null
        function open() {
            paths = win.shareable();
            peers = [];
            protocols = [];
            proto = "";
            loading = true;
            visible = true;
            Fili.shareProtocols();
        }
        function choose(id) {
            proto = id;
            peers = [];
            if (current && current.available) {
                loading = true;
                Fili.sharePeers(id, "share");
            } else {
                loading = false;
            }
        }
        function send(peer) {
            Fili.shareSend(proto, peer.id, peer.name, paths.join("\n"));
            visible = false;
            win.notice = Fili.tr("Sending to %1…").replace("%1", peer.name);
            noticeTimer.restart();
        }
        anchors.fill: parent
        visible: false
        z: 200
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        onVisibleChanged: if (!visible) files.focusList()
        Keys.onEscapePressed: visible = false
        Connections {
            target: Fili
            function onShareProtocolsReady(json) {
                if (!shareBox.visible)
                    return;
                const all = JSON.parse(json).filter(p => p.id === "bluetooth" || p.id === "localsend");
                shareBox.protocols = all;
                const first = all.find(p => p.available) || all[0];
                if (first)
                    shareBox.choose(first.id);
                else
                    shareBox.loading = false;
            }
            function onSharePeersReady(p, tag, json) {
                if (tag !== "share" || !shareBox.visible || p !== shareBox.proto)
                    return;
                shareBox.peers = JSON.parse(json);
                shareBox.loading = false;
            }
        }
        MouseArea {
            anchors.fill: parent
            onClicked: shareBox.visible = false
        }
        Rectangle {
            anchors.centerIn: parent
            width: 380
            height: shareCard.implicitHeight + 2 * FiliTheme.space4
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
            MouseArea { anchors.fill: parent }
            Column {
                id: shareCard
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space4 }
                spacing: FiliTheme.space3
                Text {
                    width: parent.width
                    elide: Text.ElideMiddle
                    text: shareBox.paths.length === 1 ? Fili.tr("Share “%1”").replace("%1", shareBox.paths[0].split("/").pop())
                                                       : Fili.tr("Share %1 files").replace("%1", shareBox.paths.length)
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: FiliTheme.weightAccent
                    color: FiliTheme.brume
                }
                Row {
                    spacing: FiliTheme.space2
                    Repeater {
                        model: shareBox.protocols
                        TextButton {
                            required property var modelData
                            label: modelData.label
                            primary: modelData.id === shareBox.proto
                            opacity: modelData.available ? 1 : 0.5
                            onClicked: shareBox.choose(modelData.id)
                        }
                    }
                }
                Text {
                    visible: shareBox.current !== null && !shareBox.current.available
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: shareBox.current ? shareBox.current.why : ""
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
                Text {
                    visible: shareBox.current !== null && shareBox.current.available && (shareBox.loading || shareBox.peers.length === 0)
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: shareBox.loading ? Fili.tr("Searching…")
                          : shareBox.proto === "bluetooth" ? Fili.tr("No paired device. Pair one in the Bluetooth panel.")
                                                             : Fili.tr("No device found on this network. LocalSend must be open on the other device.")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
                Repeater {
                    model: shareBox.loading ? [] : shareBox.peers
                    Rectangle {
                        required property var modelData
                        width: shareCard.width
                        height: 26
                        radius: FiliTheme.radiusControl
                        color: peerArea.containsMouse ? FiliTheme.nuit3 : "transparent"
                        Text {
                            anchors { left: parent.left; leftMargin: FiliTheme.space2; right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                            text: modelData.name
                            elide: Text.ElideRight
                            font.family: FiliTheme.font
                            font.pixelSize: FiliTheme.size
                            color: FiliTheme.brume
                        }
                        MouseArea {
                            id: peerArea
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: shareBox.send(modelData)
                        }
                    }
                }
                Row {
                    anchors.right: parent.right
                    TextButton { label: Fili.tr("Cancel"); onClicked: shareBox.visible = false }
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
            border.width: FiliTheme.borderWidth
            border.color: input.activeFocus ? FiliTheme.voile : FiliTheme.border
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
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
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
        border.width: FiliTheme.borderWidth
        border.color: input.activeFocus ? FiliTheme.voile : FiliTheme.border
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
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
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
                    border.width: FiliTheme.borderWidth
                    border.color: FiliTheme.border
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
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
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
    // Panneau de compression (menu « Compresser… »): nom, format, niveau, fichiers cachés. Les choix sont gardés pour la session.
    component ChoiceRow: Item {
        property var options: []
        property string current
        width: parent ? parent.width : 100
        height: 22
        activeFocusOnTab: true
        function step(d) {
            const i = options.findIndex(o => o.id === current);
            current = options[(i + d + options.length) % options.length].id;
        }
        Keys.onLeftPressed: step(-1)
        Keys.onRightPressed: step(1)
        Keys.onReturnPressed: compressBox.apply()
        Keys.onEnterPressed: compressBox.apply()
        Keys.onEscapePressed: compressBox.visible = false
        Row {
            spacing: FiliTheme.space1
            Repeater {
                model: options
                TextButton {
                    required property var modelData
                    label: modelData.label
                    primary: modelData.id === current
                    onClicked: { current = modelData.id; parent.parent.forceActiveFocus(); }
                }
            }
        }
        FocusRing {}
    }
    component PasswordField: Rectangle {
        property alias text: input.text
        property alias field: input
        property string label
        signal accepted()
        signal cancelled()
        width: parent ? parent.width : 100
        height: 26
        radius: FiliTheme.radiusControl
        color: FiliTheme.nuit
        border.width: FiliTheme.borderWidth
        border.color: input.activeFocus ? FiliTheme.voile : FiliTheme.border
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
            echoMode: TextInput.Password
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.size
            color: FiliTheme.brume
            selectByMouse: true
            onAccepted: parent.accepted()
            Keys.onEscapePressed: parent.cancelled()
        }
    }
    // Mot de passe d'une archive à extraire (demandé quand fili-archive dit qu'elle est protégée)
    Rectangle {
        id: passwordBox
        property string paths: ""
        property bool wrong: false
        function open(list, bad) {
            paths = list;
            wrong = bad;
            pwField.text = "";
            visible = true;
        }
        function apply() {
            if (pwField.text === "")
                return;
            visible = false;
            folder.extract(paths, pwField.text);
        }
        anchors.fill: parent
        visible: false
        z: 200
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        onVisibleChanged: if (visible) pwField.field.forceActiveFocus(); else files.focusList()
        Connections {
            target: folder
            function onPasswordNeeded(list, bad) { passwordBox.open(list, bad); }
        }
        MouseArea {
            anchors.fill: parent
            onClicked: passwordBox.visible = false
        }
        Rectangle {
            anchors.centerIn: parent
            width: 380
            height: pwCard.implicitHeight + 2 * FiliTheme.space4
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
            Column {
                id: pwCard
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space4 }
                spacing: FiliTheme.space3
                Text {
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: Fili.tr("“%1” is protected by a password").replace("%1", passwordBox.paths.split("\n")[0].split("/").pop())
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: FiliTheme.weightAccent
                    color: FiliTheme.brume
                }
                Text {
                    visible: passwordBox.wrong
                    width: parent.width
                    text: Fili.tr("Wrong password. Try again.")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
                PasswordField {
                    id: pwField
                    label: Fili.tr("Password")
                    onAccepted: passwordBox.apply()
                    onCancelled: passwordBox.visible = false
                }
                Row {
                    spacing: FiliTheme.space2
                    anchors.right: parent.right
                    TextButton { label: Fili.tr("Cancel"); onClicked: passwordBox.visible = false }
                    TextButton { label: Fili.tr("Extract"); primary: true; onClicked: passwordBox.apply() }
                }
            }
        }
    }
    Rectangle {
        id: compressBox
        property string paths: ""
        property string format: "zip"
        property string level: "normal"
        property bool skipHidden: false
        readonly property bool canEncrypt: format === "zip" || format === "7z"
        function open(list) {
            paths = list;
            passField2.text = "";
            nameField.text = folder.archiveName(paths);
            visible = true;
        }
        function apply() {
            if (nameField.text.trim() === "")
                return;
            visible = false;
            folder.compress(paths, nameField.text, format, level, skipHidden, canEncrypt ? passField2.text : "");
        }
        anchors.fill: parent
        visible: false
        z: 200
        color: FiliTheme.alpha(FiliTheme.nuitProfond, 0.7)
        onVisibleChanged: if (visible) { nameField.forceActiveFocus(); nameField.selectAll(); } else files.focusList()
        MouseArea {
            anchors.fill: parent
            onClicked: compressBox.visible = false
        }
        Rectangle {
            anchors.centerIn: parent
            width: 380
            height: compressCard.implicitHeight + 2 * FiliTheme.space4
            radius: FiliTheme.radiusWindow
            color: FiliTheme.nuit2
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
            Column {
                id: compressCard
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: FiliTheme.space4 }
                spacing: FiliTheme.space3
                Text {
                    width: parent.width
                    text: Fili.tr("Compress")
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
                    border.width: FiliTheme.borderWidth
                    border.color: nameField.activeFocus ? FiliTheme.voile : FiliTheme.border
                    TextInput {
                        id: nameField
                        anchors { fill: parent; leftMargin: FiliTheme.space2; rightMargin: FiliTheme.space2 }
                        verticalAlignment: TextInput.AlignVCenter
                        clip: true
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.size
                        color: FiliTheme.brume
                        selectByMouse: true
                        KeyNavigation.tab: formatRow
                        onAccepted: compressBox.apply()
                        Keys.onEscapePressed: compressBox.visible = false
                    }
                }
                Text {
                    text: Fili.tr("Format")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
                ChoiceRow {
                    id: formatRow
                    KeyNavigation.tab: levelRow
                    options: [
                        { id: "zip", label: "zip" }, { id: "7z", label: "7z" }, { id: "tar.gz", label: "tar.gz" },
                        { id: "tar.xz", label: "tar.xz" }, { id: "tar.zst", label: "tar.zst" }
                    ]
                    current: compressBox.format
                    onCurrentChanged: compressBox.format = current
                }
                Text {
                    text: Fili.tr("Compression")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
                ChoiceRow {
                    id: levelRow
                    KeyNavigation.tab: hiddenRow
                    options: [
                        { id: "fast", label: Fili.tr("Faster") }, { id: "normal", label: Fili.tr("Balanced") },
                        { id: "best", label: Fili.tr("Smaller") }
                    ]
                    current: compressBox.level
                    onCurrentChanged: compressBox.level = current
                }
                Item {
                    id: hiddenRow
                    width: parent.width
                    height: 22
                    activeFocusOnTab: true
                    KeyNavigation.tab: compressBox.canEncrypt ? passField2.field : nameField
                    function flip() { compressBox.skipHidden = !compressBox.skipHidden; }
                    Keys.onSpacePressed: flip()
                    Keys.onReturnPressed: compressBox.apply()
                    Keys.onEnterPressed: compressBox.apply()
                    Keys.onEscapePressed: compressBox.visible = false
                    Rectangle {
                        id: box
                        width: 14
                        height: 14
                        anchors.verticalCenter: parent.verticalCenter
                        radius: 3
                        color: FiliTheme.nuit
                        border.width: FiliTheme.borderWidth
                        border.color: compressBox.skipHidden ? FiliTheme.voile : FiliTheme.border
                        Text {
                            anchors.centerIn: parent
                            visible: compressBox.skipHidden
                            text: "✓"
                            font.family: FiliTheme.font
                            font.pixelSize: FiliTheme.sizeList
                            color: FiliTheme.voile
                        }
                    }
                    Text {
                        anchors { left: box.right; leftMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                        text: Fili.tr("Skip hidden files")
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.size
                        color: FiliTheme.brume
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: { hiddenRow.flip(); hiddenRow.forceActiveFocus(); }
                    }
                    FocusRing {}
                }
                PasswordField {
                    id: passField2
                    visible: compressBox.canEncrypt
                    label: Fili.tr("Password (optional)")
                    onAccepted: compressBox.apply()
                    onCancelled: compressBox.visible = false
                    field.KeyNavigation.tab: nameField
                }
                Text {
                    visible: !compressBox.canEncrypt
                    width: parent.width
                    wrapMode: Text.Wrap
                    text: Fili.tr("Only zip and 7z archives can have a password.")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume2
                }
                Row {
                    spacing: FiliTheme.space2
                    anchors.right: parent.right
                    TextButton { label: Fili.tr("Cancel"); onClicked: compressBox.visible = false }
                    TextButton { label: Fili.tr("Compress"); primary: true; onClicked: compressBox.apply() }
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
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
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
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
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
                                border.width: FiliTheme.borderWidth
                                border.color: on ? FiliTheme.voile : FiliTheme.border
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
            border.width: FiliTheme.borderWidth
            border.color: FiliTheme.border
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
