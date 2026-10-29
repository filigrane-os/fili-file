// Contour de l'élément choisi au clavier, visible seulement une fois le clavier utilisé (Fili.keyNav), comme le
// FocusRing du fili-shell. gap négatif: contour dessiné à l'intérieur (vues collées à leurs voisines).
import QtQuick
import Fili.Files

Rectangle {
    property Item target: parent
    property bool shown: target.activeFocus
    property real gap: 2
    anchors { fill: target; margins: -gap }
    radius: Math.max(0, (target.radius ?? FiliTheme.radiusControl) + gap)
    color: "transparent"
    border.width: 1
    border.color: FiliTheme.voile
    visible: shown && Fili.keyNav
}
