// Bouton d'icône de la barre d'outils (glyphe Fili Icons): survol nuit3, pressé nuit4, désactivé ardoise
import QtQuick
import Fili.Files

Rectangle {
    id: b
    property string glyph
    property string tip
    property bool enabledState: true
    signal clicked()

    width: 26
    height: 24
    radius: FiliTheme.radiusControl
    color: !enabledState ? "transparent" : area.pressed ? FiliTheme.nuit4 : area.containsMouse ? FiliTheme.nuit3 : "transparent"
    activeFocusOnTab: false

    Text {
        anchors.centerIn: parent
        text: b.glyph
        font.family: FiliTheme.icons
        font.pixelSize: 14
        color: b.enabledState ? FiliTheme.brume : FiliTheme.ardoise
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        enabled: b.enabledState
        onClicked: b.clicked()
    }
}
