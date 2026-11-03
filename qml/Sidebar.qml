// Finder-style sidebar: places (home folder, XDG folders, GTK bookmarks, trash) and devices,
// in sections with headings (11 px DemiBold muted, no capitals), monochrome glyphs tinted accentColor. The place
// shown is selected (surfaceHover background, accentColor text). To be validated: rows of 28, headings 11 DemiBold.
import QtQuick
import QtQuick.Effects
import Fili.Files

Rectangle {
    id: side
    property string current
    signal open(string path)
    signal dropped(string urls, string dest, bool copy)
    // Right click on a place: its data and the position in the window
    signal context(string path, bool removable, bool frequent, real x, real y)

    color: FiliTheme.backgroundDeep

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
        // The rows are inset px from the edge (the ListView forces x = 0 on its delegates: the margin is the content's)
        readonly property int inset: FiliTheme.space2 + 2
        leftMargin: inset
        rightMargin: inset
        anchors.bottomMargin: FiliTheme.space2
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        model: places
        keyNavigationEnabled: true
        // A hairline between rows: the background of the current place does not touch the hover one
        spacing: 2
        activeFocusOnTab: true
        highlightMoveDuration: 0
        // Drop a folder between places (or in the empty space): add it to the favorites
        DropArea {
            z: -1
            width: list.width
            height: Math.max(list.height, list.contentHeight)
            onDropped: d => {
                // addFavorite expects a path, not a file:// URL
                d.urls.forEach(u => Fili.addFavorite(decodeURIComponent(u.toString().replace(/^file:\/\//, ""))));
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
            color: FiliTheme.muted
        }
        delegate: Rectangle {
            id: row
            required property string name
            required property string path
            required property string dev
            required property bool removable
            required property string section
            required property string mono
            required property bool full
            required property int index
            readonly property bool sel: path !== "" && path === side.current
            width: ListView.view.width - 2 * list.inset
            height: FiliTheme.sideRowHeight + 2
            radius: FiliTheme.radiusControl
            color: sel || area.containsMouse || drop.containsDrag ? FiliTheme.surfaceHover : "transparent"
            border.width: drop.containsDrag || (list.activeFocus && Fili.keyNav && ListView.isCurrentItem) ? 1 : 0
            border.color: FiliTheme.accentColor

            Image {
                id: img
                x: FiliTheme.margin - list.inset
                anchors.verticalCenter: parent.verticalCenter
                width: 16
                height: 16
                sourceSize: Qt.size(32, 32)
                source: row.mono
                layer.enabled: true
                layer.effect: MultiEffect {
                    colorization: 1.0
                    colorizationColor: row.sel ? FiliTheme.accentBright : FiliTheme.accentColor
                }
            }
            // Full trash: badge on the glyph (the full glyph is hardly distinguishable at 16 px)
            Rectangle {
                visible: row.full
                x: img.x + img.width - 6
                y: img.y - 1
                width: 7
                height: 7
                radius: 3.5
                color: FiliTheme.accentBright
                border.width: FiliTheme.borderWidth
                border.color: row.sel ? FiliTheme.surfaceHover : FiliTheme.backgroundDeep
            }
            Text {
                anchors { left: img.right; leftMargin: FiliTheme.space2 + 2; right: eject.visible ? eject.left : parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                text: row.name
                opacity: row.dev !== "" && row.path === "" ? 0.6 : 1
                elide: Text.ElideRight
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: row.sel ? FiliTheme.accentColor : FiliTheme.text
            }
            // Eject a mounted volume; forget a remembered server (not connected)
            Text {
                id: eject
                z: 2 // above the click area of the row, declared further down
                visible: row.dev !== "" && (row.path !== "" || row.dev.startsWith("srv:"))
                anchors { right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                text: row.path === "" ? "✕" : "⏏"
                font.pixelSize: FiliTheme.size
                color: ejectArea.containsMouse ? FiliTheme.text : FiliTheme.muted
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
                enabled: row.path !== "nearby:" && row.path !== "network:"
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
                        if (row.path !== "" && row.path !== "nearby:" && row.path !== "network:") {
                            const p = area.mapToItem(null, e.x, e.y);
                            side.context(row.path, row.removable, row.section === Fili.tr("Recents"), p.x, p.y);
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
