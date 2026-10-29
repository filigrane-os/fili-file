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

    color: FiliTheme.nuitProfond

    ListModel { id: places }
    Component.onCompleted: JSON.parse(Fili.places()).forEach(p => places.append(p))

    ListView {
        id: list
        anchors.fill: parent
        anchors.topMargin: FiliTheme.space2
        anchors.bottomMargin: FiliTheme.space2
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        model: places
        keyNavigationEnabled: true
        activeFocusOnTab: true
        highlightMoveDuration: 0
        section.property: "section"
        section.delegate: Text {
            required property string section
            width: ListView.view.width
            height: 30
            leftPadding: FiliTheme.space4 - 2
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
            required property string mono
            required property string monoLight
            required property int index
            readonly property bool sel: path === side.current
            x: FiliTheme.space2
            width: ListView.view.width - 2 * FiliTheme.space2
            height: FiliTheme.sideRowHeight
            radius: FiliTheme.radiusControl
            color: sel || area.containsMouse || drop.containsDrag ? FiliTheme.nuit3 : "transparent"
            border.width: drop.containsDrag || (list.activeFocus && Fili.keyNav && ListView.isCurrentItem) ? 1 : 0
            border.color: FiliTheme.voile

            Image {
                id: img
                x: FiliTheme.space2
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
            Text {
                anchors { left: img.right; leftMargin: FiliTheme.space2 + 2; right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                text: row.name
                elide: Text.ElideRight
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: row.sel ? FiliTheme.voile : FiliTheme.brume
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
                onClicked: {
                    Fili.keyNav = false;
                    list.currentIndex = row.index;
                    side.open(row.path);
                }
            }
        }
        Keys.onPressed: e => {
            Fili.keyNav = true;
            if ((e.key === Qt.Key_Return || e.key === Qt.Key_Enter || e.key === Qt.Key_Space) && currentIndex >= 0) {
                side.open(places.get(currentIndex).path);
                e.accepted = true;
            }
        }
    }
}
