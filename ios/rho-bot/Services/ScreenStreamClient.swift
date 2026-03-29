import Foundation
import UIKit

/// Connects to /ws/screen as a subscriber and receives live desktop frames.
@MainActor
class ScreenStreamClient: ObservableObject {
    @Published var isConnected = false
    @Published var isDesktopOnline = false
    @Published var currentFrame: UIImage?

    private var task: URLSessionWebSocketTask?
    private var session = URLSession(configuration: .default)
    private var reconnectWork: DispatchWorkItem?
    private var reconnectAttempts = 0
    private let maxReconnectAttempts = 5

    // MARK: - Connection

    func connect() {
        disconnect()
        reconnectAttempts = 0

        let base = APIClient.shared.baseURL
        let wsBase = base
            .replacingOccurrences(of: "https://", with: "wss://")
            .replacingOccurrences(of: "http://", with: "ws://")
        let urlString = "\(wsBase)/ws/screen"

        guard let url = URL(string: urlString) else {
            print("ScreenStream: invalid URL \(urlString)")
            return
        }

        task = session.webSocketTask(with: url)
        task?.resume()
        isConnected = true

        // Authenticate then register
        sendAuth()
    }

    func disconnect() {
        reconnectWork?.cancel()
        reconnectWork = nil
        reconnectAttempts = 0
        task?.cancel(with: .goingAway, reason: nil)
        task = nil
        isConnected = false
        isDesktopOnline = false
        currentFrame = nil
    }

    func setInterval(_ ms: Int) {
        let payload: [String: Any] = [
            "type": "set_interval",
            "interval_ms": ms
        ]
        send(payload)
    }

    // MARK: - Private

    private func sendAuth() {
        guard let token = APIClient.shared.token else {
            print("ScreenStream: no auth token")
            isConnected = false
            return
        }
        send(["token": token]) {
            self.sendRegister()
        }
    }

    private func sendRegister() {
        send(["type": "register", "role": "subscriber"]) {
            self.receiveLoop()
        }
    }

    private func send(_ dict: [String: Any], completion: (() -> Void)? = nil) {
        guard let data = try? JSONSerialization.data(withJSONObject: dict),
              let text = String(data: data, encoding: .utf8) else { return }
        task?.send(.string(text)) { [weak self] error in
            Task { @MainActor in
                if let error {
                    print("ScreenStream send error: \(error)")
                    self?.handleDisconnect()
                    return
                }
                completion?()
            }
        }
    }

    private func receiveLoop() {
        task?.receive { [weak self] result in
            Task { @MainActor in
                guard let self else { return }
                switch result {
                case .success(.string(let text)):
                    self.reconnectAttempts = 0
                    self.handleMessage(text)
                    self.receiveLoop()
                case .success(.data(let data)):
                    if let text = String(data: data, encoding: .utf8) {
                        self.handleMessage(text)
                    }
                    self.receiveLoop()
                case .failure(let error):
                    print("ScreenStream error: \(error.localizedDescription)")
                    self.handleDisconnect()
                @unknown default:
                    self.receiveLoop()
                }
            }
        }
    }

    private func handleMessage(_ text: String) {
        guard let data = text.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let type = json["type"] as? String else { return }

        switch type {
        case "frame":
            guard let b64 = json["data"] as? String,
                  let imageData = Data(base64Encoded: b64),
                  let image = UIImage(data: imageData) else { return }
            currentFrame = image
            isDesktopOnline = true

        case "status":
            if let online = json["desktop_online"] as? Bool {
                isDesktopOnline = online
                if !online {
                    currentFrame = nil
                }
            }

        default:
            break
        }
    }

    private func handleDisconnect() {
        isConnected = false
        isDesktopOnline = false
        scheduleReconnect()
    }

    private func scheduleReconnect() {
        reconnectWork?.cancel()
        reconnectAttempts += 1

        guard reconnectAttempts <= maxReconnectAttempts else {
            print("ScreenStream: gave up reconnecting after \(maxReconnectAttempts) attempts")
            return
        }

        // Exponential backoff: 3s, 6s, 12s, 24s, 48s
        let delay = 3.0 * pow(2.0, Double(reconnectAttempts - 1))
        print("ScreenStream: reconnect attempt \(reconnectAttempts) in \(delay)s")

        let work = DispatchWorkItem { [weak self] in
            Task { @MainActor in
                self?.connect()
            }
        }
        reconnectWork = work
        DispatchQueue.main.asyncAfter(deadline: .now() + delay, execute: work)
    }
}
