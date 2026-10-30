// Barre latérale à la manière du Finder: lieux (dossier personnel, dossiers XDG, signets GTK, corbeille) et appareils,
// en sections à intertitres (11 px DemiGras ardoise, sans capitales), glyphes monochromes teintés voile. Le lieu
// affiché est sélectionné (fond nuit3, texte voile). Décision à valider: lignes de 28, intertitres 11 DemiGras.
import QtQuick
import QtQuick.Effects
import Fili.Files

Rectangle {
    id: side
    property string current
    signal open(string path)
    signal dropped(string urls, string dest, bool copy)
    // Clic droit sur un lieu: ses données et la position dans la fenêtre
    signal context(string path, bool removable, real x, real y)

    color: FiliTheme.nuitProfond

    ListModel { id: places }
    function reload() {
        places.clear();
        JSON.parse(Fili.places()).forEach(p => places.append(p));
    }
    Component.onCompleted: reload()
    Connections {
        target: Fili
        function onPlacesChanged() {
            side.reload();
        }
    }

    ListView {
        id: list
        anchors.fill: parent
        anchors.topMargin: FiliTheme.space1
        // Les lignes sont inset px du bord (le ListView impose x = 0 à ses délégués: la marge est celle du contenu)
        readonly property int inset: FiliTheme.space2 + 2
        leftMargin: inset
        rightMargin: inset
        anchors.bottomMargin: FiliTheme.space2
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        model: places
        keyNavigationEnabled: true
        // Un liseré entre les lignes: le fond du lieu courant ne touche pas celui du survol
        spacing: 2
        activeFocusOnTab: true
        highlightMoveDuration: 0
        // Déposer un dossier entre les lieux (ou dans le vide): l'ajouter aux favoris
        DropArea {
            z: -1
            width: list.width
            height: Math.max(list.height, list.contentHeight)
            onDropped: d => {
                d.urls.forEach(u => Fili.addFavorite(u.toString()));
                d.acceptProposedAction();
            }
        }
        section.property: "section"
        section.delegate: Text {
            required property string section
            width: ListView.view.width
            height: 38
            leftPadding: FiliTheme.margin - list.inset
            verticalAlignment: Text.AlignBottom
            bottomPadding: FiliTheme.space1
            text: section
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeSmall
            font.weight: FiliTheme.weightAccent
            color: FiliTheme.ardoise
        }
        delegate: Rectangle {
            id: row
            required property string name
            required property string path
            required property string dev
            required property bool removable
            required property string mono
            required property string monoLight
            required property bool full
            required property int index
            readonly property bool sel: path !== "" && path === side.current
            width: ListView.view.width - 2 * list.inset
            height: FiliTheme.sideRowHeight + 2
            radius: FiliTheme.radiusControl
            color: sel || area.containsMouse || drop.containsDrag ? FiliTheme.nuit3 : "transparent"
            border.width: drop.containsDrag || (list.activeFocus && Fili.keyNav && ListView.isCurrentItem) ? 1 : 0
            border.color: FiliTheme.voile

            Image {
                id: img
                x: FiliTheme.margin - list.inset
                anchors.verticalCenter: parent.verticalCenter
                width: 16
                height: 16
                sourceSize: Qt.size(32, 32)
                source: FiliTheme.light ? row.monoLight : row.mono
                layer.enabled: true
                layer.effect: MultiEffect {
                    colorization: 1.0
                    colorizationColor: row.sel ? FiliTheme.voileVif : FiliTheme.voile
                }
            }
            // Corbeille pleine: pastille sur le glyphe (le glyphe plein de Papirus ne se distingue guère à 16 px)
            Rectangle {
                visible: row.full
                x: img.x + img.width - 6
                y: img.y - 1
                width: 7
                height: 7
                radius: 3.5
                color: FiliTheme.voileVif
                border.width: 1
                border.color: row.sel ? FiliTheme.nuit3 : FiliTheme.nuitProfond
            }
            Text {
                anchors { left: img.right; leftMargin: FiliTheme.space2 + 2; right: eject.visible ? eject.left : parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                text: row.name
                opacity: row.dev !== "" && row.path === "" ? 0.6 : 1
                elide: Text.ElideRight
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: row.sel ? FiliTheme.voile : FiliTheme.brume
            }
            // Éjecter un volume monté
            Text {
                id: eject
                visible: row.dev !== "" && row.path !== ""
                anchors { right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                text: "⏏"
                font.pixelSize: FiliTheme.size
                color: ejectArea.containsMouse ? FiliTheme.brume : FiliTheme.ardoise
                MouseArea {
                    id: ejectArea
                    anchors { fill: parent; margins: -6 }
                    hoverEnabled: true
                    onClicked: Fili.unmountVolume(row.dev)
                }
            }
            DropArea {
                id: drop
                anchors.fill: parent
                onDropped: d => {
                    side.dropped(d.urls.map(u => u.toString()).join("\n"), row.path, d.proposedAction === Qt.CopyAction);
                    d.acceptProposedAction();
                }
            }
            MouseArea {
                id: area
                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                onClicked: e => {
                    if (e.button === Qt.RightButton) {
                        if (row.path !== "") {
                            const p = area.mapToItem(null, e.x, e.y);
                            side.context(row.path, row.removable, p.x, p.y);
                        }
                        return;
                    }
                    Fili.keyNav = false;
                    list.currentIndex = row.index;
                    if (row.path === "")
                        Fili.mountVolume(row.dev);
                    else
                        side.open(row.path);
                }
            }
        }
        Keys.onPressed: e => {
            Fili.keyNav = true;
            if ((e.key === Qt.Key_Return || e.key === Qt.Key_Enter || e.key === Qt.Key_Space) && currentIndex >= 0) {
                const p = places.get(currentIndex);
                if (p.path === "")
                    Fili.mountVolume(p.dev);
                else
                    side.open(p.path);
                e.accepted = true;
            }
        }
    }
}
