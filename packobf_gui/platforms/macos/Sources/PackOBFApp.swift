import SwiftUI

@main
struct PackOBFApp: App {
    var body: some Scene {
        Window("PackOBF", id: "main") {
            PackView()
                .frame(minWidth: 540, minHeight: 380)
        }
        .defaultSize(width: 720, height: 800)
        .windowResizability(.contentMinSize)
    }
}
