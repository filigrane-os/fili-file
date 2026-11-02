// Sélection par zone: tirer la souris sur le fond de la liste ou de la grille sélectionne les éléments touchés par le
// rectangle (Ctrl ou Maj: s'ajoute à la sélection). À poser dans le Flickable de la vue, sous les lignes; le rectangle
// lui-même passe au-dessus d'elles.
import QtQuick
import Fili.Files

MouseArea {
    id: band
    required property Item view
    required property Flickable flick
    // Dans le contenu de la vue, sous les lignes: ailleurs, la souris n'arrive pas jusqu'ici
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
        // rect vit dans le contenu de la vue (au-dessus des lignes): ses coordonnées incluent x de la zone
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

    // Le rectangle se dessine au-dessus des lignes (la zone, elle, reste dessous pour leur laisser les clics)
    Rectangle {
        id: rect
        parent: band.flick.contentItem
        z: 1000
        visible: false
        radius: 2
        color: FiliTheme.alpha(FiliTheme.voile, 0.15)
        border.width: FiliTheme.borderWidth
        border.color: FiliTheme.voile
    }
}
