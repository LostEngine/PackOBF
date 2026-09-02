import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import dev.lost.packobf 1.0

ApplicationWindow {
    id: root
    visible: true
    width: 1280
    height: 720
    title: qsTr("packobf")

    SystemPalette {
        id: sysPalette
        colorGroup: SystemPalette.Active
    }

    color: sysPalette.window

    AppController {
        id: appController
    }

    ListModel {
        id: logsModel
    }

    Connections {
        target: appController

        function onLogsBatch(batchStr) {
            let items = batchStr.split('\x1E');
            for (let i = 0; i < items.length - 1; i++) {
                let item = items[i];
                let level = parseInt(item.charAt(0));
                let msg = item.substring(1);
                logsModel.append({"level": level, "message": msg});
            }
        }

        function onLogsCleared() {
            logsModel.clear();
        }
    }

    function urlToPath(url) {
        let urlStr = url.toString();
        if (Qt.platform.os === "windows") {
            return urlStr.replace(/^(file:\/{3})/, "");
        } else {
            return urlStr.replace(/^(file:\/{2})/, "");
        }
    }

    FileDialog {
        id: fileDialog
        title: "Select Resource Pack"
        nameFilters: ["Zip files (*.zip)"]
        onAccepted: appController.selected_file = urlToPath(selectedFile)
    }

    FileDialog {
        id: cacheImportDialog
        title: "Import Cache File"
        nameFilters: ["Bin files (*.bin)"]
        onAccepted: appController.cache_file_path = urlToPath(selectedFile)
    }

    FileDialog {
        id: cacheCreateDialog
        title: "Create Cache File"
        fileMode: FileDialog.SaveFile
        nameFilters: ["Bin files (*.bin)"]
        onAccepted: appController.cache_file_path = urlToPath(selectedFile)
    }

    FileDialog {
        id: saveOutputDialog
        title: "Save Optimized Resource Pack"
        fileMode: FileDialog.SaveFile
        defaultSuffix: "zip"
        nameFilters: ["Zip files (*.zip)"]
        onAccepted: appController.save_output(urlToPath(selectedFile))
    }

    Dialog {
        id: statsDialog
        title: "Optimization Complete"
        anchors.centerIn: parent
        modal: true
        standardButtons: Dialog.Ok
        visible: appController.show_stats_popup
        onClosed: appController.show_stats_popup = false

        ColumnLayout {
            Label {
                text: "Time taken: " + appController.stats_time
            }
            Label {
                text: "Input size: " + appController.stats_input
            }
            Label {
                text: "Output size: " + appController.stats_output
            }
            Label {
                text: "Saved: " + appController.stats_saved
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        Label {
            text: qsTr("An open-source Minecraft: Java Edition resource pack minimizer, obfuscator and checker.")
            Layout.fillWidth: true
            color: sysPalette.windowText
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        RowLayout {
            Label {
                text: "Resource pack file"; Layout.preferredWidth: 150
            }
            Button {
                text: "Select file"
                onClicked: fileDialog.open()
            }
            TextField {
                text: appController.selected_file
                onTextChanged: appController.selected_file = text
                Layout.fillWidth: true
            }
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        RowLayout {
            Label {
                text: "Cache file (Optional)"; Layout.preferredWidth: 150
            }
            Button {
                text: "Import cache file"
                onClicked: cacheImportDialog.open()
            }
            Button {
                text: "Create cache file"
                onClicked: cacheCreateDialog.open()
            }
            TextField {
                text: appController.cache_file_path
                onTextChanged: appController.cache_file_path = text
                Layout.fillWidth: true
            }
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        RowLayout {
            Label {
                text: "Compression"
                Layout.preferredWidth: 150
                ToolTip.visible: ma1.containsMouse
                ToolTip.text: "Global compression level for the resource pack."
                MouseArea {
                    id: ma1; anchors.fill: parent; hoverEnabled: true
                }
            }
            ComboBox {
                model: ["Fastest", "Fast", "Normal", "Best", "Ultra"]
                currentIndex: appController.compression
                onCurrentIndexChanged: appController.compression = currentIndex
                Layout.preferredWidth: 200
            }
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        RowLayout {
            Label {
                text: "Shader compression (Experimental)"
                Layout.preferredWidth: 250
                ToolTip.visible: ma2.containsMouse
                ToolTip.text: "Parses GLSL core shaders to minify and obfuscate them. (Experimental)"
                MouseArea {
                    id: ma2; anchors.fill: parent; hoverEnabled: true
                }
            }
            ComboBox {
                model: ["None", "Minify", "Minify and obfuscate"]
                currentIndex: appController.shader_compression
                onCurrentIndexChanged: appController.shader_compression = currentIndex
                Layout.preferredWidth: 200
            }
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        RowLayout {
            Label {
                text: "Target Minecraft version"
                Layout.preferredWidth: 250
                ToolTip.visible: ma7.containsMouse
                ToolTip.text: "Removes parts of the resource pack specific to other Minecraft versions."
                MouseArea {
                    id: ma7; anchors.fill: parent; hoverEnabled: true
                }
            }
            ComboBox {
                id: targetVersionComboBox
                model: [
                    {text: "Any", value: 0},
                    {text: "1.21.1", value: 34},
                    {text: "1.21.2", value: 42},
                    {text: "1.21.4", value: 46},
                    {text: "1.21.5", value: 55},
                    {text: "1.21.6", value: 63},
                    {text: "1.21.7", value: 64},
                    {text: "1.21.9", value: 69},
                    {text: "1.21.11", value: 75},
                    {text: "26.1", value: 84},
                    {text: "26.2", value: 88}
                ]
                textRole: "text"
                currentIndex: indexOfValue(appController.target_version)
                onActivated: function(index) {
                    appController.target_version = model[index].value
                }
                Layout.preferredWidth: 200

                function indexOfValue(value) {
                    for (let index = 0; index < model.length; index++) {
                        if (model[index].value === value)
                            return index;
                    }
                    return 0;
                }
            }
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        RowLayout {
            spacing: 20
            CheckBox {
                text: "Rename files"
                checked: appController.rename_files
                onCheckedChanged: appController.rename_files = checked
                ToolTip.visible: ma3.containsMouse
                ToolTip.text: "Renames textures, models, and sounds to shorter names while keeping the resource pack working."
                MouseArea {
                    id: ma3; anchors.fill: parent; hoverEnabled: true; acceptedButtons: Qt.NoButton
                }
            }
            CheckBox {
                text: "Block resource pack unzipping"
                checked: appController.block_unzipping
                onCheckedChanged: appController.block_unzipping = checked
                ToolTip.visible: ma4.containsMouse
                ToolTip.text: "Adds some bytes to the resource pack to prevent files from being extracted on a file system."
                MouseArea {
                    id: ma4; anchors.fill: parent; hoverEnabled: true; acceptedButtons: Qt.NoButton
                }
            }
            CheckBox {
                text: "Corrupt PNG files"
                checked: appController.corrupt_png_files
                onCheckedChanged: appController.corrupt_png_files = checked
                ToolTip.visible: ma5.containsMouse
                ToolTip.text: "Corrupts PNG files in a way that makes them unreadable for most software except for Minecraft."
                MouseArea {
                    id: ma5; anchors.fill: parent; hoverEnabled: true; acceptedButtons: Qt.NoButton
                }
            }
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        Button {
            id: advancedOptionsButton
            text: checked ? "Hide advanced options" : "Advanced options"
            checkable: true
        }

        MenuSeparator {
            visible: advancedOptionsButton.checked
            Layout.fillWidth: true
        }

        RowLayout {
            visible: advancedOptionsButton.checked
            Label {
                text: "Worker threads"
                Layout.preferredWidth: 250
                ToolTip.visible: ma6.containsMouse
                ToolTip.text: "Number of worker threads to use. Zero for automatic."
                MouseArea {
                    id: ma6; anchors.fill: parent; hoverEnabled: true
                }
            }
            SpinBox {
                to: 256
                editable: true
                value: appController.num_threads
                onValueModified: appController.num_threads = value
                Layout.preferredWidth: 200
            }
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        RowLayout {
            Button {
                text: "Optimize"
                enabled: !appController.processing && appController.selected_file !== ""
                onClicked: appController.optimize()
            }
            Label {
                text: "Processing..."
                visible: appController.processing
                color: sysPalette.windowText
            }
            Button {
                text: "Save file"
                visible: appController.done
                onClicked: saveOutputDialog.open()
            }
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        Label {
            text: "Status: " + appController.progress_text
            Layout.fillWidth: true
            color: sysPalette.windowText
        }

        MenuSeparator {
            Layout.fillWidth: true
        }

        RowLayout {
            Label {
                text: "Logs";
                font.bold: true;
                Layout.fillWidth: true;
                color: sysPalette.windowText
            }
            CheckBox {
                text: "Info"
                checked: appController.show_info
                onCheckedChanged: {
                    appController.show_info = checked
                }
            }
            CheckBox {
                text: "Warning"
                checked: appController.show_warning
                onCheckedChanged: {
                    appController.show_warning = checked
                }
            }
            CheckBox {
                text: "Error"
                checked: appController.show_error
                onCheckedChanged: {
                    appController.show_error = checked
                }
            }
            Button {
                text: "Copy"
                onClicked: appController.copy_logs()
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true

            color: sysPalette.base
            border.color: sysPalette.mid
            border.width: 1
            radius: 4
            clip: true

            ListView {
                id: logListView
                anchors.fill: parent
                anchors.margins: 4
                clip: true
                model: logsModel

                property bool shouldAutoScroll: true

                onAtYEndChanged: {
                    if (atYEnd) {
                        shouldAutoScroll = true;
                    }
                }

                // Unobtrusively listen to MouseWheel scrolling
                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.NoButton
                    onWheel: (wheel) => {
                        wheel.accepted = false; // allow ListView to actually scroll
                        Qt.callLater(function () {
                            logListView.shouldAutoScroll = logListView.atYEnd;
                        });
                    }
                }

                // Updates based on user dragging on a touch screen / trackpad
                onContentYChanged: {
                    if (logListView.dragging) {
                        shouldAutoScroll = logListView.atYEnd;
                    }
                }

                onCountChanged: {
                    if (shouldAutoScroll) {
                        Qt.callLater(function () {
                            logListView.positionViewAtEnd();
                        });
                    }
                }

                delegate: Text {
                    width: ListView.view.width
                    height: visible ? logText.implicitHeight : 0
                    visible: {
                        if (model.level === 0 && !appController.show_info) return false;
                        if (model.level === 1 && !appController.show_warning) return false;
                        return !(model.level === 2 && !appController.show_error);
                    }

                    Text {
                        id: logText
                        width: parent.width
                        text: {
                            let prefix = "";
                            if (model.level === 0) prefix = "INFO: ";
                            else if (model.level === 1) prefix = "WARNING: ";
                            else if (model.level === 2) prefix = "ERROR: ";
                            return prefix + model.message;
                        }
                        color: {
                            if (model.level === 0) return "#0073e6";
                            if (model.level === 1) return "#d98200";
                            if (model.level === 2) return "#d90000";
                            return sysPalette.text;
                        }
                        font.bold: true
                        wrapMode: Text.Wrap
                    }
                }

                ScrollBar.vertical: ScrollBar {
                    onPositionChanged: {
                        if (pressed) {
                            logListView.shouldAutoScroll = (position + size >= 1.0 - 0.001)
                        }
                    }
                }
            }
        }
    }
}
