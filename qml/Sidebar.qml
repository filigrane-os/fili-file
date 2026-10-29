// Barre latérale: lieux (dossier personnel, dossiers XDG, signets GTK) et appareils, en sections à intertitres
// (capitales espacées, ardoise, comme PanelSection). Le lieu affiché est sélectionné (fond nuit3, texte voile).
import QtQuick
import Fili.Files

Rectangle {
    id: side
    property string current
    signal open(string path)

    color: FiliTheme.nuitProfond

    ListModel { id: places }
    Component.onCompleted: JSON.parse(Fili.places()).forEach(p => places.append(p))

    ListView {
        id: list
        anchors.fill: parent
        anchors.topMargin: FiliTheme.space1
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
            height: FiliTheme.sideRowHeight + FiliTheme.space2
            leftPadding: FiliTheme.space3
            verticalAlignment: Text.AlignBottom
            bottomPadding: FiliTheme.space1
            text: section
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeSmall
            font.capitalization: Font.AllUppercase
            font.letterSpacing: FiliTheme.sizeSmall * 0.14
            color: FiliTheme.ardoise
        }
        delegate: Rectangle {
            id: row
            required property string name
            required property string path
            required property string icon
            required property int index
            readonly property bool sel: path === side.current
            x: FiliTheme.space2
            width: ListView.view.width - 2 * FiliTheme.space2
            height: FiliTheme.sideRowHeight
            radius: FiliTheme.radiusControl
            color: sel || area.containsMouse ? FiliTheme.nuit3 : "transparent"
            border.width: list.activeFocus && Fili.keyNav && ListView.isCurrentItem ? 1 : 0
            border.color: FiliTheme.voile

            Image {
                id: img
                x: FiliTheme.space1
                anchors.verticalCenter: parent.verticalCenter
                width: FiliTheme.iconSize
                height: FiliTheme.iconSize
                sourceSize: Qt.size(FiliTheme.iconSize * 2, FiliTheme.iconSize * 2)
                source: row.icon
            }
            Text {
                anchors { left: img.right; leftMargin: FiliTheme.space2 - 2; right: parent.right; rightMargin: FiliTheme.space1; verticalCenter: parent.verticalCenter }
                text: row.name
                elide: Text.ElideRight
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: row.sel ? FiliTheme.voile : FiliTheme.brume
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
