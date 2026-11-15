import Foundation

@MainActor
final class Optimizer {
    struct Failure: LocalizedError {
        let message: String
        var errorDescription: String? { message }
    }

    enum Level: UInt8, CaseIterable {
        case info, warning, error
    }

    struct Log: Identifiable {
        let id: Int
        let level: Level
        let line: String
    }

    struct Snapshot {
        let status: String
        let summary: String?
        let logCount: Int
        let running: Bool
        let canSave: Bool
    }

    private final class Handle {
        let pointer = packobf_gui_create()!

        deinit {
            packobf_gui_destroy(pointer)
        }
    }

    private let handle = Handle()

    static func defaults() -> PackObfGuiSettings {
        packobf_gui_defaults()
    }

    func start(input: String, cache: String, settings: PackObfGuiSettings) throws {
        try Self.validatePath(input)
        try Self.validatePath(cache)
        let error = input.withCString { input in
            if cache.isEmpty {
                return packobf_gui_start(handle.pointer, input, nil, settings)
            }
            return cache.withCString { packobf_gui_start(handle.pointer, input, $0, settings) }
        }
        if let message = Self.takeString(error) {
            throw Failure(message: message)
        }
    }

    func invalidate() {
        packobf_gui_invalidate(handle.pointer)
    }

    func poll() -> Snapshot {
        let snapshot = packobf_gui_poll(handle.pointer)
        return Snapshot(
            status: Self.takeString(snapshot.status) ?? "",
            summary: Self.takeString(snapshot.summary),
            logCount: snapshot.log_count,
            running: snapshot.running != 0,
            canSave: snapshot.can_save != 0
        )
    }

    func log(at index: Int) -> Log {
        let entry = packobf_gui_log(handle.pointer, index)
        return Log(
            id: index,
            level: Level(rawValue: entry.level) ?? .info,
            line: Self.takeString(entry.line) ?? ""
        )
    }

    func copyableLogs() -> String {
        Self.takeString(packobf_gui_copyable_logs(handle.pointer)) ?? ""
    }

    func save(to output: String) throws {
        try Self.validatePath(output)
        let error = output.withCString { packobf_gui_save(handle.pointer, $0) }
        if let message = Self.takeString(error) {
            throw Failure(message: message)
        }
    }

    private static func validatePath(_ path: String) throws {
        if path.utf8.contains(0) {
            throw Failure(message: "A file path contains a NUL character.")
        }
    }

    private static func takeString(_ pointer: UnsafeMutablePointer<CChar>?) -> String? {
        guard let pointer else { return nil }
        defer { packobf_gui_free_string(pointer) }
        return String(cString: pointer)
    }
}
