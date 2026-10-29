// Bouton à texte (barre d'outils de la corbeille, barre d'état, confirmation): survol nuit3, pressé nuit4.
// primary: texte en voile (action par défaut).
import QtQuick
import Fili.Files

Rectangle {
    id: b
    property string label
    property bool primary: false
    signal clicked()

    width: text.implicitWidth + 2 * FiliTheme.space3
    height: 22
    radius: FiliTheme.radiusControl
    color: area.pressed ? FiliTheme.nuit4 : area.containsMouse ? FiliTheme.nuit3 : "transparent"
    border.width: primary ? 1 : 0
    border.color: FiliTheme.voile

    Text {
        id: text
        anchors.centerIn: parent
        text: b.label
        font.family: FiliTheme.font
        font.pixelSize: FiliTheme.sizeList
        color: b.primary ? FiliTheme.voile : FiliTheme.brume
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: b.clicked()
    }
}
