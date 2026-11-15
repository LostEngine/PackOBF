import Observation

@MainActor
@Observable
final class PackModel {
    struct Message {
        let title: String
        let detail: String
    }

    var input = "" {
        didSet { if oldValue != input { invalidate() } }
    }
    var cache = "" {
        didSet { if oldValue != cache { invalidate() } }
    }
    var settings = Optimizer.defaults() {
        didSet { invalidate() }
    }
    var message: Message?
    var pickerOpen = false
    private(set) var running = false
    private(set) var canSave = false
    private(set) var status = "Idle"
    private(set) var logs: [Optimizer.Log] = []

    private let optimizer = Optimizer()

    func start() {
        guard !running, !input.isEmpty else { return }
        do {
            try optimizer.start(input: input, cache: cache, settings: settings)
            logs.removeAll()
            refresh()
        } catch {
            message = Message(title: "Couldn’t start optimization", detail: error.localizedDescription)
        }
    }

    func pollWhileRunning() async {
        while running && !Task.isCancelled {
            do {
                try await Task.sleep(for: .milliseconds(50))
            } catch {
                return
            }
            refresh()
        }
    }

    func copyableLogs() -> String {
        optimizer.copyableLogs()
    }

    func save(to path: String) {
        do {
            try optimizer.save(to: path)
        } catch {
            message = Message(title: "Couldn’t save resource pack", detail: error.localizedDescription)
        }
    }

    private func invalidate() {
        message = nil
        optimizer.invalidate()
        refresh()
    }

    private func refresh() {
        let snapshot = optimizer.poll()
        if status != snapshot.status { status = snapshot.status }
        if running != snapshot.running { running = snapshot.running }
        if canSave != snapshot.canSave { canSave = snapshot.canSave }
        if snapshot.logCount > logs.count {
            logs.append(contentsOf: (logs.count..<snapshot.logCount).map { optimizer.log(at: $0) })
        }
        if let summary = snapshot.summary {
            message = Message(title: "Optimization Complete", detail: summary)
        }
    }
}
