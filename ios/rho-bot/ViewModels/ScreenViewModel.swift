import SwiftUI

enum FrameRate: String, CaseIterable {
    case slow = "Slow (2s)"
    case normal = "Normal (1s)"
    case fast = "Fast (0.5s)"

    var intervalMs: Int {
        switch self {
        case .slow: return 2000
        case .normal: return 1000
        case .fast: return 500
        }
    }
}

@MainActor
class ScreenViewModel: ObservableObject {
    private let client = ScreenStreamClient()

    @Published var isConnected = false
    @Published var isDesktopOnline = false
    @Published var currentFrame: UIImage?
    @Published var frameRate: FrameRate = .normal

    private var cancellables: [Any] = []

    init() {
        // Forward published properties from client
        let c1 = client.$isConnected.assign(to: &$isConnected)
        let c2 = client.$isDesktopOnline.assign(to: &$isDesktopOnline)
        let c3 = client.$currentFrame.assign(to: &$currentFrame)
        // Keep references alive (assign(to:) on @Published manages lifecycle)
    }

    func connect() {
        client.connect()
    }

    func disconnect() {
        client.disconnect()
    }

    func setFrameRate(_ rate: FrameRate) {
        frameRate = rate
        client.setInterval(rate.intervalMs)
    }
}
