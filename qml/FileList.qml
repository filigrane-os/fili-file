// View of the folder, as a list (Name, Size, Modified... columns; a click on a header sorts, a second one reverses the order) or as a
// grid of 64 px icons (name under the icon, two lines at most); same delegate, same selection and shortcuts.
// A selected row: surfaceHover background and name in accentColor (text when the window is not active); hover: surfaceHover background.
// Multiple selection: dragging on the background draws an area, Ctrl+click adds or removes, Shift+click or Shift+arrows extend, Ctrl+A everything. Keyboard: arrows,
// Home/End, Page Up/Down, Enter or Ctrl+Down opens, Backspace or Alt+Up goes to the parent, Space quick look,
// letters = go to the name that starts with them. F2 renames in place (Enter validates, Esc cancels). Dragging the
// selected rows: text/uri-list, droppable on a folder of the list, of the sidebar or in another window.
import QtQuick
import Fili.Files

Rectangle {
    id: view
    required property FolderModel folder
    // View shown: "list" or "grid" (choice kept in the user's state)
    property string viewMode: Fili.viewMode()
    readonly property bool tiles: viewMode === "grid"
    readonly property Item active: tiles ? grid : list
    readonly property int currentIndex: active.currentIndex
    function setViewMode(mode) {
        if (mode === viewMode)
            return;
        const i = active.currentIndex;
        Fili.setViewMode(mode);
        viewMode = mode;
        active.currentIndex = i;
        if (i >= 0)
            active.positionViewAtIndex(i, ListView.Contain);
        focusList();
    }
    property bool windowActive: true
    signal openRow(int row)
    signal goUp()
    signal dropped(string urls, string dest, bool copy)
    // Right click (or Menu key): position in this view; row = -1 outside the rows
    signal context(real x, real y)

    color: FiliTheme.background

    // Selected rows: path -> true. anchor: starting row of range selections.
    property var selected: ({})
    property int anchor: 0
    // Row being renamed (-1: none)
    // PDF thumbnails ready since the folder was opened: path -> true
    property var ready: ({})
    Connections {
        target: folder
        function onThumbReady(path) {
            const r = Object.assign({}, view.ready);
            r[path] = true;
            view.ready = r;
        }
    }
    // Description of the selection (status bar), empty when nothing is selected
    readonly property string info: Object.keys(selected).length > 0 ? folder.describe(Object.keys(selected).join("\n")) : ""
    property int renaming: -1
    readonly property bool editing: renaming >= 0
    // End of in-place renaming by a click elsewhere: what was typed is validated (only Esc cancels)
    signal commitEdit()
    function endRename() {
        if (renaming >= 0)
            commitEdit();
        renaming = -1;
    }

    function pick(row) {
        const s = {};
        if (row >= 0 && row < active.count)
            s[folder.pathAt(row)] = true;
        selected = s;
        anchor = row;
    }
    // Image under the cursor during a drag: the names (three at most) and the number of items, prepared on click
    property url dragImage: ""
    Item {
        id: dragPreview
        // Outside the window, but rendered: grabToImage makes the image from it
        x: -4000
        width: 220
        height: stack.implicitHeight + 6
        property var names: []
        property int total: 0
        Column {
            id: stack
            x: 3
            y: 3
            spacing: 3
            Repeater {
                model: dragPreview.names
                delegate: Rectangle {
                    required property string modelData
                    width: 200
                    height: 24
                    radius: FiliTheme.radiusControl
                    color: FiliTheme.surfaceHover
                    border.width: FiliTheme.borderWidth
                    border.color: FiliTheme.accentColor
                    Text {
                        anchors { fill: parent; leftMargin: 8; rightMargin: 8 }
                        verticalAlignment: Text.AlignVCenter
                        elide: Text.ElideMiddle
                        text: parent.modelData
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.sizeList
                        color: FiliTheme.text
                    }
                }
            }
        }
        // Badge with the number of items
        Rectangle {
            visible: dragPreview.total > 1
            x: 200 - 6
            y: -1
            width: Math.max(20, countText.implicitWidth + 10)
            height: 20
            radius: 10
            color: FiliTheme.accentBright
            Text {
                id: countText
                anchors.centerIn: parent
                text: dragPreview.total
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeSmall
                font.weight: FiliTheme.weightAccent
                color: FiliTheme.backgroundDeep
            }
        }
    }
    function prepareDrag(path) {
        const keys = selected[path] ? Object.keys(selected) : [path];
        dragPreview.names = keys.slice(0, 3).map(p => p.split("/").pop());
        dragPreview.total = keys.length;
        // Wait for the names to be laid out, then an image of the size of the preview
        Qt.callLater(() => dragPreview.grabToImage(r => { view.dragImage = r.url; }, Qt.size(dragPreview.width, dragPreview.height)));
    }
    // Click on the item of this name, once a rename is validated (what a simple click, Ctrl or Shift would have done)
    function clickNamed(name, modifiers) {
        const row = folder.rowOf(name);
        if (row < 0)
            return;
        active.currentIndex = row;
        if (modifiers & Qt.ControlModifier)
            toggle(row);
        else if (modifiers & Qt.ShiftModifier)
            extendTo(row);
        else
            pick(row);
        active.forceActiveFocus();
    }
    // A single row, selected and current
    function select(row) {
        if (row < 0)
            return;
        active.currentIndex = Math.min(row, active.count - 1);
        if (active.currentIndex >= 0)
            active.positionViewAtIndex(active.currentIndex, ListView.Contain);
        pick(active.currentIndex);
    }
    function extendTo(row) {
        if (active.count === 0)
            return;
        active.currentIndex = Math.max(0, Math.min(row, active.count - 1));
        active.positionViewAtIndex(active.currentIndex, ListView.Contain);
        const s = {};
        const from = Math.max(0, Math.min(anchor, active.currentIndex)), to = Math.max(anchor, active.currentIndex);
        for (let i = from; i <= to; i++)
            s[folder.pathAt(i)] = true;
        selected = s;
    }
    function toggle(row) {
        const s = Object.assign({}, selected);
        const p = folder.pathAt(row);
        if (s[p])
            delete s[p];
        else
            s[p] = true;
        selected = s;
        anchor = row;
        active.currentIndex = row;
    }
    function selectAll() {
        const s = {};
        for (let i = 0; i < active.count; i++)
            s[folder.pathAt(i)] = true;
        selected = s;
    }
    function selectPaths(list) {
        const s = {};
        for (const p of list)
            if (p !== "")
                s[p] = true;
        selected = s;
    }
    function invertSelection() {
        const s = {};
        for (let i = 0; i < active.count; i++) {
            const p = folder.pathAt(i);
            if (!selected[p])
                s[p] = true;
        }
        selected = s;
    }
    // Paths targeted by a command: the selection, else the current row
    function targets() {
        const keys = Object.keys(selected);
        if (keys.length > 0)
            return keys;
        return active.currentIndex >= 0 ? [folder.pathAt(active.currentIndex)] : [];
    }
    function targetsText() {
        return targets().join("\n");
    }
    function startRename() {
        if (active.currentIndex >= 0)
            renaming = active.currentIndex;
    }
    // The folder changed under the view: keep only what still exists
    function reloaded() {
        const keep = {};
        for (const p of Object.keys(selected))
            if (folder.has(p))
                keep[p] = true;
        selected = keep;
        if (active.currentIndex < 0 && active.count > 0)
            select(0);
    }
    function dragUris(path) {
        return folder.uris(selected[path] ? Object.keys(selected).join("\n") : path);
    }
    function focusList() {
        active.forceActiveFocus();
    }

    // Columns after the name: shown according to the user's choice (right click on a header), in this order
    readonly property var allColumns: [
        { key: "size", sort: 1, role: "size", label: Fili.tr("Size"), width: 96, align: Text.AlignRight },
        { key: "modified", sort: 2, role: "modified", label: Fili.tr("Modified"), width: 160, align: Text.AlignLeft },
        { key: "kind", sort: 3, role: "kind", label: Fili.tr("Kind"), width: 150, align: Text.AlignLeft },
        { key: "created", sort: 4, role: "created", label: Fili.tr("Created"), width: 160, align: Text.AlignLeft },
        { key: "permissions", sort: 5, role: "perms", label: Fili.tr("Permissions"), width: 110, align: Text.AlignLeft },
        { key: "owner", sort: 6, role: "owner", label: Fili.tr("Owner"), width: 100, align: Text.AlignLeft }
    ]
    property var enabledKeys: Fili.columns().split(",")
    readonly property var cols: allColumns.filter(c => enabledKeys.indexOf(c.key) >= 0)
    // Widths chosen by the user (drag the left edge of a header), kept in the state: {key: px}
    property var colWidths: loadWidths()
    function loadWidths() {
        const w = {};
        for (const c of allColumns) {
            const v = parseInt(Fili.uiState("col." + c.key));
            if (v >= minColWidth)
                w[c.key] = Math.min(v, maxColWidth);
        }
        return w;
    }
    readonly property int minColWidth: 56
    readonly property int maxColWidth: 600
    function baseWidth(c) {
        return colWidths[c.key] > 0 ? colWidths[c.key] : c.width;
    }
    // px <= 0: back to the default width; save=false during the drag (the value is kept on release)
    function setColWidth(key, px, save) {
        const w = Object.assign({}, colWidths);
        if (px > 0)
            w[key] = Math.max(minColWidth, Math.min(maxColWidth, Math.round(px)));
        else
            delete w[key];
        colWidths = w;
        if (save !== false)
            Fili.setUiState("col." + key, px > 0 ? String(w[key]) : "");
    }
    readonly property int colsWidth: cols.reduce((a, c) => a + baseWidth(c), 0)
    // Narrow window: the columns shrink to keep 200 px for the name (down to 35 % of their width)
    readonly property real colScale: colsWidth > 0 ? Math.max(0.35, Math.min(1, (view.width - 2 * FiliTheme.space2 - 200) / colsWidth)) : 1
    function colWidth(c) {
        return Math.round(baseWidth(c) * colScale);
    }
    function setColumns(keys) {
        Fili.setColumns(keys.join(","));
        enabledKeys = keys;
    }
    signal headerContext(real x, real y)
    readonly property int pad: FiliTheme.space3
    // Start of the name in a row: margin, icon, gap
    readonly property int nameInset: FiliTheme.space2 + FiliTheme.iconSize + FiliTheme.space2
    readonly property int cellPad: FiliTheme.space3

    // Column headers: small 11, textSecondary; sort column in text with its direction
    Row {
        id: header
        visible: !view.tiles
        x: FiliTheme.space2
        width: view.width - 2 * FiliTheme.space2
        height: 26
        Repeater {
            model: [{ key: "name", sort: 0, label: Fili.tr("Name"), width: 0, align: Text.AlignLeft }].concat(view.cols)
            delegate: Item {
                id: col
                required property var modelData
                required property int index
                readonly property bool active: view.folder.sortKey === modelData.sort
                width: index === 0 ? header.width - view.colsWidth * view.colScale : view.colWidth(modelData)
                height: header.height
                Text {
                    anchors { fill: parent; leftMargin: col.index === 0 ? view.nameInset : view.cellPad; rightMargin: view.cellPad }
                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: col.modelData.align
                    elide: Text.ElideRight
                    text: col.modelData.label + (col.active ? (view.folder.sortDesc ? "  ↓" : "  ↑") : "")
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeSmall
                    color: col.active ? FiliTheme.text : FiliTheme.textSecondary
                }
                Rectangle {
                    visible: col.index > 0
                    anchors { left: parent.left; verticalCenter: parent.verticalCenter }
                    width: 1
                    height: 14
                    color: FiliTheme.border
                }
                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: e => {
                        if (e.button === Qt.RightButton) {
                            const p = mapToItem(view, e.x, e.y);
                            view.headerContext(p.x, p.y);
                        } else
                            view.select(view.folder.sortBy(col.modelData.sort, view.currentIndex));
                    }
                }
                // Left edge of a data column: dragging it makes the column wider (to the left) or narrower;
                // a double click restores the default width. The name takes the rest.
                MouseArea {
                    id: grip
                    visible: col.index > 0
                    property real startX: 0
                    property real startWidth: 0
                    anchors { left: parent.left; top: parent.top; bottom: parent.bottom; leftMargin: -4 }
                    width: 9
                    cursorShape: Qt.SplitHCursor
                    onPressed: e => {
                        startX = mapToItem(header, e.x, 0).x;
                        startWidth = view.baseWidth(col.modelData);
                    }
                    onPositionChanged: e => {
                        if (pressed)
                            view.setColWidth(col.modelData.key, startWidth - (mapToItem(header, e.x, 0).x - startX) / view.colScale, false);
                    }
                    onReleased: Fili.setUiState("col." + col.modelData.key, String(view.colWidths[col.modelData.key] || ""))
                    onDoubleClicked: view.setColWidth(col.modelData.key, 0)
                }
            }
        }
    }
    Rectangle {
        visible: !view.tiles
        anchors.top: header.bottom
        width: parent.width
        height: 1
        color: FiliTheme.border
    }

    // Keyboard input: letters typed less than a second apart
    property string typed: ""
    Timer { id: typedReset; interval: 1000; onTriggered: view.typed = "" }

    // Keys shared by both views; perRow: number of columns of the grid (1 in list view)
    function handleKey(e, perRow) {
        Fili.keyNav = true;
        const ctrl = e.modifiers & Qt.ControlModifier, alt = e.modifiers & Qt.AltModifier, shift = e.modifiers & Qt.ShiftModifier;
        const count = active.count, cur = active.currentIndex;
        const rowsPerPage = Math.max(1, Math.floor(active.height / (tiles ? grid.cellHeight : FiliTheme.rowHeight)) - 1);
        const move = row => shift ? view.extendTo(row) : view.select(Math.max(0, Math.min(row, count - 1)));
        if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter || (e.key === Qt.Key_Down && (ctrl || alt)))
            view.openRow(cur);
        else if (e.key === Qt.Key_Backspace || (e.key === Qt.Key_Up && (ctrl || alt)))
            view.goUp();
        else if (e.key === Qt.Key_Up)
            move(cur - perRow);
        else if (e.key === Qt.Key_Down)
            move(cur + perRow);
        else if (e.key === Qt.Key_Left && tiles)
            move(cur - 1);
        else if (e.key === Qt.Key_Right && tiles)
            move(cur + 1);
        else if (e.key === Qt.Key_Menu) {
            const it = active.itemAtIndex(cur);
            const p = it ? it.mapToItem(view, FiliTheme.space4, it.height) : Qt.point(FiliTheme.space6, FiliTheme.space6);
            view.context(p.x, p.y);
        } else if (e.key === Qt.Key_Home)
            move(0);
        else if (e.key === Qt.Key_End)
            move(count - 1);
        else if (e.key === Qt.Key_PageDown)
            move(cur + rowsPerPage * perRow);
        else if (e.key === Qt.Key_PageUp)
            move(cur - rowsPerPage * perRow);
        else if (e.text.length === 1 && e.text >= " " && !ctrl && !alt) {
            view.typed += e.text;
            typedReset.restart();
            view.select(view.folder.find(view.typed));
        } else
            return;
        e.accepted = true;
    }

    ListView {
        id: list
        visible: !view.tiles
        anchors { top: header.bottom; topMargin: 1 + 2; left: parent.left; right: parent.right; bottom: parent.bottom }
        clip: true
        // The ListView forces x = 0 on its delegates: the margin of the rows is the one of the content
        leftMargin: FiliTheme.space2
        rightMargin: FiliTheme.space2
        model: view.folder
        focus: true
        activeFocusOnTab: true
        keyNavigationEnabled: false
        highlightMoveDuration: 0
        boundsBehavior: Flickable.StopAtBounds
        reuseItems: true
        KineticScroll {}

        BandSelect {
            view: view
            flick: list
        }
        MouseArea {
            id: bgArea
            z: -2
            width: list.width
            height: Math.max(list.height, list.contentHeight)
            acceptedButtons: Qt.RightButton
            onPressed: e => {
                view.pick(-1);
                const p = bgArea.mapToItem(view, e.x, e.y);
                view.context(p.x, p.y);
            }
        }
        // Drop in the folder shown (outside the folder rows)
        DropArea {
            z: -1
            x: 0
            y: 0
            width: list.width
            height: Math.max(list.height, list.contentHeight)
            onDropped: drop => {
                view.dropped(drop.urls.map(u => u.toString()).join("\n"), view.folder.path, drop.proposedAction === Qt.CopyAction);
                drop.acceptProposedAction();
            }
        }

        Keys.onPressed: e => view.handleKey(e, 1)

        delegate: fileDelegate
    }

    GridView {
        id: grid
        visible: view.tiles
        anchors { top: parent.top; topMargin: FiliTheme.space2; left: parent.left; right: parent.right; bottom: parent.bottom }
        clip: true
        cellWidth: 104
        cellHeight: 120
        readonly property int columns: Math.max(1, Math.floor((width - 2 * FiliTheme.space4) / cellWidth))
        // Grid aligned to the left (the GridView lays the cells out from the content margin)
        leftMargin: FiliTheme.space4
        rightMargin: FiliTheme.space4
        model: view.folder
        activeFocusOnTab: true
        keyNavigationEnabled: false
        highlightMoveDuration: 0
        boundsBehavior: Flickable.StopAtBounds
        reuseItems: true
        KineticScroll {}

        BandSelect {
            view: view
            flick: grid
        }
        MouseArea {
            z: -2
            width: grid.width
            height: Math.max(grid.height, grid.contentHeight)
            acceptedButtons: Qt.RightButton
            onPressed: e => {
                view.pick(-1);
                const p = mapToItem(view, e.x, e.y);
                view.context(p.x, p.y);
            }
        }
        DropArea {
            z: -1
            width: grid.width
            height: Math.max(grid.height, grid.contentHeight)
            onDropped: drop => {
                view.dropped(drop.urls.map(u => u.toString()).join("\n"), view.folder.path, drop.proposedAction === Qt.CopyAction);
                drop.acceptProposedAction();
            }
        }

        Keys.onPressed: e => view.handleKey(e, columns)

        delegate: fileDelegate
    }

    Component {
        id: fileDelegate
        Rectangle {
            id: row
            required property int index
            required property string name
            required property string path
            required property bool isDir
            required property var model
            required property string icon
            required property bool hidden
            required property string thumb
            required property bool thumbReady
            required property bool isImage
            readonly property bool tile: view.tiles
            readonly property bool current: index === view.currentIndex
            readonly property bool sel: view.selected[path] === true
            readonly property bool editing: view.renaming >= 0 && view.renaming === index && row.visible
            readonly property bool stripe: !tile && index % 2 === 1
            width: tile ? grid.cellWidth - FiliTheme.space2 : view.width - 2 * FiliTheme.space2
            height: tile ? grid.cellHeight - FiliTheme.space2 : FiliTheme.rowHeight
            // Contiguous selected rows (list): a single block, rounded at the top of the first and at the bottom of the last
            readonly property bool selAbove: sel && !tile && index > 0 && view.selected[view.folder.pathAt(index - 1)] === true
            readonly property bool selBelow: sel && !tile && index < view.active.count - 1 && view.selected[view.folder.pathAt(index + 1)] === true
            radius: tile ? FiliTheme.radiusControl + 2 : FiliTheme.radiusControl
            topLeftRadius: selAbove ? 0 : radius
            topRightRadius: selAbove ? 0 : radius
            bottomLeftRadius: selBelow ? 0 : radius
            bottomRightRadius: selBelow ? 0 : radius
            color: sel || drop.containsDrag ? FiliTheme.surfaceHover : area.containsMouse ? FiliTheme.surface : stripe ? FiliTheme.alpha(FiliTheme.surface, 0.55) : "transparent"
            border.width: drop.containsDrag ? 1 : current && view.active.activeFocus && Fili.keyNav ? 1 : 0
            border.color: FiliTheme.accentColor
            readonly property color ink: sel && view.windowActive ? FiliTheme.accentColor : row.hidden ? FiliTheme.textSecondary : FiliTheme.text

            function beginRename() {
                input.text = row.path.split("/").pop();
                input.forceActiveFocus();
                const dot = input.text.lastIndexOf(".");
                if (!row.isDir && dot > 0)
                    input.select(0, dot);
                else
                    input.selectAll();
            }
            onEditingChanged: if (editing) beginRename()
            Component.onCompleted: if (editing) beginRename()
            ListView.onReused: if (editing) beginRename()
            GridView.onReused: if (editing) beginRename()

            Image {
                id: img
                visible: !row.tile
                x: FiliTheme.space2
                anchors.verticalCenter: parent.verticalCenter
                width: FiliTheme.iconSize
                height: FiliTheme.iconSize
                sourceSize: Qt.size(FiliTheme.iconSize * 2, FiliTheme.iconSize * 2)
                source: Fili.tint(row.icon, Fili.accent, Fili.light)
                asynchronous: true
                opacity: row.hidden ? 0.6 : 1
                fillMode: Image.PreserveAspectFit
            }
            // PDF thumbnail (first page), over the icon of the type
            Image {
                anchors.fill: img
                visible: !row.tile && status === Image.Ready
                source: row.thumb !== "" && (row.thumbReady || view.ready[row.path]) ? row.thumb + "?" + (view.ready[row.path] ? 1 : 0) : ""
                sourceSize: Qt.size(FiliTheme.iconSize * 2, FiliTheme.iconSize * 2)
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                cache: false
            }
            // Grid: 64 px icon (or thumbnail) and the name below, on two lines at most
            Image {
                id: tileImg
                visible: row.tile
                x: Math.round((row.width - width) / 2)
                y: FiliTheme.space2
                width: 64
                height: 64
                sourceSize: Qt.size(128, 128)
                source: row.tile ? Fili.tint(row.icon, Fili.accent, Fili.light) : ""
                asynchronous: true
                opacity: row.hidden ? 0.6 : 1
                fillMode: Image.PreserveAspectFit
            }
            Image {
                anchors.fill: tileImg
                visible: row.tile && status === Image.Ready
                source: row.tile && row.thumb !== "" && (row.thumbReady || view.ready[row.path]) ? row.thumb + "?" + (view.ready[row.path] ? 1 : 0) : ""
                sourceSize: Qt.size(128, 128)
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                cache: false
            }
            Text {
                visible: row.tile && !row.editing
                x: FiliTheme.space1
                y: tileImg.y + tileImg.height + FiliTheme.space1
                width: row.width - 2 * FiliTheme.space1
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
                maximumLineCount: 2
                elide: Text.ElideRight
                text: row.name
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: row.ink
            }
            Text {
                anchors { left: img.right; leftMargin: FiliTheme.space2; right: cells.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                visible: !row.tile && !row.editing
                text: row.name
                elide: Text.ElideMiddle
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: row.ink
            }
            // In-place rename
            Rectangle {
                x: row.tile ? FiliTheme.space1 : FiliTheme.space2 + FiliTheme.iconSize + FiliTheme.space2 - 4
                y: row.tile ? tileImg.y + tileImg.height + FiliTheme.space1 : 1
                width: row.tile ? row.width - 2 * FiliTheme.space1 : row.width - x - cells.width - FiliTheme.space2
                height: row.tile ? 22 : parent.height - 2
                visible: row.editing
                radius: FiliTheme.radiusControl
                color: FiliTheme.surface
                border.width: FiliTheme.borderWidth
                border.color: FiliTheme.accentColor
                TextInput {
                    id: input
                    anchors { fill: parent; leftMargin: 4; rightMargin: 4 }
                    verticalAlignment: TextInput.AlignVCenter
                    clip: true
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.sizeList
                    color: FiliTheme.text
                    selectionColor: FiliTheme.surfaceActive
                    selectedTextColor: FiliTheme.text
                    function commit() {
                        if (view.renaming < 0 || view.renaming !== row.index)
                            return;
                        view.renaming = -1;
                        if (text !== row.path.split("/").pop()) {
                            const r = view.folder.rename(row.path, text);
                            if (r >= 0)
                                view.select(r);
                        }
                        view.focusList();
                    }
                    // Enter validates and stops there (without it, it goes up to the list, which would open the item)
                    Keys.onReturnPressed: e => { commit(); e.accepted = true; }
                    Keys.onEnterPressed: e => { commit(); e.accepted = true; }
                    Connections {
                        target: view
                        function onCommitEdit() { input.commit(); }
                    }
                    Keys.onEscapePressed: {
                        view.renaming = -1;
                        view.focusList();
                    }
                    onActiveFocusChanged: if (!activeFocus) commit()
                }
            }
            Row {
                id: cells
                visible: !row.tile
                anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                Repeater {
                    model: row.tile ? [] : view.cols
                    delegate: Text {
                        required property var modelData
                        width: view.colWidth(modelData)
                        leftPadding: view.cellPad
                        rightPadding: view.cellPad
                        horizontalAlignment: modelData.align
                        elide: Text.ElideRight
                        text: row.model[modelData.role]
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.sizeList
                        font.features: { "tnum": 1 }
                        color: row.sel && view.windowActive ? FiliTheme.accentColor : FiliTheme.textSecondary
                    }
                }
            }

            // Drag: the selection (or the single row) leaves as text/uri-list
            Item {
                id: ghost
                width: 1
                height: 1
                Drag.active: area.drag.active
                Drag.imageSource: view.dragImage
                Drag.hotSpot: Qt.point(24, 14)
                Drag.dragType: Drag.Automatic
                Drag.supportedActions: Qt.CopyAction | Qt.MoveAction
                Drag.proposedAction: Qt.MoveAction
                Drag.mimeData: area.drag.active ? { "text/uri-list": view.dragUris(row.path) } : ({})
            }
            DropArea {
                id: drop
                anchors.fill: parent
                enabled: row.isDir
                onDropped: d => {
                    view.dropped(d.urls.map(u => u.toString()).join("\n"), row.path, d.proposedAction === Qt.CopyAction);
                    d.acceptProposedAction();
                }
            }
            MouseArea {
                id: area
                anchors.fill: parent
                hoverEnabled: true
                drag.target: ghost
                drag.threshold: 8
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                property bool collapse: false
                onPressed: e => {
                    if (e.button === Qt.RightButton) {
                        if (!row.sel) {
                            view.active.currentIndex = row.index;
                            view.pick(row.index);
                        }
                        const p = area.mapToItem(view, e.x, e.y);
                        view.context(p.x, p.y);
                        return;
                    }
                    Fili.keyNav = false;
                    if (view.renaming >= 0) {
                        // Validate the rename first: the row can change place (sorting), the click is redone by name
                        const name = row.name, mods = e.modifiers;
                        view.endRename();
                        Qt.callLater(() => view.clickNamed(name, mods));
                        return;
                    }
                    collapse = false;
                    if (e.modifiers & Qt.ControlModifier)
                        view.toggle(row.index);
                    else if (e.modifiers & Qt.ShiftModifier) {
                        view.active.currentIndex = row.index;
                        view.extendTo(row.index);
                    } else if (!row.sel) {
                        view.active.currentIndex = row.index;
                        view.pick(row.index);
                    } else {
                        view.active.currentIndex = row.index;
                        // Multiple selection: it shrinks on release if there was no drag
                        collapse = true;
                    }
                    view.prepareDrag(row.path);
                    view.active.forceActiveFocus();
                }
                onReleased: {
                    if (collapse && !drag.active)
                        view.pick(row.index);
                    collapse = false;
                }
                onDoubleClicked: view.openRow(row.index)
            }
        }
    }

    FiliScrollBar {
        flick: list
        visible: needed && !view.tiles
        anchors { top: list.top; bottom: list.bottom; right: list.right }
    }
    FiliScrollBar {
        flick: grid
        visible: needed && view.tiles
        anchors { top: grid.top; bottom: grid.bottom; right: grid.right }
    }

    // Dossier vide
    Text {
        anchors.centerIn: view.active
        visible: view.active.count === 0
        text: Fili.tr("Empty folder")
        font.family: FiliTheme.font
        font.pixelSize: FiliTheme.size
        color: FiliTheme.textSecondary
    }
}
