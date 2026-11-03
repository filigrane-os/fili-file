// Panneau d'aperçu à droite (Ctrl+Maj+P): image, première page d'un PDF, début d'un texte, contenu d'un dossier.
// Suit la sélection (un seul élément), avec un court délai pour ne pas travailler pendant qu'on fait défiler.
import QtQuick
import Fili.Files

Rectangle {
    id: pane
    // Chemin de l'élément sélectionné (vide: aucun ou plusieurs)
    property string path: ""
    color: FiliTheme.background

    property string kind: "none"
    property string body: ""
    onPathChanged: delay.restart()
    onVisibleChanged: if (visible) delay.restart()
    Timer {
        id: delay
        interval: 120
        onTriggered: {
            if (!pane.visible || pane.path === "") {
                pane.kind = "none";
                pane.body = "";
                return;
            }
            const r = Fili.previewInfo(pane.path);
            const i = r.indexOf("\n");
            pane.kind = r.substring(0, i);
            pane.body = r.substring(i + 1);
        }
    }

    Image {
        visible: pane.kind === "image"
        anchors { fill: parent; margins: FiliTheme.space4 }
        source: pane.kind === "image" ? pane.body : ""
        sourceSize: Qt.size(960, 960)
        fillMode: Image.PreserveAspectFit
        asynchronous: true
        cache: false
    }
    Flickable {
        visible: pane.kind === "text" || pane.kind === "folder"
        anchors { fill: parent; margins: FiliTheme.space4 }
        contentWidth: width
        contentHeight: txt.implicitHeight
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        Text {
            id: txt
            width: parent.width
            text: pane.kind === "text" || pane.kind === "folder" ? pane.body : ""
            wrapMode: pane.kind === "text" ? Text.Wrap : Text.NoWrap
            elide: Text.ElideRight
            font.family: pane.kind === "text" ? FiliTheme.mono : FiliTheme.font
            font.pixelSize: pane.kind === "text" ? FiliTheme.sizeSmall : FiliTheme.sizeList
            lineHeight: 1.2
            color: FiliTheme.text
        }
    }
    Text {
        visible: pane.kind === "none" || (pane.kind === "folder" && pane.body === "")
        anchors.centerIn: parent
        text: pane.kind === "folder" ? Fili.tr("Empty folder") : pane.path === "" ? Fili.tr("No preview") : Fili.tr("No preview available")
        font.family: FiliTheme.font
        font.pixelSize: FiliTheme.sizeList
        color: FiliTheme.muted
    }
}
