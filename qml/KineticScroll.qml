// Défilement inertiel au pavé tactile pour une liste (ListView, Flickable): suit les doigts, puis la liste continue sur
// son élan et ralentit quand ils se lèvent (comme macOS). La molette d'une souris (crans, sans pixelDelta) garde le
// défilement par pas de Qt. Seule animation du shell avec le flou de niri: quelques dizaines de rendus, au lâcher.
// Usage: KineticScroll {} en enfant direct de la liste.
import QtQuick

WheelHandler {
    id: k
    // Déclaré dans la liste, le gestionnaire est rattaché à son contentItem
    readonly property Flickable view: parent instanceof Flickable ? parent : parent?.parent ?? null
    property real velocity: 0   // px/s, lissée
    property real last: 0
    acceptedDevices: PointerDevice.TouchPad
    target: null

    onWheel: e => {
        const dy = e.pixelDelta.y;
        if (dy === 0)
            return;
        view.cancelFlick();
        const now = Date.now();
        const dt = Math.max(1, now - last);
        last = now;
        // Nouveau geste après une pause: vitesse repartie de zéro
        velocity = dt > 100 ? 0 : 0.6 * velocity + 0.4 * (dy * 1000 / dt);
        const max = Math.max(view.originY, view.originY + view.contentHeight - view.height);
        view.contentY = Math.max(view.originY, Math.min(max, view.contentY - dy));
        lift.restart();
    }
    // Plus d'événement depuis 50 ms: doigts levés, la liste part sur son élan
    property Timer lift: Timer {
        interval: 50
        onTriggered: if (Math.abs(k.velocity) > 150) k.view.flick(0, k.velocity)
    }
}
