// Toolbar icon button (Fili Icons glyph): hover surfaceHover, pressed surfaceActive, disabled muted
import QtQuick
import Fili.Files

Rectangle {
    id: b
    property string glyph
    property string tip
    property bool enabledState: true
    signal clicked()

    width: 28
    height: 24
    radius: FiliTheme.radiusControl
    color: !enabledState ? "transparent" : area.pressed ? FiliTheme.surfaceActive : area.containsMouse ? FiliTheme.surfaceHover : "transparent"
    activeFocusOnTab: false

    Text {
        anchors.centerIn: parent
        text: b.glyph
        font.family: FiliTheme.icons
        font.pixelSize: 14
        color: b.enabledState ? FiliTheme.text : FiliTheme.muted
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        enabled: b.enabledState
        onClicked: b.clicked()
    }
}
