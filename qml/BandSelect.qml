// Rubber-band selection: dragging the mouse on the background of the list or grid selects the items it touches
// (Ctrl or Shift: adds to the selection). Place it in the view's Flickable, under the rows; the rectangle itself is
// drawn above them.
import QtQuick
import Fili.Files

MouseArea {
    id: band
    required property Item view
    required property Flickable flick
    // In the view content, under the rows: elsewhere the mouse never gets here
    parent: flick.contentItem
    z: -1
    x: -flick.leftMargin
    width: flick.width
    height: Math.max(flick.height, flick.contentHeight)
    acceptedButtons: Qt.LeftButton
    preventStealing: true
    property real x0: 0
    property real y0: 0
    property var base: ({})

    onPressed: e => {
        const add = e.modifiers & (Qt.ControlModifier | Qt.ShiftModifier);
        view.endRename();
        Fili.keyNav = false;
        if (!add)
            view.pick(-1);
        base = add ? Object.assign({}, view.selected) : {};
        x0 = e.x;
        y0 = e.y;
        rect.visible = false;
        view.active.forceActiveFocus();
    }
    onPositionChanged: e => {
        if (!pressed)
            return;
        const x = Math.max(0, Math.min(e.x, width)), y = Math.max(0, Math.min(e.y, height));
        if (!rect.visible && Math.abs(x - x0) + Math.abs(y - y0) < 4)
            return;
        // rect lives in the view content (above the rows): its coordinates include the x of the area
        rect.x = Math.min(x0, x) + band.x;
        rect.y = Math.min(y0, y);
        rect.width = Math.abs(x - x0);
        rect.height = Math.abs(y - y0);
        rect.visible = true;
        const s = Object.assign({}, base);
        for (const it of flick.contentItem.children) {
            if (it.path === undefined || !it.visible)
                continue;
            const left = rect.x;
            if (it.x < left + rect.width && it.x + it.width > left && it.y < rect.y + rect.height && it.y + it.height > rect.y)
                s[it.path] = true;
        }
        view.selected = s;
    }
    onReleased: rect.visible = false
    onCanceled: rect.visible = false

    // The rectangle is drawn above the rows (the area itself stays below to leave them the clicks)
    Rectangle {
        id: rect
        parent: band.flick.contentItem
        z: 1000
        visible: false
        radius: 2
        color: FiliTheme.alpha(FiliTheme.accentColor, 0.15)
        border.width: FiliTheme.borderWidth
        border.color: FiliTheme.accentColor
    }
}
