// « À proximité »: le partage de proximité de Casier, à la manière d'AirDrop dans le Finder. Les appareils joignables
// (LocalSend sur le réseau local, Bluetooth appairés) sont des pastilles: on y dépose des fichiers (glisser depuis une
// autre fenêtre) ou on clique pour choisir les fichiers à envoyer. En dessous, qui peut nous envoyer des fichiers
// (interrupteurs par protocole) et les derniers fichiers reçus. Les demandes entrantes arrivent en notification.
import QtQuick
import Fili.Files

Rectangle {
    id: view
    color: FiliTheme.nuit
    // Message passager de la fenêtre
    signal notice(string text)
    // Ouvrir un fichier reçu dans le dossier
    signal openPath(string path)

    property var info: ({ dest: "", confirm: "ask", minutes: 0, name: "", protocols: [] })
    property var received: []
    property var peers: ({ localsend: [], bluetooth: [] })
    property int pending: 0
    readonly property var protocols: info.protocols.filter(p => p.id === "bluetooth" || p.id === "localsend")
    readonly property bool anyOn: protocols.some(p => p.on)
    readonly property var devices: {
        const out = [];
        for (const p of protocols) {
            if (!p.available)
                continue;
            for (const d of (peers[p.id] || []))
                out.push({ proto: p.id, label: p.label, id: d.id, name: d.name });
        }
        return out;
    }

    // Un tour = infos de réception (protocoles disponibles), puis un scan par protocole. Pas de nouveau tour tant que
    // le précédent n'est pas fini; les réponses portent l'étiquette de leur tour, les autres sont ignorées.
    property int round: 0
    property bool wantScan: false
    function refresh() {
        if (pending > 0 || wantScan)
            return;
        wantScan = true;
        Fili.receiveInfo();
    }
    function scan() {
        wantScan = false;
        const ids = protocols.filter(p => p.available).map(p => p.id);
        round++;
        pending = ids.length;
        ids.forEach(id => Fili.sharePeers(id, "nearby" + round));
    }
    function when(t) {
        return Qt.formatDateTime(new Date(t * 1000), "d MMM, hh:mm");
    }
    // Seuls les fichiers locaux (file://) sont envoyés; un nom avec saut de ligne ne passerait pas par la liste
    function pathsOf(urls) {
        const out = [];
        for (const u of urls) {
            const s = u.toString();
            if (!s.startsWith("file://"))
                continue;
            try {
                const p = decodeURIComponent(s.slice(7));
                if (p.indexOf("\n") < 0)
                    out.push(p);
            } catch (e) {}
        }
        return out;
    }
    function send(device, urls) {
        const paths = pathsOf(urls);
        if (paths.length === 0) {
            view.notice(Fili.tr("Nothing to send: drop files from a folder."));
            return;
        }
        Fili.shareSend(device.proto, device.id, device.name, paths.join("\n"));
        view.notice(Fili.tr("Sending to %1…").replace("%1", device.name));
    }

    onVisibleChanged: if (visible) refresh()
    // Les appareils vont et viennent: nouvelle recherche toutes les 10 s tant que la vue est ouverte
    Timer {
        interval: 10000
        running: view.visible
        repeat: true
        onTriggered: view.refresh()
    }
    Connections {
        target: Fili
        function onReceiveInfoReady(info, received) {
            try {
                const i = JSON.parse(info);
                if (i.protocols)
                    view.info = i;
                view.received = JSON.parse(received);
            } catch (e) {}
            if (view.wantScan)
                view.scan();
        }
        function onSharePeersReady(proto, tag, json) {
            if (tag !== "nearby" + view.round || view.pending === 0)
                return;
            const p = Object.assign({}, view.peers);
            try {
                p[proto] = JSON.parse(json);
            } catch (e) {
                p[proto] = [];
            }
            view.peers = p;
            view.pending--;
        }
    }

    Flickable {
        anchors.fill: parent
        contentWidth: width
        contentHeight: Math.max(height, column.implicitHeight + 2 * FiliTheme.space6)
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: column
            width: Math.min(parent.width - 2 * FiliTheme.margin, 560)
            x: (parent.width - width) / 2
            y: FiliTheme.space6
            spacing: FiliTheme.space4

            Text {
                text: Fili.tr("Nearby")
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeTitle
                font.weight: FiliTheme.weightAccent
                color: FiliTheme.brume
            }
            Text {
                width: parent.width
                wrapMode: Text.Wrap
                text: Fili.tr("Drop files on a device to send them, or click a device to choose the files. LocalSend finds devices on this network; Bluetooth uses paired devices.")
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: FiliTheme.brume2
            }

            // Appareils
            Flow {
                width: parent.width
                spacing: FiliTheme.space3
                Repeater {
                    model: view.devices
                    Item {
                        id: dev
                        required property var modelData
                        width: 108
                        height: 112
                        Rectangle {
                            id: bubble
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: 64
                            height: 64
                            radius: 32
                            color: drop.containsDrag || hit.containsMouse ? FiliTheme.nuit4 : FiliTheme.nuit3
                            border.width: drop.containsDrag ? 2 : 1
                            border.color: drop.containsDrag ? FiliTheme.voile : FiliTheme.nuit4
                            Text {
                                anchors.centerIn: parent
                                text: dev.modelData.name.charAt(0).toUpperCase()
                                font.family: FiliTheme.font
                                font.pixelSize: FiliTheme.sizeTitle + 6
                                font.weight: FiliTheme.weightAccent
                                color: FiliTheme.voile
                            }
                        }
                        Text {
                            anchors { top: bubble.bottom; topMargin: FiliTheme.space2; horizontalCenter: parent.horizontalCenter }
                            width: parent.width
                            horizontalAlignment: Text.AlignHCenter
                            elide: Text.ElideRight
                            text: dev.modelData.name
                            font.family: FiliTheme.font
                            font.pixelSize: FiliTheme.size
                            color: FiliTheme.brume
                        }
                        Text {
                            anchors { bottom: parent.bottom; horizontalCenter: parent.horizontalCenter }
                            text: dev.modelData.label
                            font.family: FiliTheme.font
                            font.pixelSize: FiliTheme.sizeSmall
                            color: FiliTheme.ardoise
                        }
                        DropArea {
                            id: drop
                            anchors.fill: parent
                            onDropped: d => {
                                view.send(dev.modelData, d.urls);
                                d.acceptProposedAction();
                            }
                        }
                        MouseArea {
                            id: hit
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: {
                                Fili.sharePick(dev.modelData.proto, dev.modelData.id, dev.modelData.name);
                            }
                        }
                    }
                }
            }
            Text {
                visible: view.devices.length === 0
                width: parent.width
                wrapMode: Text.Wrap
                text: view.pending > 0 ? Fili.tr("Searching…")
                      : Fili.tr("No device found. LocalSend must be open on the other device, or pair it in the Bluetooth panel.")
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: FiliTheme.brume2
            }

            Rectangle { width: parent.width; height: 1; color: FiliTheme.nuit4 }

            // Réception: qui peut nous trouver
            Text {
                text: Fili.tr("Allow others to find this computer")
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                font.weight: FiliTheme.weightAccent
                color: FiliTheme.brume
            }
            Text {
                width: parent.width
                wrapMode: Text.Wrap
                text: view.anyOn ? Fili.tr("Visible as “%1”. Each transfer asks for your approval.").replace("%1", view.info.name)
                                 : Fili.tr("Off: this computer is not visible to other devices.")
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: FiliTheme.brume2
            }
            Repeater {
                model: view.protocols
                Rectangle {
                    required property var modelData
                    width: column.width
                    height: 30
                    radius: FiliTheme.radiusControl
                    color: FiliTheme.nuit3
                    opacity: modelData.available ? 1 : 0.5
                    Text {
                        anchors { left: parent.left; leftMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
                        text: modelData.label
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.size
                        color: FiliTheme.brume
                    }
                    Text {
                        anchors { right: toggle.left; rightMargin: FiliTheme.space3; verticalCenter: parent.verticalCenter }
                        width: 260
                        horizontalAlignment: Text.AlignRight
                        elide: Text.ElideRight
                        text: modelData.available ? (modelData.on ? Fili.tr("On") : Fili.tr("Off")) : modelData.why
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.sizeList
                        color: modelData.on ? FiliTheme.voile : FiliTheme.brume2
                    }
                    TextButton {
                        id: toggle
                        anchors { right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                        visible: modelData.available
                        primary: !modelData.on
                        label: modelData.on ? Fili.tr("Turn Off") : Fili.tr("Turn On")
                        onClicked: Fili.receiveSet(modelData.id, !modelData.on)
                    }
                }
            }
            Text {
                width: parent.width
                elide: Text.ElideMiddle
                text: Fili.tr("Saved in %1").replace("%1", view.info.dest)
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.sizeList
                color: FiliTheme.brume2
            }

            // Reçus récemment
            Text {
                visible: view.received.length > 0
                text: Fili.tr("Received recently")
                font.family: FiliTheme.font
                font.pixelSize: FiliTheme.size
                font.weight: FiliTheme.weightAccent
                color: FiliTheme.brume
            }
            Repeater {
                model: view.received.slice(0, 8)
                Rectangle {
                    required property var modelData
                    width: column.width
                    height: 26
                    radius: FiliTheme.radiusControl
                    color: itemArea.containsMouse ? FiliTheme.nuit3 : "transparent"
                    Text {
                        anchors { left: parent.left; leftMargin: FiliTheme.space2; right: from.left; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                        elide: Text.ElideMiddle
                        text: modelData.path.split("/").pop()
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.size
                        color: FiliTheme.brume
                    }
                    Text {
                        id: from
                        anchors { right: parent.right; rightMargin: FiliTheme.space2; verticalCenter: parent.verticalCenter }
                        text: modelData.sender + " · " + view.when(modelData.time)
                        font.family: FiliTheme.font
                        font.pixelSize: FiliTheme.sizeList
                        color: FiliTheme.brume2
                    }
                    MouseArea {
                        id: itemArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: view.openPath(modelData.path)
                    }
                }
            }
        }
    }
}
