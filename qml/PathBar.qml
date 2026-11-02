// Chemin du dossier affiché, en segments cliquables (depuis le dossier personnel quand on y est, sinon depuis la
// racine). Ctrl+L le change en champ de saisie: Entrée y va, Échap annule.
import QtQuick
import Fili.Files

Item {
    id: bar
    property string path
    property bool editing: false
    property int fontSize: FiliTheme.size
    signal open(string path)
    // Clic droit sur un segment: chemin complet et position dans la fenêtre
    signal context(string path, real x, real y)
    signal done()

    // Propositions façon fzf pour ce qui est tapé (Haut/Bas choisissent, Tab complète, Entrée valide la proposition choisie,
    // la première par défaut après une frappe, ou, sans choix, le texte tel quel)
    property var suggestions: []
    property int selected: -1
    function shown(p) {
        const h = Fili.home();
        return p === h ? "~" : p.startsWith(h + "/") ? "~" + p.slice(h.length) : p;
    }
    // preselect: après une frappe, la première proposition est choisie d'avance (Entrée la valide sans les flèches),
    // sauf quand le texte finit par « / » (dossier complet: Entrée y va tel quel, les flèches choisissent un contenu)
    function refresh(preselect) {
        suggestions = editing ? JSON.parse(Fili.pathSuggest(input.text)) : [];
        selected = preselect === true && suggestions.length > 0 && !input.text.endsWith("/") ? 0 : -1;
    }
    // Remplace le texte par la proposition i (un « / » en fin pour un dossier, où l'on continue de taper)
    function accept(i) {
        const c = suggestions[i];
        input.text = shown(c.path) + (c.dir ? "/" : "");
        input.cursorPosition = input.text.length;
        refresh();
    }
    onEditingChanged: if (!editing) suggestions = []

    function edit() {
        input.text = path === Fili.home() ? "~/" : path.startsWith(Fili.home() + "/") ? "~" + path.slice(Fili.home().length) + "/" : path + "/";
        editing = true;
        input.forceActiveFocus();
        input.cursorPosition = input.text.length;
        refresh();
    }

    readonly property var crumbs: {
        const home = Fili.home();
        const out = [];
        let base = "";
        let rest = path;
        if (path === home || path.startsWith(home + "/")) {
            out.push({ name: home.split("/").pop(), path: home });
            base = home;
            rest = path.slice(home.length);
        } else {
            out.push({ name: "/", path: "/" });
        }
        for (const part of rest.split("/").filter(s => s.length > 0)) {
            base += "/" + part;
            out.push({ name: part, path: base });
        }
        return out;
    }

    ListView {
        id: crumbList
        anchors.fill: parent
        visible: !bar.editing
        orientation: ListView.Horizontal
        interactive: false
        clip: true
        model: bar.crumbs
        spacing: 0
        onCountChanged: positionViewAtEnd()
        onWidthChanged: positionViewAtEnd()
        delegate: Row {
            id: crumb
            required property var modelData
            required property int index
            readonly property bool last: index === crumbList.count - 1
            height: crumbList.height
            Text {
                visible: crumb.index > 0
                anchors.verticalCenter: parent.verticalCenter
                width: 14
                horizontalAlignment: Text.AlignHCenter
                text: ""
                font.family: FiliTheme.icons
                font.pixelSize: 9
                color: FiliTheme.ardoise
            }
            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: label.implicitWidth + 2 * FiliTheme.space2
                height: Math.min(22, bar.height)
                radius: FiliTheme.radiusControl
                color: area.containsMouse && !crumb.last ? FiliTheme.nuit3 : "transparent"
                Text {
                    id: label
                    anchors.centerIn: parent
                    text: crumb.modelData.name
                    font.family: FiliTheme.font
                    font.pixelSize: bar.fontSize
                    font.weight: crumb.last ? FiliTheme.weightAccent : Font.Normal
                    color: crumb.last ? FiliTheme.brume : FiliTheme.brume2
                }
                MouseArea {
                    id: area
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: e => {
                        if (e.button === Qt.RightButton) {
                            const p = area.mapToItem(null, e.x, e.y);
                            bar.context(crumb.modelData.path, p.x, p.y);
                        } else
                            bar.open(crumb.modelData.path);
                    }
                }
            }
        }
    }

    Rectangle {
        anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter }
        height: 24
        visible: bar.editing
        radius: FiliTheme.radiusControl
        color: FiliTheme.nuit2
        border.width: 1
        border.color: FiliTheme.voile
        TextInput {
            id: input
            anchors { fill: parent; leftMargin: FiliTheme.space2; rightMargin: FiliTheme.space2 }
            verticalAlignment: TextInput.AlignVCenter
            clip: true
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.size
            color: FiliTheme.brume
            selectionColor: FiliTheme.nuit4
            selectedTextColor: FiliTheme.brume
            onTextEdited: bar.refresh(true)
            onAccepted: {
                if (bar.selected >= 0) {
                    const c = bar.suggestions[bar.selected];
                    bar.open(c.path);
                } else
                    bar.open(text);
            }
            Keys.onEscapePressed: bar.done()
            Keys.onDownPressed: if (bar.suggestions.length > 0) bar.selected = (bar.selected + 1) % bar.suggestions.length
            Keys.onUpPressed: if (bar.suggestions.length > 0) bar.selected = bar.selected <= 0 ? bar.suggestions.length - 1 : bar.selected - 1
            Keys.onTabPressed: e => {
                if (bar.suggestions.length > 0)
                    bar.accept(bar.selected >= 0 ? bar.selected : 0);
                e.accepted = true;
            }
            onActiveFocusChanged: if (!activeFocus) bar.editing = false
        }
    }

    // Liste des propositions, au-dessus du champ (la barre d'état est en bas de la fenêtre)
    Rectangle {
        visible: bar.editing && bar.suggestions.length > 0
        anchors { left: parent.left; right: parent.right; bottom: parent.top; bottomMargin: FiliTheme.space2 }
        height: Math.min(bar.suggestions.length, 8) * 26 + 2 * FiliTheme.space1
        radius: FiliTheme.radiusControl
        color: FiliTheme.nuit2
        border.width: 1
        border.color: FiliTheme.nuit4
        ListView {
            id: sugList
            anchors { fill: parent; margins: FiliTheme.space1 }
            clip: true
            model: bar.suggestions
            interactive: false
            currentIndex: bar.selected
            onCurrentIndexChanged: positionViewAtIndex(currentIndex < 0 ? 0 : currentIndex, ListView.Contain)
            delegate: Rectangle {
                required property var modelData
                required property int index
                width: sugList.width
                height: 26
                radius: FiliTheme.radiusControl
                color: index === bar.selected || sugArea.containsMouse ? FiliTheme.nuit3 : "transparent"
                Text {
                    anchors { left: parent.left; leftMargin: FiliTheme.space2; right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                    text: modelData.name + (modelData.dir ? "/" : "")
                    elide: Text.ElideRight
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    color: modelData.dir ? FiliTheme.brume : FiliTheme.brume2
                }
                MouseArea {
                    id: sugArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: bar.open(modelData.path)
                }
            }
        }
    }
}
