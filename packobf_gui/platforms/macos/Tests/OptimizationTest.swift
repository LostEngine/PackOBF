import Foundation

@main
struct OptimizationTest {
    struct Failure: Error {
        let description: String
    }

    @MainActor
    static func main() async throws {
        let files = FileManager.default
        let temp = files.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try files.createDirectory(at: temp, withIntermediateDirectories: false)
        defer { try? files.removeItem(at: temp) }

        let input = temp.appendingPathComponent("resource pack 🚀.zip")
        let cache = temp.appendingPathComponent("cache.bin")
        let output = temp.appendingPathComponent("optimized.zip")
        let endOfDirectory: [UInt8] = [0x50, 0x4b, 0x05, 0x06]
        try Data(endOfDirectory + Array(repeating: 0, count: 18)).write(to: input)

        let model = PackModel()
        try expect(model.status == "Idle" && !model.canSave, "The application should start idle")
        try expect(model.settings.compression == 1, "Use the backend’s default compression")
        try expect(model.settings.rename_files == 1, "Use the backend’s default renaming option")
        model.input = input.path
        model.cache = cache.path
        model.settings.worker_threads = 1
        model.settings.compression = 0
        model.settings.target_version = 97

        try await optimize(model)
        try expect(model.canSave, "A successful job should enable saving")
        try expect(model.message?.title == "Optimization Complete", "A successful job should show a summary")
        try expect(model.message?.detail.contains("Input size: 22 B") == true, "Report input size")
        try expect(files.fileExists(atPath: cache.path), "Create the requested cache file")
        try expect(model.logs.contains { $0.level == .warning }, "Expose warning severity")
        try expect(model.copyableLogs().contains("WARNING: Invalid cache file"), "Copy all backend logs")

        model.message = nil
        model.save(to: output.path)
        let saved = try Data(contentsOf: output)
        try expect(saved.suffix(22).starts(with: endOfDirectory), "Save a ZIP to the selected path")

        model.save(to: temp.appendingPathComponent("missing/pack.zip").path)
        try expect(model.message?.title == "Couldn’t save resource pack", "Report save errors")
        try expect(model.canSave, "Keep output available after a failed save")

        model.settings.rename_files = 0
        try expect(!model.canSave && model.status == "Idle", "Invalidate output when options change")
        try await optimize(model)
        try expect(model.canSave && model.copyableLogs().contains("INFO: Cache loaded:"), "Reuse the cache")

        model.input = temp.appendingPathComponent("missing.zip").path
        try expect(!model.canSave && model.message == nil, "Invalidate output and summary when the input changes")
        try await optimize(model)
        try expect(model.status == "Error reading file" && !model.canSave, "Report missing input")
        try expect(model.logs.contains { $0.level == .error && $0.line.contains("missing.zip") }, "Expose error severity")
        try expect(!model.copyableLogs().contains("Cache loaded:"), "Clear logs when a new job starts")

        model.input = input.path
        model.input += "\0"
        model.start()
        try expect(!model.running && model.message?.title == "Couldn’t start optimization", "Report invalid paths")
        print("Swift/Rust optimizer integration passed")
    }

    @MainActor
    private static func optimize(_ model: PackModel) async throws {
        model.start()
        let poll = Task { await model.pollWhileRunning() }
        defer { poll.cancel() }
        let deadline = ContinuousClock.now + .seconds(10)
        while model.running {
            try expect(ContinuousClock.now < deadline, "Optimization timed out")
            try await Task.sleep(for: .milliseconds(10))
        }
    }

    private static func expect(_ condition: Bool, _ description: String) throws {
        guard condition else { throw Failure(description: description) }
    }
}
