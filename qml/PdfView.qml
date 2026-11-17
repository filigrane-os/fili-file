// PDF viewer of the Quick Look overlay (QtPdf): the pages stay put under the mouse (no dragging of the document),
// a drag selects text, a pinch, Ctrl+wheel and Ctrl+Plus / Minus / 0 zoom, the wheel scrolls. Zoom is centered on the viewport.
import QtQuick
import QtQuick.Pdf
import QtQuick.Shapes
import Fili.Files

Item {
    id: v
    property url source
    property real renderScale: 1
    property string selectedText: ""
    property var activeSelection: null
    readonly property real minScale: 0.2
    readonly property real maxScale: 8
    readonly property bool ready: doc.status === PdfDocument.Ready

    function fit() {
        if (!ready || doc.pageCount === 0) return;
        renderScale = Math.max(minScale, Math.min(maxScale, (width - 2 * 16) / doc.maxPageWidth));
    }
    function zoomBy(factor) {
        const ratio = flick.contentHeight > 0 ? (flick.contentY + flick.height / 2) / flick.contentHeight : 0;
        renderScale = Math.max(minScale, Math.min(maxScale, renderScale * factor));
        flick.returnToBounds();
        Qt.callLater(() => { flick.contentY = Math.max(0, Math.min(flick.contentHeight - flick.height, ratio * flick.contentHeight - flick.height / 2)); });
    }
    // The page of the current selection, else the first one
    function selectAll() { const p = activeSelection ?? pages.itemAt(0)?.selection; if (p) p.selectAll(); }
    // Through the clipboard of Casier (wl-copy), like its other copies; PDF lines end in \r\n
    function copySelection() { if (selectedText !== "") Fili.copyText(selectedText.replace(/\r\n/g, "\n")); }
    // Ctrl+Plus, Ctrl+Minus, Ctrl+0: true when the key was handled
    function handleKey(key) {
        if (key === Qt.Key_Plus || key === Qt.Key_Equal) zoomBy(1.2);
        else if (key === Qt.Key_Minus) zoomBy(1 / 1.2);
        else if (key === Qt.Key_0) fit();
        else return false;
        return true;
    }
    onWidthChanged: if (!zoomed) fit()
    property bool zoomed: false
    onSourceChanged: { zoomed = false; selectedText = ""; activeSelection = null; flick.contentY = 0; flick.contentX = 0; }
    onReadyChanged: if (ready) fit()

    PdfDocument { id: doc; source: v.source }

    Flickable {
        id: flick
        anchors.fill: parent
        clip: true
        interactive: false   // a drag selects text, it never moves the document
        boundsBehavior: Flickable.StopAtBounds
        contentWidth: Math.max(width, column.width + 2 * 16)
        contentHeight: column.height + 2 * 16

        // Ctrl + wheel zooms; the plain wheel scrolls (inertial, shift or a sideways gesture scrolls sideways)
        WheelHandler {
            acceptedModifiers: Qt.ControlModifier
            acceptedDevices: PointerDevice.AllDevices
            target: null
            onWheel: e => {
                const d = e.angleDelta.y !== 0 ? e.angleDelta.y : e.pixelDelta.y * 3;
                if (d !== 0) { v.zoomed = true; v.zoomBy(Math.pow(1.1, d / 120)); }
            }
        }
        WheelHandler {
            acceptedModifiers: Qt.ShiftModifier
            acceptedDevices: PointerDevice.AllDevices
            target: null
            onWheel: e => {
                const d = e.angleDelta.y !== 0 ? e.angleDelta.y : e.angleDelta.x;
                flick.contentX = Math.max(0, Math.min(flick.contentWidth - flick.width, flick.contentX - d));
            }
        }
        KineticScroll { acceptedModifiers: Qt.NoModifier }
        // Pinch on the touchpad (or a touch screen) zooms, like Ctrl + wheel
        PinchHandler {
            target: null
            acceptedDevices: PointerDevice.TouchPad | PointerDevice.TouchScreen
            rotationAxis.enabled: false
            xAxis.enabled: false
            yAxis.enabled: false
            onScaleChanged: delta => { v.zoomed = true; v.zoomBy(delta); }
        }

        Column {
            id: column
            x: Math.max(16, (flick.contentWidth - width) / 2)
            y: 16
            spacing: 12
            Repeater {
                id: pages
                model: v.ready ? doc.pageCount : 0
                Item {
                    id: paper
                    required property int index
                    readonly property alias selection: selection
                    readonly property size pointSize: doc.pagePointSize(index)
                    width: pointSize.width * v.renderScale
                    height: pointSize.height * v.renderScale
                    Rectangle { anchors.fill: parent; color: "white" }
                    PdfPageImage {
                        id: image
                        anchors.fill: parent
                        document: doc
                        currentFrame: paper.index
                        asynchronous: true
                        fillMode: Image.PreserveAspectFit
                        sourceSize.width: paper.pointSize.width * v.renderScale * Screen.devicePixelRatio
                    }
                    PdfSelection {
                        id: selection
                        anchors.fill: parent
                        document: doc
                        page: paper.index
                        renderScale: v.renderScale
                        // Set by the drag below, not bound: PdfSelection ignores `from` while `hold` is true, and a
                        // press moves the press position before the handlers report it, so a binding lost the start
                        hold: false
                        onTextChanged: if (text !== "") { v.activeSelection = selection; v.selectedText = text; } else if (v.activeSelection === selection) v.selectedText = ""
                    }
                    // PdfSelection only computes the selected area: the highlight is drawn here
                    Shape {
                        anchors.fill: parent
                        ShapePath {
                            strokeWidth: -1
                            fillColor: FiliTheme.alpha(FiliTheme.accentColor, 0.45)
                            scale: Qt.size(v.renderScale, v.renderScale)
                            PathMultiline { paths: selection.geometry }
                        }
                    }
                    // QtPdf selects only between two points that both lie on a character: a drag started in the
                    // margin or released past the end of a line would select nothing. The ends are moved onto the
                    // nearest run of text, whose rectangles come from an invisible "select all" of the page.
                    PdfSelection {
                        id: probe
                        visible: false
                        document: doc
                        page: paper.index
                        renderScale: 1
                    }
                    property var runs: null   // [{x0, y0, x1, y1}] in points, read on the first drag
                    function readRuns() {
                        const out = [];
                        probe.selectAll();
                        const g = probe.geometry;
                        for (let i = 0; i < g.length; ++i) {
                            // QPolygonF is opaque to JavaScript: its points are read from its text form
                            const pts = String(g[i]).match(/QPointF\(([-\d.e]+),\s*([-\d.e]+)\)/g) || [];
                            let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
                            for (const p of pts) {
                                const n = p.match(/[-\d.e]+/g).map(Number);
                                x0 = Math.min(x0, n[0]); x1 = Math.max(x1, n[0]);
                                y0 = Math.min(y0, n[1]); y1 = Math.max(y1, n[1]);
                            }
                            if (pts.length > 0 && x1 > x0 && y1 > y0) out.push({ x0: x0, y0: y0, x1: x1, y1: y1 });
                        }
                        probe.clear();
                        return out;
                    }
                    // A point of the page (pixels) moved onto the nearest run of text, or left as is without one
                    function snap(p) {
                        if (runs === null) runs = readRuns();
                        const s = v.renderScale, x = p.x / s, y = p.y / s;
                        let best = null, bestDy = Infinity, bestDx = Infinity;
                        for (const r of runs) {
                            const dy = y < r.y0 ? r.y0 - y : y > r.y1 ? y - r.y1 : 0;
                            const dx = x < r.x0 ? r.x0 - x : x > r.x1 ? x - r.x1 : 0;
                            if (dy < bestDy || (dy === bestDy && dx < bestDx)) { best = r; bestDy = dy; bestDx = dx; }
                        }
                        if (best === null || (bestDx === 0 && bestDy === 0)) return p;
                        const inset = Math.min(0.5, (best.x1 - best.x0) / 2);
                        const sx = Math.max(best.x0 + inset, Math.min(best.x1 - inset, x));
                        const sy = bestDy === 0 ? y : (best.y0 + best.y1) / 2;
                        return Qt.point(sx * s, sy * s);
                    }
                    DragHandler {
                        id: drag
                        acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad | PointerDevice.Stylus
                        target: null
                        grabPermissions: PointerHandler.CanTakeOverFromAnything
                        onActiveChanged: if (active) {
                            v.activeSelection = selection;
                            selection.from = paper.snap(centroid.pressPosition);
                            selection.to = paper.snap(centroid.position);
                        }
                        onCentroidChanged: if (active) selection.to = paper.snap(centroid.position)
                    }
                    // A click without a drag forgets the selection
                    TapHandler {
                        acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad | PointerDevice.Stylus
                        onTapped: { selection.clear(); v.selectedText = ""; }
                    }
                    // A click anywhere in a page forgets the selection of the others
                    Connections {
                        target: v
                        function onActiveSelectionChanged() { if (v.activeSelection !== selection) selection.clear(); }
                    }
                }
            }
        }
    }
    FiliScrollBar { flick: flick; anchors { top: parent.top; bottom: parent.bottom; right: parent.right } }
}
