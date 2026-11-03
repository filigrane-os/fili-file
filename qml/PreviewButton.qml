// Bouton de la barre d'outils: montre ou cache le panneau d'aperçu (même commande que Ctrl+Maj+P). Pictogramme dessiné:
// une fenêtre dont la bande de droite est pleine quand le panneau est ouvert.
import QtQuick
import Fili.Files

Rectangle {
    id: b
    property bool on: false
    signal clicked()

    width: 28
    height: 24
    radius: FiliTheme.radiusControl
    color: area.pressed ? FiliTheme.surfaceActive : area.containsMouse || b.on ? FiliTheme.surfaceHover : "transparent"

    Rectangle {
        anchors.centerIn: parent
        width: 16
        height: 12
        radius: 2
        color: "transparent"
        border { width: 1.5; color: FiliTheme.text }
        Rectangle {
            x: parent.width - width - 1.5
            y: 1.5
            width: 4
            height: parent.height - 3
            color: b.on ? FiliTheme.text : FiliTheme.alpha(FiliTheme.text, 0.45)
        }
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: b.clicked()
    }
}
