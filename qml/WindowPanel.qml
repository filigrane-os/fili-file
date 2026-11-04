// Window panel (toolbar button "Window"): every window action niri offers, one clickable tile with its pictogram.
// It opens under the button, a click elsewhere or Esc closes it, and it closes after an action. The actions run on
// this window (the focused one) through Fili.windowAction (src/window.rs lists the names it accepts). Outside niri
// the panel offers what Qt does on any desktop: fullscreen, maximize, minimize (items with `local`).
import QtQuick
import QtQuick.Window
import Fili.Files

Item {
    id: panel
    property real anchorRight: width
    property real anchorTop: 0
    readonly property int tileW: 92
    readonly property int tileH: 62
    readonly property var localGroups: [
        { title: Fili.tr("Window"), items: [
            { local: "fullscreen", glyph: "fullscreen", label: Fili.tr("Fullscreen") },
            { local: "maximize", glyph: "maximize-edges", label: Fili.tr("Maximize") },
            { local: "minimize", glyph: "minimize", label: Fili.tr("Minimize") }
        ] }
    ]
    readonly property var groups: Fili.windowControl() ? niriGroups : localGroups
    readonly property var niriGroups: [
        { title: Fili.tr("Size"), items: [
            { action: "fullscreen-window", glyph: "fullscreen", label: Fili.tr("Fullscreen") },
            { action: "toggle-windowed-fullscreen", glyph: "windowed-fullscreen", label: Fili.tr("Windowed fullscreen") },
            { action: "maximize-column", glyph: "maximize-column", label: Fili.tr("Maximize column") },
            { action: "maximize-window-to-edges", glyph: "maximize-edges", label: Fili.tr("Maximize to edges") },
            { action: "expand-column-to-available-width", glyph: "expand", label: Fili.tr("Fill the width") },
            { action: "switch-preset-column-width", glyph: "preset-width", label: Fili.tr("Next width") },
            { action: "switch-preset-window-height", glyph: "preset-height", label: Fili.tr("Next height") },
            { action: "reset-window-height", glyph: "reset-height", label: Fili.tr("Reset height") }
        ] },
        { title: Fili.tr("Position"), items: [
            { action: "center-column", glyph: "center-column", label: Fili.tr("Center column") },
            { action: "center-window", glyph: "center-window", label: Fili.tr("Center window") },
            { action: "toggle-window-floating", glyph: "float", label: Fili.tr("Floating") },
            { action: "move-column-left", glyph: "column-left", label: Fili.tr("Column left") },
            { action: "move-column-right", glyph: "column-right", label: Fili.tr("Column right") },
            { action: "move-window-up", glyph: "window-up", label: Fili.tr("Move up") },
            { action: "move-window-down", glyph: "window-down", label: Fili.tr("Move down") }
        ] },
        { title: Fili.tr("Arrange"), items: [
            { action: "consume-or-expel-window-left", glyph: "join-left", label: Fili.tr("Join left") },
            { action: "consume-or-expel-window-right", glyph: "join-right", label: Fili.tr("Join right") },
            { action: "toggle-column-tabbed-display", glyph: "tabs", label: Fili.tr("Tabbed column") },
            { action: "toggle-overview", glyph: "overview", label: Fili.tr("Overview") }
        ] },
        { title: Fili.tr("Send to"), items: [
            { action: "move-window-to-workspace-up", glyph: "workspace-up", label: Fili.tr("Workspace above") },
            { action: "move-window-to-workspace-down", glyph: "workspace-down", label: Fili.tr("Workspace below") },
            { action: "move-window-to-monitor-left", glyph: "monitor-left", label: Fili.tr("Monitor left") },
            { action: "move-window-to-monitor-right", glyph: "monitor-right", label: Fili.tr("Monitor right") },
            { action: "screenshot-window", glyph: "screenshot", label: Fili.tr("Screenshot") }
        ] }
    ]

    // Keyboard selection: index in the flat list of tiles; rows[] gives the tiles of each visual row
    property int current: 0
    // Last pointer position seen: the pointer resting under the panel when it opens must not take the selection
    property real px: -1
    property real py: -1
    readonly property var rows: {
        const r = [];
        let n = 0;
        for (const g of groups) {
            for (let i = 0; i < g.items.length; i += 3) {
                const row = [];
                for (let k = i; k < Math.min(i + 3, g.items.length); k++)
                    row.push(n + k);
                r.push(row);
            }
            n += g.items.length;
        }
        return r;
    }
    readonly property var flat: groups.reduce((a, g) => a.concat(g.items), [])
    function rowOf(i) {
        return rows.findIndex(r => r.includes(i));
    }
    function moveRow(d) {
        const r = rowOf(current), col = rows[r].indexOf(current);
        const t = rows[(r + d + rows.length) % rows.length];
        current = t[Math.min(col, t.length - 1)];
    }

    visible: false
    anchors.fill: parent
    focus: visible
    z: 100

    function open() {
        current = 0;
        px = -1;
        visible = true;
        forceActiveFocus();
    }
    function close() {
        visible = false;
    }
    function run(item) {
        close();
        if (item.action) {
            Fili.windowAction(item.action);
            return;
        }
        const w = Window.window;
        if (!w)
            return;
        const target = { fullscreen: Window.FullScreen, maximize: Window.Maximized, minimize: Window.Minimized }[item.local];
        w.visibility = w.visibility === target ? Window.Windowed : target;
    }
    Keys.onPressed: e => {
        const n = flat.length;
        switch (e.key) {
        case Qt.Key_Escape: close(); break;
        case Qt.Key_Right: current = (current + 1) % n; break;
        case Qt.Key_Left: current = (current - 1 + n) % n; break;
        case Qt.Key_Tab: current = (current + 1) % n; break;
        case Qt.Key_Backtab: current = (current - 1 + n) % n; break;
        case Qt.Key_Down: moveRow(1); break;
        case Qt.Key_Up: moveRow(-1); break;
        case Qt.Key_Home: current = 0; break;
        case Qt.Key_End: current = n - 1; break;
        case Qt.Key_Return:
        case Qt.Key_Enter:
        case Qt.Key_Space: run(flat[current]); break;
        default: return;
        }
        e.accepted = true;
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        onPressed: panel.close()
    }

    Rectangle {
        id: card
        x: Math.max(FiliTheme.space1, panel.anchorRight - width)
        y: panel.anchorTop
        width: 3 * panel.tileW + 2 * FiliTheme.space1 + 2 * FiliTheme.space2 + 2
        height: body.height + 2 * FiliTheme.space2 + 2
        radius: 8
        color: FiliTheme.surface
        border.width: FiliTheme.borderWidth
        border.color: FiliTheme.border

        // Keeps clicks inside the card from closing it
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
        }

        Column {
            id: body
            x: FiliTheme.space2 + 1
            y: FiliTheme.space2 + 1
            spacing: FiliTheme.space2
            Repeater {
                model: panel.groups
                delegate: Column {
                    id: grp
                    required property var modelData
                    required property int index
                    readonly property int base: panel.groups.slice(0, index).reduce((a, g) => a + g.items.length, 0)
                    spacing: FiliTheme.space1
                    Text {
                        text: grp.modelData.title
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.sizeSmall
                        font.weight: FiliTheme.weightAccent
                        color: FiliTheme.muted
                    }
                    Grid {
                        columns: 3
                        spacing: FiliTheme.space1
                        Repeater {
                            model: grp.modelData.items
                            delegate: Rectangle {
                                id: tile
                                required property var modelData
                                required property int index
                                readonly property bool selected: panel.current === grp.base + index
                                width: panel.tileW
                                height: panel.tileH
                                radius: FiliTheme.radiusControl
                                color: tileArea.pressed ? FiliTheme.surfaceActive : tile.selected ? FiliTheme.surfaceHover : "transparent"
                                border.width: tile.selected ? 1 : 0
                                border.color: FiliTheme.accentColor
                                Column {
                                    anchors.centerIn: parent
                                    spacing: 4
                                    WindowGlyph {
                                        anchors.horizontalCenter: parent.horizontalCenter
                                        kind: tile.modelData.glyph
                                        width: 20
                                        height: 20
                                    }
                                    Text {
                                        width: panel.tileW - 8
                                        horizontalAlignment: Text.AlignHCenter
                                        text: tile.modelData.label
                                        wrapMode: Text.WordWrap
                                        maximumLineCount: 2
                                        elide: Text.ElideRight
                                        font.family: FiliTheme.font
                                        font.pixelSize: FiliTheme.sizeSmall
                                        color: FiliTheme.text
                                    }
                                }
                                MouseArea {
                                    id: tileArea
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    onPositionChanged: mouse => {
                                        const p = tileArea.mapToItem(panel, mouse.x, mouse.y);
                                        const moved = panel.px >= 0 && Math.abs(p.x - panel.px) + Math.abs(p.y - panel.py) >= 3;
                                        if (moved)
                                            panel.current = grp.base + tile.index;
                                        if (panel.px < 0 || moved) {
                                            panel.px = p.x;
                                            panel.py = p.y;
                                        }
                                    }
                                    onClicked: panel.run(tile.modelData)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
