pragma Singleton
// Whether a drop is a copy. Between two views of Casier the proposed action is not reliable on Wayland (the source
// proposes "move" but the negotiated action can come back as "copy"), so a drag that starts here copies only while Ctrl
// is held and moves otherwise. A drag from another application keeps the action it proposes.
import QtQuick

QtObject {
    function isCopy(drop) {
        if (drop.source)
            return (Qt.application.keyboardModifiers & Qt.ControlModifier) !== 0;
        return drop.proposedAction === Qt.CopyAction;
    }
}
