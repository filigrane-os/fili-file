// Vue en liste du dossier (colonnes Nom, Taille, Modifié; un clic sur un en-tête trie, un second inverse l'ordre).
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
    property alias currentIndex: list.currentIndex
    property bool windowActive: true
    signal openRow(int row)
    signal goUp()
    signal dropped(string urls, string dest, bool copy)

    color: FiliTheme.nuit

    // Lignes sélectionnées: chemin -> true. anchor: ligne de départ des sélections par plage.
    property var selected: ({})
    property int anchor: 0
    // Ligne en cours de renommage (-1: aucune)
    property int renaming: -1
    readonly property bool editing: renaming >= 0

    function pick(row) {
        const s = {};
        if (row >= 0 && row < list.count)
            s[folder.pathAt(row)] = true;
        selected = s;
        anchor = row;
    }
    // Une seule ligne sélectionnée et courante
    function select(row) {
        if (row < 0)
            return;
        list.currentIndex = Math.min(row, list.count - 1);
        if (list.currentIndex >= 0)
            list.positionViewAtIndex(list.currentIndex, ListView.Contain);
        pick(list.currentIndex);
    }
    function extendTo(row) {
        if (list.count === 0)
            return;
        list.currentIndex = Math.max(0, Math.min(row, list.count - 1));
        list.positionViewAtIndex(list.currentIndex, ListView.Contain);
        const s = {};
        const from = Math.max(0, Math.min(anchor, list.currentIndex)), to = Math.max(anchor, list.currentIndex);
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
        list.currentIndex = row;
    }
    function selectAll() {
        const s = {};
        for (let i = 0; i < list.count; i++)
            s[folder.pathAt(i)] = true;
        selected = s;
    }
    // Chemins visés par une commande: la sélection, sinon la ligne courante
    function targets() {
        const keys = Object.keys(selected);
        if (keys.length > 0)
            return keys;
        return list.currentIndex >= 0 ? [folder.pathAt(list.currentIndex)] : [];
    }
    function targetsText() {
        return targets().join("\n");
    }
    function startRename() {
        if (list.currentIndex >= 0)
            renaming = list.currentIndex;
    }
    // Le dossier a changé sous la vue: ne garder que ce qui existe encore
    function reloaded() {
        const keep = {};
        for (const p of Object.keys(selected))
            if (folder.has(p))
                keep[p] = true;
        selected = keep;
        if (list.currentIndex < 0 && list.count > 0)
            select(0);
    }
    function dragUris(path) {
        return folder.uris(selected[path] ? Object.keys(selected).join("\n") : path);
    }
    function focusList() {
        list.forceActiveFocus();
    }

    readonly property int sizeWidth: 84
    readonly property int dateWidth: 150
    readonly property int pad: FiliTheme.space3
    // Début du nom dans une ligne: marge, icône, écart
    readonly property int nameInset: FiliTheme.space2 + FiliTheme.iconSize + FiliTheme.space2

    // En-têtes de colonnes: petit 11, brume2; colonne de tri en brume avec son sens
    Row {
        id: header
        x: FiliTheme.space2
        width: view.width - 2 * FiliTheme.space2
        height: 26
        Repeater {
            model: [
                { key: 0, label: Fili.tr("Name"), width: header.width - view.sizeWidth - view.dateWidth, align: Text.AlignLeft },
                { key: 1, label: Fili.tr("Size"), width: view.sizeWidth, align: Text.AlignRight },
                { key: 2, label: Fili.tr("Modified"), width: view.dateWidth, align: Text.AlignLeft }
            ]
            delegate: Item {
                id: col
                required property var modelData
                readonly property bool active: view.folder.sortKey === modelData.key
                width: modelData.width
                height: header.height
                Text {
                    anchors { fill: parent; leftMargin: col.modelData.key === 2 ? FiliTheme.space4 : view.nameInset; rightMargin: col.modelData.key === 1 ? FiliTheme.space2 : FiliTheme.space2 }
                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: col.modelData.align
                    elide: Text.ElideRight
                    text: col.modelData.label + (col.active ? (view.folder.sortDesc ? "  ↓" : "  ↑") : "")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeSmall
                    color: col.active ? FiliTheme.brume : FiliTheme.brume2
                }
                Rectangle {
                    visible: col.modelData.key > 0
                    anchors { left: parent.left; verticalCenter: parent.verticalCenter }
                    width: 1
                    height: 14
                    color: FiliTheme.nuit4
                }
                MouseArea {
                    anchors.fill: parent
                    onClicked: view.select(view.folder.sortBy(col.modelData.key, list.currentIndex))
                }
            }
        }
    }
    Rectangle {
        anchors.top: header.bottom
        width: parent.width
        height: 1
        color: FiliTheme.nuit4
    }

    ListView {
        id: list
        anchors { top: header.bottom; topMargin: 1 + 2; left: parent.left; right: parent.right; bottom: parent.bottom }
        clip: true
        model: view.folder
        focus: true
        activeFocusOnTab: true
        keyNavigationEnabled: false
        highlightMoveDuration: 0
        boundsBehavior: Flickable.StopAtBounds
        reuseItems: true
        KineticScroll {}

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

        // Saisie au clavier: lettres tapées à moins d'une seconde d'intervalle
        property string typed: ""
        Timer { id: typedReset; interval: 1000; onTriggered: list.typed = "" }

        Keys.onPressed: e => {
            Fili.keyNav = true;
            const ctrl = e.modifiers & Qt.ControlModifier, alt = e.modifiers & Qt.AltModifier, shift = e.modifiers & Qt.ShiftModifier;
            const page = Math.max(1, Math.floor(list.height / FiliTheme.rowHeight) - 1);
            const move = row => shift ? view.extendTo(row) : view.select(Math.max(0, Math.min(row, count - 1)));
            if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter || (e.key === Qt.Key_Down && (ctrl || alt)))
                view.openRow(currentIndex);
            else if (e.key === Qt.Key_Backspace || (e.key === Qt.Key_Up && (ctrl || alt)))
                view.goUp();
            else if (e.key === Qt.Key_Up)
                move(currentIndex - 1);
            else if (e.key === Qt.Key_Down)
                move(currentIndex + 1);
            else if (e.key === Qt.Key_Space && list.typed === "")
                view.folder.quicklook(currentIndex);
            else if (e.key === Qt.Key_Home)
                move(0);
            else if (e.key === Qt.Key_End)
                move(count - 1);
            else if (e.key === Qt.Key_PageDown)
                move(currentIndex + page);
            else if (e.key === Qt.Key_PageUp)
                move(currentIndex - page);
            else if (e.text.length === 1 && e.text >= " " && !ctrl && !alt) {
                list.typed += e.text;
                typedReset.restart();
                view.select(view.folder.find(list.typed));
            } else
                return;
            e.accepted = true;
        }

        delegate: Rectangle {
            id: row
            required property int index
            required property string name
            required property string path
            required property bool isDir
            required property string size
            required property string modified
            required property string icon
            required property bool hidden
            readonly property bool current: ListView.isCurrentItem
            readonly property bool sel: view.selected[path] === true
            readonly property bool editing: view.renaming === index
            readonly property bool stripe: index % 2 === 1
            x: FiliTheme.space2
            width: ListView.view.width - 2 * FiliTheme.space2
            height: FiliTheme.rowHeight
            radius: FiliTheme.radiusControl
            color: sel || drop.containsDrag ? FiliTheme.nuit3 : area.containsMouse ? FiliTheme.nuit2 : stripe ? FiliTheme.alpha(FiliTheme.nuit2, 0.55) : "transparent"
            border.width: drop.containsDrag ? 1 : current && list.activeFocus && Fili.keyNav ? 1 : 0
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

            Image {
                id: img
                x: FiliTheme.space2
                anchors.verticalCenter: parent.verticalCenter
                width: FiliTheme.iconSize
                height: FiliTheme.iconSize
                sourceSize: Qt.size(FiliTheme.iconSize * 2, FiliTheme.iconSize * 2)
                source: row.icon
                asynchronous: true
                opacity: row.hidden ? 0.6 : 1
            }
            Text {
                anchors { left: img.right; leftMargin: FiliTheme.space2; right: sizeText.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                visible: !row.editing
                text: row.name
                elide: Text.ElideMiddle
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: row.ink
            }
            // Renommage sur place
            Rectangle {
                anchors { left: img.right; leftMargin: FiliTheme.space2 - 4; right: sizeText.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                height: parent.height - 2
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
            Text {
                id: sizeText
                anchors { right: dateText.left; rightMargin: FiliTheme.space4; verticalCenter: parent.verticalCenter }
                width: view.sizeWidth
                horizontalAlignment: Text.AlignRight
                text: row.size
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                font.features: { "tnum": 1 }
                color: row.sel && view.windowActive ? FiliTheme.voile : FiliTheme.brume2
            }
            Text {
                id: dateText
                anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                width: view.dateWidth - FiliTheme.space4
                text: row.modified
                elide: Text.ElideRight
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                font.features: { "tnum": 1 }
                color: row.sel && view.windowActive ? FiliTheme.voile : FiliTheme.brume2
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
                property bool collapse: false
                onPressed: e => {
                    Fili.keyNav = false;
                    view.renaming = -1;
                    collapse = false;
                    if (e.modifiers & Qt.ControlModifier)
                        view.toggle(row.index);
                    else if (e.modifiers & Qt.ShiftModifier) {
                        list.currentIndex = row.index;
                        view.extendTo(row.index);
                    } else if (!row.sel) {
                        list.currentIndex = row.index;
                        view.pick(row.index);
                    } else {
                        list.currentIndex = row.index;
                        // Sélection multiple: elle se réduit au relâchement s'il n'y a pas eu de glisser
                        collapse = true;
                    }
                    list.forceActiveFocus();
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
        anchors.centerIn: list
        visible: list.count === 0
        text: Fili.tr("Empty folder")
        font.family: FiliTheme.font
        font.pixelSize: FiliTheme.size
        color: FiliTheme.brume2
    }
}
