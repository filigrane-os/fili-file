// Drag of the favorites with the mouse, on the real qml/Sidebar.qml (copied next to this file by tests/sidebar-drag.sh)
// with a stand-in for the Rust object (Fili/Files/Fili.qml). Headless: QT_QPA_PLATFORM=offscreen.
import QtQuick
import QtTest
import Fili.Files
import Fili.UI

Item {
    width: 300; height: 500
    readonly property bool kitReady: { FiliHost.backend = backend; return true; }
    QtObject {
        id: backend
        property bool light: false
        property string accent: "default"
        property int settingsRev: 0
        property bool keyNav: false
        function themeJson() {
            const x = new XMLHttpRequest();
            x.open("GET", Qt.resolvedUrl("theme.json"), false);
            x.send();
            return x.responseText;
        }
    }
    Sidebar { id: side; anchors.fill: parent }
    TestCase {
        name: "sidebar"
        when: windowShown
        function rowCenter(i) {
            // contentItem y of row i via the ListView
            const list = findList(side);
            const it = list.itemAtIndex(i);
            return it.mapToItem(side, it.width / 2, it.height / 2);
        }
        function findList(o) {
            for (let i = 0; i < o.children.length; ++i) {
                const c = o.children[i];
                if (c.hasOwnProperty("currentIndex")) return c;
            }
            return null;
        }
        function test_drag_down_reorders() {
            compare(Fili.favs.join(","), "/a,/b,/c,/d");
            const list = findList(side);
            // rows 0..3 are the favorites (the heading is a section delegate, not a row), row 4 is in Locations
            const a = rowCenter(0), c = rowCenter(2);
            mousePress(side, a.x, a.y);
            mouseMove(side, a.x, a.y + 10);
            mouseMove(side, c.x, c.y);
            verify(side.dragFrom === 0, "drag started on row 0, dragFrom=" + side.dragFrom);
            compare(side.dragTo, 2);
            mouseRelease(side, c.x, c.y);
            compare(Fili.favs.join(","), "/b,/c,/a,/d");
            compare(side.dragFrom, -1);
            compare(list.currentIndex, 2, "the cursor follows the moved favorite");
        }
        function test_a_click_is_not_a_drag() {
            Fili.calls = [];
            const b = rowCenter(1);
            const opened = [];
            side.open.connect(p => opened.push(p));
            mouseClick(side, b.x, b.y);
            compare(opened.length, 1, "a plain click still opens the place");
            compare(Fili.calls.length, 0);
        }
        function test_the_release_of_a_drag_does_not_open_the_place() {
            const opened = [];
            side.open.connect(p => opened.push(p));
            const x = rowCenter(3);
            mousePress(side, x.x, x.y);
            mouseMove(side, x.x, x.y - 12);
            mouseMove(side, x.x, rowCenter(1).y);
            mouseRelease(side, x.x, rowCenter(1).y);
            compare(opened.length, 0, "no navigation after a drag");
        }
        function test_it_stays_inside_the_section() {
            const a = rowCenter(1);
            const computer = rowCenter(4);
            mousePress(side, a.x, a.y);
            mouseMove(side, a.x, a.y + 10);
            mouseMove(side, computer.x, computer.y);
            verify(side.dragTo <= 3, "never past the last favorite, dragTo=" + side.dragTo);
            mouseRelease(side, computer.x, computer.y);
            compare(Fili.favs[Fili.favs.length - 1], Fili.favs[3]);
        }
    }
}
