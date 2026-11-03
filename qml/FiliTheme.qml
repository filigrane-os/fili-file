pragma Singleton
// Tokens du fili-shell (dotfiles/config/quickshell/FiliTheme.qml), sans Quickshell: mêmes noms; valeurs par défaut
// identiques (theme.toml, vérifié par cargo test, tests::default_theme_matches_the_shell), personnalisables. Mode clair suivi en direct par Fili (état de fili-theme).
import QtQuick
import Fili.Files

QtObject {
    readonly property bool light: Fili.light
    readonly property string accent: Fili.accent

    // Thème fusionné: défauts de theme.toml surchargés par ~/.config/filigrane/fili-file.toml (sections [theme],
    // [theme.dark], [theme.light], [theme.accents]); les clés sont en anglais, les noms de jetons restent ceux du shell
    readonly property var theme: JSON.parse(Fili.themeJson())
    readonly property var m: light ? theme.light : theme.dark
    readonly property color background: m.background
    readonly property color backgroundDeep: m.background_deep
    readonly property color surface: m.surface
    readonly property color surfaceHover: m.surface_hover
    readonly property color surfaceActive: m.surface_active
    readonly property color border: m.border
    readonly property color muted: m.muted
    readonly property color textSecondary: m.text_secondary
    // Couleur d'accentuation (fili-theme accent): [sombre, sombre vif, clair, clair vif]; défaut = brume en sombre, noir (nuit) en clair.
    // Un `accent` fixé dans le mode passe devant.
    readonly property var accentTones: theme.accents
    readonly property var tone: accentTones[accent] || accentTones.default
    readonly property color accentColor: m.accent || (light ? tone[2] : tone[0])
    readonly property color accentBright: m.accent_bright || (light ? tone[3] : tone[1])
    readonly property color text: m.text
    readonly property color warning: m.warning
    readonly property color danger: m.danger
    function alpha(c, a) {
        return Qt.rgba(c.r, c.g, c.b, a);
    }

    readonly property string font: m.font
    readonly property string mono: m.mono
    readonly property string icons: m.icons
    readonly property int size: m.size
    readonly property int sizeList: m.size_list
    readonly property int sizeSmall: m.size_small
    readonly property int sizeLabel: m.size_label
    readonly property int sizeTitle: m.size_title
    readonly property int weightAccent: m.weight_accent
    readonly property int radiusMark: m.radius_mark
    readonly property int radiusControl: m.radius_control
    readonly property int radiusWindow: m.radius_window
    readonly property int borderWidth: m.border_width
    readonly property int space1: m.space_1
    readonly property int space2: m.space_2
    readonly property int space3: m.space_3
    readonly property int space4: m.space_4
    readonly property int space5: m.space_5
    readonly property int space6: m.space_6

    // Propres à fili-file (décision à valider): ligne de liste et de barre latérale, icône de liste, barres
    readonly property int rowHeight: m.row_height
    readonly property int sideRowHeight: m.side_row_height
    readonly property int iconSize: m.icon_size
    // Marge extérieure de la fenêtre: capsule de navigation, glyphes et intertitres de la barre latérale, fil d'Ariane,
    // recherche et information de sélection s'alignent dessus
    readonly property int margin: m.margin
    readonly property int toolbarHeight: m.toolbar_height
    readonly property int statusHeight: m.status_height
    readonly property int sidebarWidth: m.sidebar_width
}
