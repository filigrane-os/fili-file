// Vue en liste du dossier (colonnes Nom, Taille, Modifié; un clic sur un en-tête trie, un second inverse l'ordre).
// Une ligne sélectionnée: fond nuit3 et nom en voile (brume quand la fenêtre n'est pas active); survol: fond nuit3.
// Clavier: flèches, Début/Fin, Page préc./suiv., Entrée ou Ctrl+↓ ouvre, Retour arrière ou Alt+↑ remonte,
// Espace aperçu rapide, lettres = aller au nom qui commence ainsi.
import QtQuick
import Fili.Files

Rectangle {
    id: view
    required property FolderModel folder
    property alias currentIndex: list.currentIndex
    property bool windowActive: true
    signal openRow(int row)
    signal goUp()

    color: FiliTheme.nuit

    function select(row) {
        if (row < 0)
            return;
        list.currentIndex = Math.min(row, list.count - 1);
        list.positionViewAtIndex(list.currentIndex, ListView.Contain);
    }
    function focusList() {
        list.forceActiveFocus();
    }

    readonly property int sizeWidth: 84
    readonly property int dateWidth: 150
    readonly property int pad: FiliTheme.space3

    // En-têtes de colonnes: petit 11, brume2; colonne de tri en brume avec son sens
    Row {
        id: header
        x: view.pad
        width: view.width - 2 * view.pad
        height: 24
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
                    anchors { fill: parent; leftMargin: col.modelData.key === 2 ? FiliTheme.space4 : FiliTheme.iconSize + FiliTheme.space2; rightMargin: col.modelData.key === 1 ? 0 : FiliTheme.space2 }
                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: col.modelData.align
                    elide: Text.ElideRight
                    text: col.modelData.label + (col.active ? (view.folder.sortDesc ? "  ↓" : "  ↑") : "")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeSmall
                    color: col.active ? FiliTheme.brume : FiliTheme.brume2
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
        anchors { top: header.bottom; topMargin: 1 + FiliTheme.space1; left: parent.left; right: parent.right; bottom: parent.bottom }
        clip: true
        model: view.folder
        focus: true
        activeFocusOnTab: true
        keyNavigationEnabled: true
        highlightMoveDuration: 0
        boundsBehavior: Flickable.StopAtBounds
        reuseItems: true
        KineticScroll {}

        // Saisie au clavier: lettres tapées à moins d'une seconde d'intervalle
        property string typed: ""
        Timer { id: typedReset; interval: 1000; onTriggered: list.typed = "" }

        Keys.onPressed: e => {
            Fili.keyNav = true;
            const ctrl = e.modifiers & Qt.ControlModifier, alt = e.modifiers & Qt.AltModifier;
            const page = Math.max(1, Math.floor(list.height / FiliTheme.rowHeight) - 1);
            if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter || (e.key === Qt.Key_Down && (ctrl || alt)))
                view.openRow(currentIndex);
            else if (e.key === Qt.Key_Backspace || (e.key === Qt.Key_Up && (ctrl || alt)))
                view.goUp();
            else if (e.key === Qt.Key_Space && list.typed === "")
                view.folder.quicklook(currentIndex);
            else if (e.key === Qt.Key_Home)
                view.select(0);
            else if (e.key === Qt.Key_End)
                view.select(count - 1);
            else if (e.key === Qt.Key_PageDown)
                view.select(Math.min(count - 1, currentIndex + page));
            else if (e.key === Qt.Key_PageUp)
                view.select(Math.max(0, currentIndex - page));
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
            required property string size
            required property string modified
            required property string icon
            required property bool hidden
            readonly property bool sel: ListView.isCurrentItem
            x: FiliTheme.space2
            width: ListView.view.width - 2 * FiliTheme.space2
            height: FiliTheme.rowHeight
            radius: FiliTheme.radiusControl
            color: sel || area.containsMouse ? FiliTheme.nuit3 : "transparent"
            border.width: sel && list.activeFocus && Fili.keyNav ? 1 : 0
            border.color: FiliTheme.voile
            readonly property color ink: sel && view.windowActive ? FiliTheme.voile : row.hidden ? FiliTheme.brume2 : FiliTheme.brume

            Image {
                id: img
                x: view.pad - FiliTheme.space2
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
                text: row.name
                elide: Text.ElideMiddle
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: row.ink
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
                anchors { right: parent.right; rightMargin: view.pad - FiliTheme.space2; verticalCenter: parent.verticalCenter }
                width: view.dateWidth - FiliTheme.space4
                text: row.modified
                elide: Text.ElideRight
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                font.features: { "tnum": 1 }
                color: row.sel && view.windowActive ? FiliTheme.voile : FiliTheme.brume2
            }
            MouseArea {
                id: area
                anchors.fill: parent
                hoverEnabled: true
                onPressed: {
                    Fili.keyNav = false;
                    list.currentIndex = row.index;
                    list.forceActiveFocus();
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
