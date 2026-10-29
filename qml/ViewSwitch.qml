// Bascule liste / grille de la barre d'outils: deux segments dans une capsule (comme les boutons de navigation), le mode
// actif en nuit3 avec son pictogramme en voile. Pictogrammes dessinés (trois barres, quatre carrés).
import QtQuick
import Fili.Files

Rectangle {
    id: sw
    property string mode: "list"
    signal chose(string mode)

    width: 2 * 30 + 2 * FiliTheme.space1 + 2
    height: 30
    radius: 8
    color: FiliTheme.nuit2
    border.width: 1
    border.color: FiliTheme.nuit4

    component Segment: Rectangle {
        id: seg
        required property string value
        property bool on: sw.mode === value
        x: FiliTheme.space1 + (value === "list" ? 0 : 30)
        anchors.verticalCenter: parent.verticalCenter
        width: 30
        height: 22
        radius: FiliTheme.radiusControl
        color: on || area.containsMouse ? FiliTheme.nuit3 : "transparent"
        readonly property color ink: on ? FiliTheme.voile : FiliTheme.brume2
        default property alias glyph: holder.data
        Item {
            id: holder
            anchors.centerIn: parent
            width: 14
            height: 12
        }
        MouseArea {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onClicked: sw.chose(seg.value)
        }
    }

    Segment {
        value: "list"
        Repeater {
            model: 3
            Rectangle {
                required property int index
                y: index * 5
                width: 14
                height: 2
                radius: 1
                color: parent.parent.ink
            }
        }
    }
    Segment {
        value: "grid"
        Repeater {
            model: 4
            Rectangle {
                required property int index
                x: (index % 2) * 8
                y: Math.floor(index / 2) * 7
                width: 6
                height: 5
                radius: 1
                color: parent.parent.ink
            }
        }
    }
}
