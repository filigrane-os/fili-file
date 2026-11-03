// Indicateur de défilement vertical discret: poignée fine au bord droit d'une liste ou d'une grille, visible pendant le
// défilement, au survol du bord et au glisser (fondu), absente quand tout tient. La poignée se glisse; un clic dans la
// piste saute d'une page. À poser comme frère de la vue, ancré sur elle (anchors.top/bottom/right de la vue).
import QtQuick
import Fili.Files

Item {
    id: bar
    required property Flickable flick
    readonly property bool needed: flick.contentHeight > flick.height + 1
    readonly property real ratio: Math.min(1, flick.height / Math.max(1, flick.contentHeight))
    readonly property real thumbHeight: Math.max(32, ratio * height)
    // Part parcourue (0 à 1) du défilement possible
    readonly property real fraction: {
        const room = flick.contentHeight - flick.height;
        return room > 0 ? Math.max(0, Math.min(1, (flick.contentY - flick.originY) / room)) : 0;
    }
    visible: needed
    width: 14
    z: 50

    // Poignée
    Rectangle {
        id: thumb
        x: parent.width - width - 3
        width: grip.containsMouse || grip.drag.active ? 8 : 6
        height: bar.thumbHeight
        radius: width / 2
        color: FiliTheme.alpha(FiliTheme.text, grip.drag.active ? 0.55 : grip.containsMouse ? 0.45 : 0.3)
        opacity: flick.moving || grip.containsMouse || grip.drag.active || hold.running ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: 180 } }
        Behavior on width { NumberAnimation { duration: 100 } }
    }
    // Position d'après le défilement, sauf pendant qu'on la glisse (c'est alors elle qui mène le défilement)
    Binding {
        target: thumb
        property: "y"
        value: bar.fraction * (bar.height - thumb.height)
        when: !grip.drag.active
    }
    // Reste visible un instant après la fin du défilement
    Timer { id: hold; interval: 700 }
    Connections {
        target: bar.flick
        function onContentYChanged() { hold.restart(); }
    }

    MouseArea {
        id: grip
        anchors.fill: parent
        hoverEnabled: true
        // La poignée suit le curseur; le défilement en découle
        drag.target: thumb
        drag.axis: Drag.YAxis
        drag.minimumY: 0
        drag.maximumY: bar.height - thumb.height
        onPressed: e => {
            // Clic hors de la poignée: une page vers le clic
            if (e.y < thumb.y || e.y > thumb.y + thumb.height) {
                const page = bar.flick.height * 0.9;
                bar.flick.contentY = Math.max(bar.flick.originY, Math.min(bar.flick.originY + bar.flick.contentHeight - bar.flick.height, bar.flick.contentY + (e.y < thumb.y ? -page : page)));
            }
        }
        onPositionChanged: {
            if (grip.drag.active) {
                const span = bar.height - thumb.height;
                const room = bar.flick.contentHeight - bar.flick.height;
                if (span > 0 && room > 0)
                    bar.flick.contentY = bar.flick.originY + thumb.y / span * room;
            }
        }
    }
}
