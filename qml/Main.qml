// Fenêtre de fili-file: barre d'outils (précédent, suivant, chemin), barre latérale, liste, barre d'état.
// Raccourcis: Alt+← / Alt+→ historique, Alt+↑ parent, Ctrl+L chemin, Ctrl+H fichiers cachés, F5 relire,
// Ctrl+W ou Ctrl+Q fermer; boutons précédent/suivant de la souris.
import QtQuick
import Fili.Files

Window {
    id: win
    width: 900
    height: 560
    minimumWidth: 520
    minimumHeight: 300
    visible: true
    color: FiliTheme.nuit
    title: folder.path === "/" ? "/" : folder.path.split("/").pop()

    FolderModel { id: folder }

    function show(row) {
        if (row >= 0)
            files.select(row);
    }
    function open(path) {
        const row = folder.go(path);
        if (row >= 0 || folder.error === "") {
            pathBar.editing = false;
            show(row);
            files.focusList();
        }
    }

    Component.onCompleted: {
        show(folder.go(Fili.startPath()));
        files.focusList();
    }

    Shortcut { sequence: "Alt+Left"; onActivated: win.show(folder.back()) }
    Shortcut { sequence: "Alt+Right"; onActivated: win.show(folder.forward()) }
    Shortcut { sequence: "Ctrl+["; onActivated: win.show(folder.back()) }
    Shortcut { sequence: "Ctrl+]"; onActivated: win.show(folder.forward()) }
    Shortcut { sequence: "Ctrl+L"; onActivated: pathBar.edit() }
    Shortcut { sequence: "Ctrl+H"; onActivated: win.show(folder.toggleHidden(files.currentIndex)) }
    Shortcut { sequence: "F5"; onActivated: win.show(folder.refresh(files.currentIndex)) }
    Shortcut { sequence: "Ctrl+R"; onActivated: win.show(folder.refresh(files.currentIndex)) }
    Shortcut { sequences: ["Ctrl+W", "Ctrl+Q"]; onActivated: Qt.quit() }

    // Boutons précédent/suivant de la souris, partout dans la fenêtre
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.BackButton | Qt.ForwardButton
        onClicked: e => win.show(e.button === Qt.BackButton ? folder.back() : folder.forward())
    }

    Rectangle {
        id: toolbar
        width: parent.width
        height: FiliTheme.toolbarHeight
        color: FiliTheme.nuit
        Row {
            id: nav
            x: FiliTheme.space2
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2
            IconButton {
                glyph: ""
                enabledState: folder.canBack
                onClicked: win.show(folder.back())
            }
            IconButton {
                glyph: ""
                enabledState: folder.canForward
                onClicked: win.show(folder.forward())
            }
        }
        PathBar {
            id: pathBar
            anchors { left: nav.right; leftMargin: FiliTheme.space2; right: parent.right; rightMargin: FiliTheme.space3; top: parent.top; bottom: parent.bottom }
            path: folder.path
            onOpen: p => win.open(p)
            onDone: {
                editing = false;
                files.focusList();
            }
        }
    }
    Rectangle {
        id: toolbarLine
        anchors.top: toolbar.bottom
        width: parent.width
        height: 1
        color: FiliTheme.nuit4
    }

    Sidebar {
        id: sidebar
        anchors { top: toolbarLine.bottom; bottom: statusbar.top; left: parent.left }
        width: FiliTheme.sidebarWidth
        current: folder.path
        onOpen: p => win.open(p)
    }
    Rectangle {
        id: sideLine
        anchors { top: toolbarLine.bottom; bottom: statusbar.top; left: sidebar.right }
        width: 1
        color: FiliTheme.nuit4
    }

    FileList {
        id: files
        anchors { top: toolbarLine.bottom; bottom: statusbar.top; left: sideLine.right; right: parent.right }
        folder: folder
        windowActive: win.active
        onOpenRow: row => win.show(folder.activate(row))
        onGoUp: win.show(folder.up())
    }

    // Barre d'état: nombre d'éléments et espace libre, ou l'erreur de la dernière action
    Rectangle {
        id: statusbar
        anchors.bottom: parent.bottom
        width: parent.width
        height: FiliTheme.statusHeight
        color: FiliTheme.nuit
        Rectangle {
            width: parent.width
            height: 1
            color: FiliTheme.nuit4
        }
        Text {
            anchors { fill: parent; leftMargin: FiliTheme.space3; rightMargin: FiliTheme.space3 }
            verticalAlignment: Text.AlignVCenter
            horizontalAlignment: Text.AlignHCenter
            elide: Text.ElideMiddle
            text: folder.error || folder.status
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeSmall
            color: folder.error ? FiliTheme.cendre : FiliTheme.brume2
        }
    }
}
