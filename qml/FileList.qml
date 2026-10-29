// Vue du dossier, en liste (colonnes Nom, Taille, Modifié...; un clic sur un en-tête trie, un second inverse l'ordre) ou en
// grille d'icônes de 64 px (nom sous l'icône, deux lignes au plus); même délégué, mêmes sélection et raccourcis.
// Une ligne sélectionnée: fond nuit3 et nom en voile (brume quand la fenêtre n'est pas active); survol: fond nuit3.
// Sélection multiple: Ctrl+clic ajoute ou retire, Maj+clic ou Maj+flèches étendent, Ctrl+A tout. Clavier: flèches,
// Début/Fin, Page préc./suiv., Entrée ou Ctrl+↓ ouvre, Retour arrière ou Alt+↑ remonte, Espace aperçu rapide,
// lettres = aller au nom qui commence ainsi. F2 renomme sur place (Entrée valide, Échap annule). Glisser les lignes
// sélectionnées: text/uri-list, déposable sur un dossier de la liste, de la barre latérale ou dans une autre fenêtre.
import QtQuick
import Fili.Files

Rectangle {
    id: view
    required property FolderModel folder
    // Vue affichée: "list" ou "grid" (choix gardé dans l'état de l'utilisateur)
    property string viewMode: Fili.viewMode()
    readonly property bool tiles: viewMode === "grid"
    readonly property Item active: tiles ? grid : list
    readonly property int currentIndex: active.currentIndex
    function setViewMode(mode) {
        if (mode === viewMode)
            return;
        const i = active.currentIndex;
        Fili.setViewMode(mode);
        viewMode = mode;
        active.currentIndex = i;
        if (i >= 0)
            active.positionViewAtIndex(i, ListView.Contain);
        focusList();
    }
    property bool windowActive: true
    signal openRow(int row)
    signal goUp()
    signal dropped(string urls, string dest, bool copy)
    // Clic droit (ou touche Menu): position dans cette vue; row = -1 hors des lignes
    signal context(real x, real y)

    color: FiliTheme.nuit

    // Lignes sélectionnées: chemin -> true. anchor: ligne de départ des sélections par plage.
    property var selected: ({})
    property int anchor: 0
    // Ligne en cours de renommage (-1: aucune)
    // Miniatures PDF prêtes depuis l'ouverture du dossier: chemin -> true
    property var ready: ({})
    Connections {
        target: folder
        function onThumbReady(path) {
            const r = Object.assign({}, view.ready);
            r[path] = true;
            view.ready = r;
        }
    }
    // Description de la sélection (barre d'état), vide si rien n'est sélectionné
    readonly property string info: Object.keys(selected).length > 0 ? folder.describe(Object.keys(selected).join("\n")) : ""
    property int renaming: -1
    readonly property bool editing: renaming >= 0

    function pick(row) {
        const s = {};
        if (row >= 0 && row < active.count)
            s[folder.pathAt(row)] = true;
        selected = s;
        anchor = row;
    }
    // Une seule ligne sélectionnée et courante
    function select(row) {
        if (row < 0)
            return;
        active.currentIndex = Math.min(row, active.count - 1);
        if (active.currentIndex >= 0)
            active.positionViewAtIndex(active.currentIndex, ListView.Contain);
        pick(active.currentIndex);
    }
    function extendTo(row) {
        if (active.count === 0)
            return;
        active.currentIndex = Math.max(0, Math.min(row, active.count - 1));
        active.positionViewAtIndex(active.currentIndex, ListView.Contain);
        const s = {};
        const from = Math.max(0, Math.min(anchor, active.currentIndex)), to = Math.max(anchor, active.currentIndex);
        for (let i = from; i <= to; i++)
            s[folder.pathAt(i)] = true;
        selected = s;
    }
    function toggle(row) {
        const s = Object.assign({}, selected);
        const p = folder.pathAt(row);
        if (s[p])
            delete s[p];
        else
            s[p] = true;
        selected = s;
        anchor = row;
        active.currentIndex = row;
    }
    function selectAll() {
        const s = {};
        for (let i = 0; i < active.count; i++)
            s[folder.pathAt(i)] = true;
        selected = s;
    }
    // Chemins visés par une commande: la sélection, sinon la ligne courante
    function targets() {
        const keys = Object.keys(selected);
        if (keys.length > 0)
            return keys;
        return active.currentIndex >= 0 ? [folder.pathAt(active.currentIndex)] : [];
    }
    function targetsText() {
        return targets().join("\n");
    }
    function startRename() {
        if (active.currentIndex >= 0)
            renaming = active.currentIndex;
    }
    // Le dossier a changé sous la vue: ne garder que ce qui existe encore
    function reloaded() {
        const keep = {};
        for (const p of Object.keys(selected))
            if (folder.has(p))
                keep[p] = true;
        selected = keep;
        if (active.currentIndex < 0 && active.count > 0)
            select(0);
    }
    function dragUris(path) {
        return folder.uris(selected[path] ? Object.keys(selected).join("\n") : path);
    }
    function focusList() {
        active.forceActiveFocus();
    }

    // Colonnes après le nom: affichées selon le choix de l'utilisateur (clic droit sur un en-tête), dans cet ordre
    readonly property var allColumns: [
        { key: "size", sort: 1, role: "size", label: Fili.tr("Size"), width: 96, align: Text.AlignRight },
        { key: "modified", sort: 2, role: "modified", label: Fili.tr("Modified"), width: 160, align: Text.AlignLeft },
        { key: "kind", sort: 3, role: "kind", label: Fili.tr("Kind"), width: 150, align: Text.AlignLeft },
        { key: "created", sort: 4, role: "created", label: Fili.tr("Created"), width: 160, align: Text.AlignLeft },
        { key: "permissions", sort: 5, role: "perms", label: Fili.tr("Permissions"), width: 110, align: Text.AlignLeft },
        { key: "owner", sort: 6, role: "owner", label: Fili.tr("Owner"), width: 100, align: Text.AlignLeft }
    ]
    property var enabledKeys: Fili.columns().split(",")
    readonly property var cols: allColumns.filter(c => enabledKeys.indexOf(c.key) >= 0)
    readonly property int colsWidth: cols.reduce((a, c) => a + c.width, 0)
    // Fenêtre étroite: les colonnes se resserrent pour garder 200 px au nom (jusqu'à 35 % de leur largeur)
    readonly property real colScale: colsWidth > 0 ? Math.max(0.35, Math.min(1, (view.width - 2 * FiliTheme.space2 - 200) / colsWidth)) : 1
    function colWidth(c) {
        return Math.round(c.width * colScale);
    }
    function setColumns(keys) {
        Fili.setColumns(keys.join(","));
        enabledKeys = keys;
    }
    signal headerContext(real x, real y)
    readonly property int pad: FiliTheme.space3
    // Début du nom dans une ligne: marge, icône, écart
    readonly property int nameInset: FiliTheme.space2 + FiliTheme.iconSize + FiliTheme.space2
    readonly property int cellPad: FiliTheme.space3

    // En-têtes de colonnes: petit 11, brume2; colonne de tri en brume avec son sens
    Row {
        id: header
        visible: !view.tiles
        x: FiliTheme.space2
        width: view.width - 2 * FiliTheme.space2
        height: 26
        Repeater {
            model: [{ key: "name", sort: 0, label: Fili.tr("Name"), width: 0, align: Text.AlignLeft }].concat(view.cols)
            delegate: Item {
                id: col
                required property var modelData
                required property int index
                readonly property bool active: view.folder.sortKey === modelData.sort
                width: index === 0 ? header.width - view.colsWidth * view.colScale : view.colWidth(modelData)
                height: header.height
                Text {
                    anchors { fill: parent; leftMargin: col.index === 0 ? view.nameInset : view.cellPad; rightMargin: view.cellPad }
                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: col.modelData.align
                    elide: Text.ElideRight
                    text: col.modelData.label + (col.active ? (view.folder.sortDesc ? "  ↓" : "  ↑") : "")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeSmall
                    color: col.active ? FiliTheme.brume : FiliTheme.brume2
                }
                Rectangle {
                    visible: col.index > 0
                    anchors { left: parent.left; verticalCenter: parent.verticalCenter }
                    width: 1
                    height: 14
                    color: FiliTheme.nuit4
                }
                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: e => {
                        if (e.button === Qt.RightButton) {
                            const p = mapToItem(view, e.x, e.y);
                            view.headerContext(p.x, p.y);
                        } else
                            view.select(view.folder.sortBy(col.modelData.sort, active.currentIndex));
                    }
                }
            }
        }
    }
    Rectangle {
        visible: !view.tiles
        anchors.top: header.bottom
        width: parent.width
        height: 1
        color: FiliTheme.nuit4
    }

    // Saisie au clavier: lettres tapées à moins d'une seconde d'intervalle
    property string typed: ""
    Timer { id: typedReset; interval: 1000; onTriggered: view.typed = "" }

    // Touches communes aux deux vues; perRow: nombre de colonnes de la grille (1 en liste)
    function handleKey(e, perRow) {
        Fili.keyNav = true;
        const ctrl = e.modifiers & Qt.ControlModifier, alt = e.modifiers & Qt.AltModifier, shift = e.modifiers & Qt.ShiftModifier;
        const count = active.count, cur = active.currentIndex;
        const rowsPerPage = Math.max(1, Math.floor(active.height / (tiles ? grid.cellHeight : FiliTheme.rowHeight)) - 1);
        const move = row => shift ? view.extendTo(row) : view.select(Math.max(0, Math.min(row, count - 1)));
        if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter || (e.key === Qt.Key_Down && (ctrl || alt)))
            view.openRow(cur);
        else if (e.key === Qt.Key_Backspace || (e.key === Qt.Key_Up && (ctrl || alt)))
            view.goUp();
        else if (e.key === Qt.Key_Up)
            move(cur - perRow);
        else if (e.key === Qt.Key_Down)
            move(cur + perRow);
        else if (e.key === Qt.Key_Left && tiles)
            move(cur - 1);
        else if (e.key === Qt.Key_Right && tiles)
            move(cur + 1);
        else if (e.key === Qt.Key_Menu) {
            const it = active.itemAtIndex(cur);
            const p = it ? it.mapToItem(view, FiliTheme.space4, it.height) : Qt.point(FiliTheme.space6, FiliTheme.space6);
            view.context(p.x, p.y);
        } else if (e.key === Qt.Key_Space && view.typed === "")
            view.folder.quicklook(cur);
        else if (e.key === Qt.Key_Home)
            move(0);
        else if (e.key === Qt.Key_End)
            move(count - 1);
        else if (e.key === Qt.Key_PageDown)
            move(cur + rowsPerPage * perRow);
        else if (e.key === Qt.Key_PageUp)
            move(cur - rowsPerPage * perRow);
        else if (e.text.length === 1 && e.text >= " " && !ctrl && !alt) {
            view.typed += e.text;
            typedReset.restart();
            view.select(view.folder.find(view.typed));
        } else
            return;
        e.accepted = true;
    }

    ListView {
        id: list
        visible: !view.tiles
        anchors { top: header.bottom; topMargin: 1 + 2; left: parent.left; right: parent.right; bottom: parent.bottom }
        clip: true
        // Le ListView impose x = 0 à ses délégués: la marge des lignes est celle du contenu
        leftMargin: FiliTheme.space2
        rightMargin: FiliTheme.space2
        model: view.folder
        focus: true
        activeFocusOnTab: true
        keyNavigationEnabled: false
        highlightMoveDuration: 0
        boundsBehavior: Flickable.StopAtBounds
        reuseItems: true
        KineticScroll {}

        MouseArea {
            id: bgArea
            z: -2
            width: list.width
            height: Math.max(list.height, list.contentHeight)
            acceptedButtons: Qt.RightButton
            onPressed: e => {
                view.pick(-1);
                const p = bgArea.mapToItem(view, e.x, e.y);
                view.context(p.x, p.y);
            }
        }
        // Déposer dans le dossier affiché (hors des lignes de dossier)
        DropArea {
            z: -1
            x: 0
            y: 0
            width: list.width
            height: Math.max(list.height, list.contentHeight)
            onDropped: drop => {
                view.dropped(drop.urls.map(u => u.toString()).join("\n"), view.folder.path, drop.proposedAction === Qt.CopyAction);
                drop.acceptProposedAction();
            }
        }

        Keys.onPressed: e => view.handleKey(e, 1)

        delegate: fileDelegate
    }

    GridView {
        id: grid
        visible: view.tiles
        anchors { top: parent.top; topMargin: FiliTheme.space2; left: parent.left; right: parent.right; bottom: parent.bottom }
        clip: true
        cellWidth: 104
        cellHeight: 112
        readonly property int columns: Math.max(1, Math.floor((width - 2 * FiliTheme.space4) / cellWidth))
        // Grille alignée à gauche (le GridView pose les cellules à partir de la marge du contenu)
        leftMargin: FiliTheme.space4
        rightMargin: FiliTheme.space4
        model: view.folder
        activeFocusOnTab: true
        keyNavigationEnabled: false
        highlightMoveDuration: 0
        boundsBehavior: Flickable.StopAtBounds
        reuseItems: true
        KineticScroll {}

        MouseArea {
            z: -2
            width: grid.width
            height: Math.max(grid.height, grid.contentHeight)
            acceptedButtons: Qt.RightButton
            onPressed: e => {
                view.pick(-1);
                const p = mapToItem(view, e.x, e.y);
                view.context(p.x, p.y);
            }
        }
        DropArea {
            z: -1
            width: grid.width
            height: Math.max(grid.height, grid.contentHeight)
            onDropped: drop => {
                view.dropped(drop.urls.map(u => u.toString()).join("\n"), view.folder.path, drop.proposedAction === Qt.CopyAction);
                drop.acceptProposedAction();
            }
        }

        Keys.onPressed: e => view.handleKey(e, columns)

        delegate: fileDelegate
    }

    Component {
        id: fileDelegate
        Rectangle {
            id: row
            required property int index
            required property string name
            required property string path
            required property bool isDir
            required property var model
            required property string icon
            required property bool hidden
            required property string thumb
            required property bool thumbReady
            required property bool isImage
            readonly property bool tile: view.tiles
            readonly property bool current: index === view.currentIndex
            readonly property bool sel: view.selected[path] === true
            readonly property bool editing: view.renaming === index
            readonly property bool stripe: !tile && index % 2 === 1
            width: tile ? grid.cellWidth - FiliTheme.space2 : view.width - 2 * FiliTheme.space2
            height: tile ? grid.cellHeight - FiliTheme.space2 : FiliTheme.rowHeight
            radius: tile ? FiliTheme.radiusControl + 2 : FiliTheme.radiusControl
            color: sel || drop.containsDrag ? FiliTheme.nuit3 : area.containsMouse ? FiliTheme.nuit2 : stripe ? FiliTheme.alpha(FiliTheme.nuit2, 0.55) : "transparent"
            border.width: drop.containsDrag ? 1 : current && view.active.activeFocus && Fili.keyNav ? 1 : 0
            border.color: FiliTheme.voile
            readonly property color ink: sel && view.windowActive ? FiliTheme.voile : row.hidden ? FiliTheme.brume2 : FiliTheme.brume

            function beginRename() {
                input.text = row.name;
                input.forceActiveFocus();
                const dot = row.name.lastIndexOf(".");
                if (!row.isDir && dot > 0)
                    input.select(0, dot);
                else
                    input.selectAll();
            }
            onEditingChanged: if (editing) beginRename()
            Component.onCompleted: if (editing) beginRename()
            ListView.onReused: if (editing) beginRename()
            GridView.onReused: if (editing) beginRename()

            Image {
                id: img
                visible: !row.tile
                x: FiliTheme.space2
                anchors.verticalCenter: parent.verticalCenter
                width: FiliTheme.iconSize
                height: FiliTheme.iconSize
                sourceSize: Qt.size(FiliTheme.iconSize * 2, FiliTheme.iconSize * 2)
                source: row.icon
                asynchronous: true
                opacity: row.hidden ? 0.6 : 1
                fillMode: Image.PreserveAspectFit
            }
            // Miniature de PDF (première page), par-dessus l'icône du type
            Image {
                anchors.fill: img
                visible: !row.tile && status === Image.Ready
                source: row.thumb !== "" && (row.thumbReady || view.ready[row.path]) ? row.thumb + "?" + (view.ready[row.path] ? 1 : 0) : ""
                sourceSize: Qt.size(FiliTheme.iconSize * 2, FiliTheme.iconSize * 2)
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                cache: false
            }
            // Grille: icône de 64 px (ou miniature) et nom dessous, sur deux lignes au plus
            Image {
                id: tileImg
                visible: row.tile
                x: Math.round((row.width - width) / 2)
                y: FiliTheme.space2
                width: 64
                height: 64
                sourceSize: Qt.size(128, 128)
                source: row.tile ? row.icon : ""
                asynchronous: true
                opacity: row.hidden ? 0.6 : 1
                fillMode: Image.PreserveAspectFit
            }
            Image {
                anchors.fill: tileImg
                visible: row.tile && status === Image.Ready
                source: row.tile && row.thumb !== "" && (row.thumbReady || view.ready[row.path]) ? row.thumb + "?" + (view.ready[row.path] ? 1 : 0) : ""
                sourceSize: Qt.size(128, 128)
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                cache: false
            }
            Text {
                visible: row.tile && !row.editing
                x: FiliTheme.space1
                y: tileImg.y + tileImg.height + FiliTheme.space1
                width: row.width - 2 * FiliTheme.space1
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
                maximumLineCount: 2
                elide: Text.ElideRight
                text: row.name
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: row.ink
            }
            Text {
                anchors { left: img.right; leftMargin: FiliTheme.space2; right: cells.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                visible: !row.tile && !row.editing
                text: row.name
                elide: Text.ElideMiddle
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: row.ink
            }
            // Renommage sur place
            Rectangle {
                x: row.tile ? FiliTheme.space1 : FiliTheme.space2 + FiliTheme.iconSize + FiliTheme.space2 - 4
                y: row.tile ? tileImg.y + tileImg.height + FiliTheme.space1 : 1
                width: row.tile ? row.width - 2 * FiliTheme.space1 : row.width - x - cells.width - FiliTheme.space2
                height: row.tile ? 22 : parent.height - 2
                visible: row.editing
                radius: FiliTheme.radiusControl
                color: FiliTheme.nuit2
                border.width: 1
                border.color: FiliTheme.voile
                TextInput {
                    id: input
                    anchors { fill: parent; leftMargin: 4; rightMargin: 4 }
                    verticalAlignment: TextInput.AlignVCenter
                    clip: true
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.brume
                    selectionColor: FiliTheme.nuit4
                    selectedTextColor: FiliTheme.brume
                    function commit() {
                        if (view.renaming !== row.index)
                            return;
                        view.renaming = -1;
                        if (text !== row.name) {
                            const r = view.folder.rename(row.path, text);
                            if (r >= 0)
                                view.select(r);
                        }
                        view.focusList();
                    }
                    onAccepted: commit()
                    Keys.onEscapePressed: {
                        view.renaming = -1;
                        view.focusList();
                    }
                    onActiveFocusChanged: if (!activeFocus) commit()
                }
            }
            Row {
                id: cells
                visible: !row.tile
                anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                Repeater {
                    model: row.tile ? [] : view.cols
                    delegate: Text {
                        required property var modelData
                        width: view.colWidth(modelData)
                        leftPadding: view.cellPad
                        rightPadding: view.cellPad
                        horizontalAlignment: modelData.align
                        elide: Text.ElideRight
                        text: row.model[modelData.role]
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.sizeList
                        font.features: { "tnum": 1 }
                        color: row.sel && view.windowActive ? FiliTheme.voile : FiliTheme.brume2
                    }
                }
            }

            // Glisser: la sélection (ou la ligne seule) part en text/uri-list
            Item {
                id: ghost
                width: 1
                height: 1
                Drag.active: area.drag.active
                Drag.dragType: Drag.Automatic
                Drag.supportedActions: Qt.CopyAction | Qt.MoveAction
                Drag.proposedAction: Qt.MoveAction
                Drag.mimeData: area.drag.active ? { "text/uri-list": view.dragUris(row.path) } : ({})
            }
            DropArea {
                id: drop
                anchors.fill: parent
                enabled: row.isDir
                onDropped: d => {
                    view.dropped(d.urls.map(u => u.toString()).join("\n"), row.path, d.proposedAction === Qt.CopyAction);
                    d.acceptProposedAction();
                }
            }
            MouseArea {
                id: area
                anchors.fill: parent
                hoverEnabled: true
                drag.target: ghost
                drag.threshold: 8
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                property bool collapse: false
                onPressed: e => {
                    if (e.button === Qt.RightButton) {
                        if (!row.sel) {
                            view.active.currentIndex = row.index;
                            view.pick(row.index);
                        }
                        const p = area.mapToItem(view, e.x, e.y);
                        view.context(p.x, p.y);
                        return;
                    }
                    Fili.keyNav = false;
                    view.renaming = -1;
                    collapse = false;
                    if (e.modifiers & Qt.ControlModifier)
                        view.toggle(row.index);
                    else if (e.modifiers & Qt.ShiftModifier) {
                        view.active.currentIndex = row.index;
                        view.extendTo(row.index);
                    } else if (!row.sel) {
                        view.active.currentIndex = row.index;
                        view.pick(row.index);
                    } else {
                        view.active.currentIndex = row.index;
                        // Sélection multiple: elle se réduit au relâchement s'il n'y a pas eu de glisser
                        collapse = true;
                    }
                    view.active.forceActiveFocus();
                }
                onReleased: {
                    if (collapse && !drag.active)
                        view.pick(row.index);
                    collapse = false;
                }
                onDoubleClicked: view.openRow(row.index)
            }
        }
    }

    // Dossier vide
    Text {
        anchors.centerIn: view.active
        visible: view.active.count === 0
        text: Fili.tr("Empty folder")
        font.family: FiliTheme.font
        font.pixelSize: FiliTheme.size
        color: FiliTheme.brume2
    }
}
