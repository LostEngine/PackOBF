import AppKit
import SwiftUI
import UniformTypeIdentifiers

struct PackView: View {
    @State private var model = PackModel()
    @State private var advancedExpanded = false
    @State private var showLogs = false
    @State private var logLevels = Set(Optimizer.Level.allCases)

    var body: some View {
        @Bindable var model = model
        Form {
            Section {
                pathRow("Resource pack file", path: $model.input) {
                    fileButton(.input, icon: "folder")
                }
                pathRow("Cache file (recommended)", path: $model.cache) {
                    fileButton(.importCache, icon: "folder")
                    fileButton(.createCache, icon: "doc.badge.plus")
                }
            } header: {
                VStack(alignment: .leading, spacing: 3) {
                    Text("Resource pack")
                    Text("Select a Minecraft: Java Edition resource pack ZIP.")
                        .font(.caption)
                        .fontWeight(.regular)
                        .foregroundStyle(.secondary)
                }
            }
            settings
            advancedOptions
        }
        .formStyle(.grouped)
        .scrollContentBackground(.hidden)
        .background(.windowBackground)
        .safeAreaInset(edge: .bottom, spacing: 0) {
            actions
        }
        .task(id: model.running) {
            if model.running { await model.pollWhileRunning() }
        }
        .alert(
            Text(model.message?.title ?? ""),
            isPresented: Binding(
                get: { model.message != nil && !model.pickerOpen },
                set: { if !$0 { model.message = nil } }
            ),
            presenting: model.message
        ) { _ in
            Button("OK", role: .cancel) {}
        } message: { message in
            Text(message.detail)
        }
    }

    private var settings: some View {
        Section("Optimization settings") {
            choice("Compression", detail: "Global compression level for the resource pack.",
                   key: \.compression, options: Self.indexed(["Fast", "Normal"]))
            choice("Shader compression (Experimental)", detail: "Parses GLSL core shaders to minify and obfuscate them.",
                   key: \.shader_compression, options: Self.indexed(["None", "Minify", "Minify and obfuscate"]))
            choice("Sound quality", detail: "Re-encodes sound files using libvorbis with the requested quality.",
                   key: \.sound_quality, options: Self.indexed(["Dynamic"] + (-1...10).map { String($0) }))
            choice("Target Minecraft version", detail: "Removes parts of the resource pack specific to other Minecraft versions.",
                   key: \.target_version, options: Self.versions)
            toggle("Rename files", detail: "Renames textures, models, and sounds to shorter names.",
                   key: \.rename_files)
            toggle("Block resource pack unzipping",
                   detail: "Adds bytes to prevent extracting the entire resource pack to a file system.",
                   key: \.block_unzipping)
            toggle("Corrupt PNG files",
                   detail: "Makes PNG files unreadable for most software except Minecraft.",
                   key: \.corrupt_png_files)
        }
    }

    private var actions: some View {
        VStack(spacing: 0) {
            Divider()
            HStack(spacing: 8) {
                Text(model.status)
                    .lineLimit(1)
                    .help(model.status)
                Spacer()
                if model.running {
                    ProgressView()
                        .controlSize(.small)
                        .accessibilityLabel("Processing")
                } else if model.canSave {
                    Button("Optimize", action: model.start)
                    Button("Save file") { choose(.output) }
                        .buttonStyle(.borderedProminent)
                } else {
                    Button("Optimize", action: model.start)
                        .buttonStyle(.borderedProminent)
                        .disabled(model.input.isEmpty)
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(.windowBackground)
        }
    }

    private func pathRow<Buttons: View>(
        _ title: String, path: Binding<String>, @ViewBuilder buttons: () -> Buttons
    ) -> some View {
        HStack(spacing: 8) {
            Text(title).frame(width: 158, alignment: .leading)
            TextField(title, text: path, prompt: Text("Enter a file path"))
                .labelsHidden()
                .accessibilityLabel(title)
                .textFieldStyle(.roundedBorder)
            buttons()
        }
    }

    private func fileButton(_ action: FileAction, icon: String) -> some View {
        Button { choose(action) } label: {
            Label(action.title, systemImage: icon).labelStyle(.iconOnly)
        }
        .help(action.title)
    }

    private func settingRow<Control: View>(
        _ title: String, detail: String, @ViewBuilder control: () -> Control
    ) -> some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                Text(detail).font(.caption).foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .fixedSize(horizontal: false, vertical: true)
            control()
        }
    }

    private func choice(
        _ title: String, detail: String,
        key: WritableKeyPath<PackObfGuiSettings, UInt8>, options: [Choice]
    ) -> some View {
        settingRow(title, detail: detail) {
            ChoiceMenu(
                title: title,
                options: options,
                selection: Binding(
                    get: { model.settings[keyPath: key] },
                    set: { model.settings[keyPath: key] = $0 }
                )
            )
            .frame(width: 172)
        }
    }

    private func toggle(
        _ title: String, detail: String, key: WritableKeyPath<PackObfGuiSettings, UInt8>
    ) -> some View {
        settingRow(title, detail: detail) {
            Toggle(title, isOn: Binding(
                get: { model.settings[keyPath: key] != 0 },
                set: { model.settings[keyPath: key] = $0 ? 1 : 0 }
            ))
            .labelsHidden()
            .toggleStyle(.switch)
        }
    }

    private var advancedOptions: some View {
        Section {
            Button {
                advancedExpanded.toggle()
            } label: {
                HStack(spacing: 6) {
                    Image(systemName: advancedExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption.weight(.semibold))
                        .frame(width: 12)
                        .accessibilityHidden(true)
                    Text("Advanced options")
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.vertical, 10)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityValue(advancedExpanded ? "Expanded" : "Collapsed")

            if advancedExpanded {
                settingRow("Worker threads", detail: "Zero for automatic") {
                    workerThreads
                }
            }
        } footer: {
            logs
        }
    }

    private var logs: some View {
        VStack(alignment: .leading, spacing: 8) {
            Button(showLogs ? "Hide logs" : "Show logs") { showLogs.toggle() }
            if showLogs {
                LogView(logs: model.logs, levels: $logLevels) {
                    let pasteboard = NSPasteboard.general
                    pasteboard.clearContents()
                    if !pasteboard.setString(model.copyableLogs(), forType: .string) {
                        model.message = .init(
                            title: "Couldn’t copy logs",
                            detail: "The clipboard could not be updated."
                        )
                    }
                }
            }
        }
        .font(.body)
        .foregroundStyle(.primary)
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.top, 4)
    }

    private var workerThreads: some View {
        HStack(spacing: 4) {
            TextField("Worker threads", text: Binding(
                get: { String(model.settings.worker_threads) },
                set: { text in
                    if text.isEmpty {
                        model.settings.worker_threads = 0
                    } else if let count = Int(text) {
                        model.settings.worker_threads = UInt32(min(max(count, 0), 256))
                    }
                }
            ))
            .labelsHidden()
            .accessibilityLabel("Worker threads")
            .multilineTextAlignment(.trailing)
            .frame(width: 50)
            Stepper("Adjust worker threads", value: Binding(
                get: { model.settings.worker_threads },
                set: { model.settings.worker_threads = $0 }
            ), in: 0...256)
            .labelsHidden()
        }
    }

    private func choose(_ action: FileAction) {
        guard !model.pickerOpen, let window = NSApp.keyWindow else { return }
        let panel = action.makePanel()
        model.pickerOpen = true
        panel.beginSheetModal(for: window) { response in
            defer { model.pickerOpen = false }
            guard response == .OK, let url = panel.url else { return }
            switch action {
            case .input:
                model.input = url.path
            case .importCache, .createCache:
                model.cache = url.path
            case .output:
                model.save(to: url.path)
            }
        }
    }

    private struct Choice {
        let id: UInt8
        let title: String

        init(_ title: String, _ id: UInt8) {
            self.id = id
            self.title = title
        }
    }

    private struct ChoiceMenu: NSViewRepresentable {
        let title: String
        let options: [Choice]
        @Binding var selection: UInt8

        func makeNSView(context: Context) -> NSPopUpButton {
            let button = NSPopUpButton(frame: .zero, pullsDown: false)
            button.isBordered = false
            button.setAccessibilityLabel(title)
            for option in options {
                button.addItem(withTitle: option.title)
                button.lastItem?.tag = Int(option.id)
            }
            button.target = context.coordinator
            button.action = #selector(Coordinator.selectItem)
            return button
        }

        func updateNSView(_ button: NSPopUpButton, context: Context) {
            context.coordinator.selection = $selection
            button.selectItem(withTag: Int(selection))
        }

        func sizeThatFits(
            _ proposal: ProposedViewSize, nsView button: NSPopUpButton, context: Context
        ) -> CGSize? {
            let nativeSize = button.intrinsicContentSize
            return CGSize(width: proposal.width ?? nativeSize.width, height: nativeSize.height)
        }

        func makeCoordinator() -> Coordinator {
            Coordinator(selection: $selection)
        }

        @MainActor
        final class Coordinator: NSObject {
            var selection: Binding<UInt8>

            init(selection: Binding<UInt8>) {
                self.selection = selection
            }

            @objc func selectItem(_ button: NSPopUpButton) {
                if let item = button.selectedItem, let value = UInt8(exactly: item.tag) {
                    selection.wrappedValue = value
                }
            }
        }
    }

    private static func indexed(_ labels: [String]) -> [Choice] {
        labels.enumerated().map { Choice($0.element, UInt8($0.offset)) }
    }

    private static let versions: [Choice] = [
        Choice("Any", 0), Choice("1.21.1", 34), Choice("1.21.2", 42), Choice("1.21.4", 46),
        Choice("1.21.5", 55), Choice("1.21.6", 63), Choice("1.21.7", 64), Choice("1.21.9", 69),
        Choice("1.21.11", 75), Choice("26.1", 84), Choice("26.2", 88), Choice("26.3", 97),
    ]
}

private enum FileAction {
    case input, importCache, createCache, output

    var title: String {
        switch self {
        case .input: "Select resource pack"
        case .importCache: "Import cache file"
        case .createCache: "Create cache file"
        case .output: "Save resource pack"
        }
    }

    @MainActor
    func makePanel() -> NSSavePanel {
        let panel: NSSavePanel
        switch self {
        case .input, .importCache:
            let open = NSOpenPanel()
            open.canChooseDirectories = false
            open.allowsMultipleSelection = false
            panel = open
        case .createCache, .output:
            let save = NSSavePanel()
            save.canCreateDirectories = true
            save.nameFieldStringValue = self == .output ? "pack.zip" : "cache.bin"
            panel = save
        }
        panel.title = title
        panel.allowedContentTypes = switch self {
        case .input, .output: [.zip]
        case .importCache, .createCache: [UTType(filenameExtension: "bin") ?? .data]
        }
        return panel
    }
}
