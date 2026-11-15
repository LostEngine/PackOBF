import AppKit
import SwiftUI

struct LogView: View {
    let logs: [Optimizer.Log]
    @Binding var levels: Set<Optimizer.Level>
    let copy: () -> Void

    @Environment(\.colorSchemeContrast) private var contrast
    @State private var position = ScrollPosition(edge: .bottom)
    @State private var follow = true

    var body: some View {
        let logBox = RoundedRectangle(cornerRadius: 7)
        VStack(spacing: 6) {
            HStack(spacing: 10) {
                filter("Info", level: .info)
                filter("Warning", level: .warning)
                filter("Error", level: .error)
                Spacer()
                Button("Copy", action: copy).help("Copy all log messages")
            }
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 3) {
                    ForEach(logs.filter { levels.contains($0.level) }) { log in
                        Text(log.line)
                            .font(.system(.caption, design: .monospaced))
                            .foregroundStyle(color(for: log.level))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
                .padding(8)
            }
            .frame(height: 164)
            .background(.windowBackground, in: logBox)
            .overlay {
                logBox.strokeBorder(Color(nsColor: .separatorColor))
                    .allowsHitTesting(false)
            }
            .accessibilityLabel("Optimization logs")
            .defaultScrollAnchor(.bottom, for: .initialOffset)
            .scrollPosition($position)
            .onScrollPhaseChange { old, new, context in
                if new == .idle && old != .animating {
                    let geometry = context.geometry
                    follow = geometry.visibleRect.maxY >= geometry.contentSize.height - 2
                } else if new == .interacting {
                    follow = false
                }
            }
            .onChange(of: logs.count) {
                if logs.isEmpty { follow = true }
                if follow { position.scrollTo(edge: .bottom) }
            }
            .onChange(of: levels) {
                follow = true
                position.scrollTo(edge: .bottom)
            }
        }
    }

    private func filter(_ title: String, level: Optimizer.Level) -> some View {
        Toggle(title, isOn: Binding(
            get: { levels.contains(level) },
            set: { visible in
                if visible { levels.insert(level) } else { levels.remove(level) }
            }
        ))
        .toggleStyle(.checkbox)
    }

    private func color(for level: Optimizer.Level) -> Color {
        if contrast == .increased { return .primary }
        return switch level {
        case .info: Color(nsColor: .systemBlue)
        case .warning: Color(nsColor: .systemOrange)
        case .error: Color(nsColor: .systemRed)
        }
    }
}
