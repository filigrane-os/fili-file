// Quick Look: every file of the folder in one overlay, ported from Conversations. Pictures, videos and sounds are
// played here; text and tables (monospace), PDF (first pages), folders (listing) and anything else (its facts) are
// read from Fili.quickLookContent (PDF: PdfView.qml). Left/Right go to the previous/next one, Space plays or pauses a video or a sound
// (closes for the others), Esc closes.
import QtQuick
import QtMultimedia
import Fili.Files

Rectangle {
    id: ql
    z: 220
    visible: false
    color: FiliTheme.alpha(FiliTheme.backgroundDeep, 0.92)
    focus: visible

    property string path: ""    // the file shown: Main keeps it on the current row of the list
    property int rowNumber: 0     // its row (from 1) and the number of rows, for the counter
    property int rowCount: 0
    property string playerError: ""
    property bool autoplay: true
    readonly property string kind: path !== "" ? kindOf(path) : ""
    readonly property bool isPicture: kind === "image"
    readonly property bool isVideo: kind === "video"
    readonly property bool isAudio: kind === "audio"
    readonly property bool playable: isVideo || isAudio
    readonly property bool selectable: docKind === "pdf" || docKind === "text" || docKind === "folder"
    // Content of the other kinds (see Fili.quickLookContent)
    property string docKind: ""
    property string docText: ""
    property var facts: ({ size: 0, mime: "" })
    signal closed()

    // "image", "video", "audio", else "doc"
    function kindOf(p) {
        const n = p.toLowerCase();
        if (/\.(png|jpe?g|gif|webp|apng|bmp|svg|avif|heic|tiff?)$/.test(n)) return "image";
        if (/\.(mp4|mkv|webm|mov|m4v|avi|ogv)$/.test(n)) return "video";
        if (/\.(mp3|flac|ogg|oga|opus|m4a|wav|aac)$/.test(n)) return "audio";
        return "doc";
    }
    // Shows the file `p` (the one selected in Casier); the arrow keys are handed to the list (signal `navigate`),
    // which moves the selection, and Main calls show() again
    function show(file, number, count) {
        ql.path = file;
        ql.rowNumber = number;
        ql.rowCount = count;
        if (!visible) {
            visible = true;
            forceActiveFocus();
        }
    }
    function close() { player.stop(); visible = false; closed(); }
    // Copy and Select All of the window (Ctrl+C, Ctrl+A, menu bar) act on the text shown here while it is open:
    // the shortcuts of the window come before the keys of the overlay
    function copySelection() {
        if (docKind === "pdf") pdfView.copySelection();
        else if (docView.visible && docLabel.selectedText !== "") Fili.copyText(docLabel.selectedText);
    }
    function selectAll() {
        if (docKind === "pdf") pdfView.selectAll();
        else if (docView.visible) docLabel.selectAll();
    }
    function sizeText(n) {
        if (n < 1024) return n + " B";
        if (n < 1048576) return Math.round(n / 1024) + " KB";
        if (n < 1073741824) return (n / 1048576).toFixed(1) + " MB";
        return (n / 1073741824).toFixed(1) + " GB";
    }
    // Reads what the current file shows when it is not a picture, a video or a sound
    function load() {
        docKind = "";
        docText = "";
        if (path === "" || kindOf(path) !== "doc") return;
        const r = Fili.quickLookContent(path);
        const nl = r.indexOf("\n");
        const k = nl < 0 ? r : r.slice(0, nl);
        const body = nl < 0 ? "" : r.slice(nl + 1);
        if (k === "other") { const l = body.split("\n"); facts = { size: parseInt(l[0]) || 0, mime: l[1] || "" }; }
        else docText = body;
        docKind = k;
        docView.contentY = 0;
    }
    function clock(ms) {
        const s = Math.max(0, Math.floor(ms / 1000));
        return Math.floor(s / 60) + ":" + String(s % 60).padStart(2, "0");
    }
    function togglePlay() {
        autoplay = false;
        if (player.playbackState === MediaPlayer.PlayingState) player.pause(); else player.play();
    }
    onPathChanged: {
        player.stop();
        autoplay = true;
        playerError = "";
        // kindOf, not `kind`: the bindings that depend on `path` may not have run yet
        const k = path !== "" ? kindOf(path) : "";
        player.source = k === "video" || k === "audio" ? "file://" + encodeURI(path) : "";
        load();
    }

    signal navigate(var key, var modifiers)
    Keys.onEscapePressed: close()
    Keys.onSpacePressed: playable ? togglePlay() : close()
    Keys.onPressed: e => {
        if (docKind === "pdf" && (e.modifiers & Qt.ControlModifier)) {
            if (pdfView.handleKey(e.key)) { pdfView.zoomed = true; e.accepted = true; return; }
        }
        if ([Qt.Key_Up, Qt.Key_Down, Qt.Key_Left, Qt.Key_Right, Qt.Key_Home, Qt.Key_End, Qt.Key_PageUp, Qt.Key_PageDown].indexOf(e.key) >= 0) {
            ql.navigate(e.key, e.modifiers);
            e.accepted = true;
        }
    }

    MediaPlayer {
        id: player
        onMediaStatusChanged: if (mediaStatus === MediaPlayer.LoadedMedia && ql.autoplay && ql.visible) play()
        onErrorOccurred: (error, text) => ql.playerError = text
        videoOutput: video
        audioOutput: AudioOutput { }
    }

    // A click on the background closes it, except over text and PDF (a drag selects there). Always present: it keeps
    // every mouse event from reaching the list behind the overlay
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; onClicked: if (!ql.selectable) ql.close() }

    Item {
        id: top
        anchors { left: parent.left; right: parent.right; top: parent.top }
        height: 44
        Text {
            anchors { left: parent.left; leftMargin: FiliTheme.margin; right: counter.left; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
            text: ql.path.split("/").pop()
            elide: Text.ElideMiddle
            textFormat: Text.PlainText
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.size
            font.weight: FiliTheme.weightAccent
            color: FiliTheme.text
        }
        Text {
            id: counter
            anchors { right: closeButton.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
            text: ql.rowCount > 1 ? ql.rowNumber + " / " + ql.rowCount : ""
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeList
            color: FiliTheme.muted
        }
        Rectangle {
            id: closeButton
            anchors { right: parent.right; rightMargin: FiliTheme.margin; verticalCenter: parent.verticalCenter }
            width: 28
            height: 24
            radius: FiliTheme.radiusControl
            color: closeArea.pressed ? FiliTheme.surfaceActive : closeArea.containsMouse ? FiliTheme.surfaceHover : "transparent"
            Text { anchors.centerIn: parent; text: "✕"; font.pixelSize: 12; color: FiliTheme.text }
            MouseArea { id: closeArea; anchors.fill: parent; hoverEnabled: true; onClicked: ql.close() }
        }
    }

    Item {
        id: stage
        anchors { left: parent.left; right: parent.right; top: top.bottom; bottom: bar.top; leftMargin: 64; rightMargin: 64 }
        MouseArea { anchors.fill: parent; enabled: !ql.selectable; onClicked: ql.playable ? ql.togglePlay() : ql.close() }
        readonly property bool animatedFile: /\.(gif|webp|apng)$/i.test(ql.path)
        Image {
            anchors.fill: parent
            visible: ql.isPicture && !stage.animatedFile
            source: visible ? "file://" + encodeURI(ql.path) : ""
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            cache: false
            autoTransform: true
            sourceSize.width: 2400
        }
        AnimatedImage {
            anchors.fill: parent
            playing: visible
            visible: ql.isPicture && stage.animatedFile
            source: visible ? "file://" + encodeURI(ql.path) : ""
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            cache: false
            sourceSize.width: 2400
        }
        VideoOutput {
            id: video
            anchors.fill: parent
            visible: ql.isVideo
        }
        Text {
            anchors.centerIn: parent
            visible: ql.isAudio
            text: "♪"
            font.pixelSize: 64
            color: FiliTheme.accentColor
        }
        // Text, table, folder listing (the text can be selected, Ctrl+C copies)
        Flickable {
            id: docView
            anchors.fill: parent
            visible: ql.docKind === "text" || ql.docKind === "folder"
            clip: true
            contentWidth: width
            contentHeight: docLabel.implicitHeight + 2 * FiliTheme.space3
            boundsBehavior: Flickable.StopAtBounds
            KineticScroll {}
            TextEdit {
                id: docLabel
                readOnly: true
                selectByMouse: true
                selectionColor: FiliTheme.accentColor
                x: FiliTheme.space3
                y: FiliTheme.space3
                width: parent.width - 2 * FiliTheme.space3 - 14
                text: ql.docText
                textFormat: TextEdit.PlainText
                wrapMode: TextEdit.WrapAnywhere
                font.family: FiliTheme.mono
                font.pixelSize: FiliTheme.sizeList
                color: FiliTheme.text
            }
        }
        // PDF: pages drawn by QtPdf, the text can be selected with the mouse
        PdfView {
            id: pdfView
            anchors.fill: parent
            visible: ql.docKind === "pdf"
            source: ql.docKind === "pdf" ? "file://" + encodeURI(ql.path) : ""
        }
        FiliScrollBar { flick: docView; anchors { top: docView.top; bottom: docView.bottom; right: docView.right } visible: needed && docView.visible }
        // A file that cannot be shown: its facts
        Column {
            anchors.centerIn: parent
            visible: ql.docKind === "other"
            spacing: FiliTheme.space2
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: "▣"
                font.pixelSize: 64
                color: FiliTheme.accentColor
            }
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: ql.path.split("/").pop()
                textFormat: Text.PlainText
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                color: FiliTheme.text
            }
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: ql.sizeText(ql.facts.size) + (ql.facts.mime !== "" ? " · " + ql.facts.mime : "")
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: FiliTheme.muted
            }
        }
        Text {
            anchors.centerIn: parent
            visible: ql.playerError !== ""
            text: ql.playerError
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeList
            color: FiliTheme.muted
        }
    }

    // Player bar (video and sound)
    Item {
        id: bar
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: ql.playable ? 44 : 0
        visible: height > 0
        Row {
            id: playRow
            anchors { left: parent.left; leftMargin: FiliTheme.margin; verticalCenter: parent.verticalCenter }
            spacing: FiliTheme.space2
            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: 20
                text: player.playbackState === MediaPlayer.PlayingState ? "❚❚" : "▶"
                font.pixelSize: FiliTheme.size
                color: FiliTheme.text
                MouseArea { anchors.fill: parent; onClicked: ql.togglePlay() }
            }
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: ql.clock(player.position)
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeSmall
                color: FiliTheme.muted
            }
        }
        Text {
            id: total
            anchors { right: parent.right; rightMargin: FiliTheme.margin; verticalCenter: parent.verticalCenter }
            text: ql.clock(player.duration)
            font.family: FiliTheme.font
            font.pixelSize: FiliTheme.sizeSmall
            color: FiliTheme.muted
        }
        Rectangle {
            anchors { left: playRow.right; leftMargin: FiliTheme.space3; right: total.left; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
            height: 4
            radius: 2
            color: FiliTheme.surface
            Rectangle {
                width: player.duration > 0 ? parent.width * player.position / player.duration : 0
                height: parent.height
                radius: 2
                color: FiliTheme.accentColor
            }
            MouseArea {
                anchors { fill: parent; topMargin: -10; bottomMargin: -10 }
                onPressed: mouse => player.position = Math.max(0, Math.min(1, mouse.x / width)) * player.duration
                onPositionChanged: mouse => { if (pressed) player.position = Math.max(0, Math.min(1, mouse.x / width)) * player.duration; }
            }
        }
    }
}
