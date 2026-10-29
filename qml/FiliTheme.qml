pragma Singleton
// Tokens du fili-shell (dotfiles/config/quickshell/FiliTheme.qml), sans Quickshell: mêmes noms, mêmes valeurs (vérifié
// par cargo test, tests::same_tokens_as_the_shell). Mode clair suivi en direct par Fili (état de fili-theme).
import QtQuick
import Fili.Files

QtObject {
    readonly property bool light: Fili.light

    readonly property color nuit: light ? "#f3f1ec" : "#1c222b"
    readonly property color nuitProfond: light ? "#f8f7f3" : "#181d25"
    readonly property color nuit2: light ? "#e9e6df" : "#232b36"
    readonly property color nuit3: light ? "#dfdbd2" : "#2d3744"
    readonly property color nuit4: light ? "#cdc8bd" : "#3c4858"
    readonly property color ardoise: "#5f7894"
    readonly property color brume2: light ? "#4f5e70" : "#a9b8c8"
    readonly property color voile: light ? "#3f5a78" : "#9dbbd6"
    readonly property color voileVif: light ? "#2f4a66" : "#b9d0e4"
    readonly property color brume: light ? "#1c222b" : "#dbe7f4"
    readonly property color sable: light ? "#80662b" : "#d2bf92"
    readonly property color cendre: light ? "#9a4f55" : "#c98f93"
    function alpha(c, a) {
        return Qt.rgba(c.r, c.g, c.b, a);
    }

    readonly property string font: "Jost"
    readonly property string mono: "JetBrains Mono"
    readonly property string icons: "Fili Icons"
    readonly property int size: 13
    readonly property int sizeList: 12
    readonly property int sizeSmall: 11
    readonly property int sizeLabel: 10
    readonly property int sizeTitle: 18
    readonly property int weightAccent: Font.DemiBold
    readonly property int radiusMark: 2
    readonly property int radiusControl: 6
    readonly property int radiusWindow: 10
    readonly property int space1: 4
    readonly property int space2: 8
    readonly property int space3: 12
    readonly property int space4: 16
    readonly property int space5: 20
    readonly property int space6: 24

    // Propres à fili-file (décision à valider): ligne de liste et de barre latérale, icône de liste, barres
    readonly property int rowHeight: 24
    readonly property int sideRowHeight: 28
    readonly property int iconSize: 16
    readonly property int toolbarHeight: 38
    readonly property int statusHeight: 22
    readonly property int sidebarWidth: 200
}
