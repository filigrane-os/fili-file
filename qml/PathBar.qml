// Chemin du dossier affiché, en segments cliquables (depuis le dossier personnel quand on y est, sinon depuis la
// racine). Ctrl+L le change en champ de saisie: Entrée y va, Échap annule.
import QtQuick
import Fili.Files

Item {
    id: bar
    property string path
    property bool editing: false
    signal open(string path)
    signal done()

    function edit() {
        input.text = path === Fili.home() ? "~/" : path.startsWith(Fili.home() + "/") ? "~" + path.slice(Fili.home().length) + "/" : path + "/";
        editing = true;
        input.forceActiveFocus();
        input.cursorPosition = input.text.length;
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
                height: 22
                radius: FiliTheme.radiusControl
                color: area.containsMouse && !crumb.last ? FiliTheme.nuit3 : "transparent"
                Text {
                    id: label
                    anchors.centerIn: parent
                    text: crumb.modelData.name
                    font.family: FiliTheme.font
                    font.pixelSize: FiliTheme.size
                    font.weight: crumb.last ? FiliTheme.weightAccent : Font.Normal
                    color: crumb.last ? FiliTheme.brume : FiliTheme.brume2
                }
                MouseArea {
                    id: area
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: bar.open(crumb.modelData.path)
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
            onAccepted: {
                bar.open(text);
            }
            Keys.onEscapePressed: bar.done()
            onActiveFocusChanged: if (!activeFocus) bar.editing = false
        }
    }
}
