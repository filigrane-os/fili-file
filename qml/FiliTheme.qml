pragma Singleton
// Tokens du fili-shell (dotfiles/config/quickshell/FiliTheme.qml), sans Quickshell: mêmes noms, mêmes valeurs (vérifié
// par cargo test, tests::same_tokens_as_the_shell). Mode clair suivi en direct par Fili (état de fili-theme).
import QtQuick
import Fili.Files

QtObject {
    readonly property bool light: Fili.light
    readonly property string accent: Fili.accent

    readonly property color nuit: light ? "#f3f1ec" : "#1c222b"
    readonly property color nuitProfond: light ? "#f8f7f3" : "#181d25"
    readonly property color nuit2: light ? "#e9e6df" : "#232b36"
    readonly property color nuit3: light ? "#dfdbd2" : "#2d3744"
    readonly property color nuit4: light ? "#cdc8bd" : "#3c4858"
    readonly property color ardoise: "#5f7894"
    readonly property color brume2: light ? "#4f5e70" : "#a9b8c8"
    // Couleur d'accentuation (fili-theme accent): [sombre, sombre vif, clair, clair vif]; défaut = brume en sombre, noir (nuit) en clair
    readonly property var accentTones: ({ default: ["#dbe7f4", "#f3f1ec", "#1c222b", "#0f1319"], voile: ["#9dbbd6", "#b9d0e4", "#3f5a78", "#2f4a66"], ardoise: ["#7f9ab6", "#9ab2ca", "#4f6884", "#3d5470"], sauge: ["#a3bfa8", "#bcd3c0", "#4a7354", "#385a42"], sable: ["#d2bf92", "#e0d1ab", "#80662b", "#654f1f"], cendre: ["#c98f93", "#d9a8ab", "#9a4f55", "#7d3c42"], brume: ["#dbe7f4", "#f3f1ec", "#4f5e70", "#1c222b"] })
    readonly property var tone: accentTones[accent] || accentTones.default
    readonly property color voile: light ? tone[2] : tone[0]
    readonly property color voileVif: light ? tone[3] : tone[1]
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
    // Marge extérieure de la fenêtre: capsule de navigation, glyphes et intertitres de la barre latérale, fil d'Ariane,
    // recherche et information de sélection s'alignent dessus
    readonly property int margin: 24
    readonly property int toolbarHeight: 44
    readonly property int statusHeight: 28
    readonly property int sidebarWidth: 200
}
