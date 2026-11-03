// Inertial touchpad scrolling for a list (ListView, Flickable): follows the fingers, then the list keeps going on
// its momentum and slows down when they lift (like macOS). A mouse wheel (notches, no pixelDelta) keeps Qt's
// step scrolling. The only animation of the shell besides niri's blur: a few dozen frames, on release.
// Usage: KineticScroll {} as a direct child of the list.
import QtQuick

WheelHandler {
    id: k
    // Declared in the list, the handler is attached to its contentItem
    readonly property Flickable view: parent instanceof Flickable ? parent : parent?.parent ?? null
    property real velocity: 0   // px/s, smoothed
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
        // New gesture after a pause: speed restarted from zero
        velocity = dt > 100 ? 0 : 0.6 * velocity + 0.4 * (dy * 1000 / dt);
        const max = Math.max(view.originY, view.originY + view.contentHeight - view.height);
        view.contentY = Math.max(view.originY, Math.min(max, view.contentY - dy));
        lift.restart();
    }
    // No event for 50 ms: fingers lifted, the list goes on with its momentum
    property Timer lift: Timer {
        interval: 50
        onTriggered: if (Math.abs(k.velocity) > 150) k.view.flick(0, k.velocity)
    }
}
