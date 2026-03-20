import Foundation

/// Connects to the marionette WebSocket server and handles the message protocol.
@MainActor
class WebSocketClient: ObservableObject {
    static let shared = WebSocketClient()

    @Published var isConnected = false
    @Published var isRunning = false

    var onStep: ((StepMessage) -> Void)?
    var onComplete: ((CompleteMessage) -> Void)?
    var onError: ((String) -> Void)?
    var onStopped: (() -> Void)?
    var onAskUser: ((String) -> Void)?

    let wsURL = "wss://marionette-production.up.railway.app/ws"

    private var task: URLSessionWebSocketTask?
    private var session = URLSession(configuration: .default)
    private var reconnectWork: DispatchWorkItem?
    private var reconnectAttempts = 0
    private let maxReconnectAttempts = 5

    private init() {}

    // MARK: - Connection

    /// Connect to the WebSocket server. Called lazily when user sends a message.
    func connect() {
        disconnect()
        guard let url = URL(string: wsURL) else { return }
        reconnectAttempts = 0
        task = session.webSocketTask(with: url)
        task?.resume()
        isConnected = true
        receiveLoop()
    }

    /// Ensure connected — connect if not already.
    func ensureConnected() {
        if task == nil || !isConnected {
            connect()
        }
    }

    func disconnect() {
        reconnectWork?.cancel()
        reconnectWork = nil
        reconnectAttempts = 0
        task?.cancel(with: .goingAway, reason: nil)
        task = nil
        isConnected = false
        isRunning = false
    }

    // MARK: - Send

    func runTask(_ taskText: String, settings: [String: Any] = [:]) {
        ensureConnected()
        let payload: [String: Any] = [
            "type": "run",
            "task": taskText,
            "settings": settings
        ]
        send(payload)
        isRunning = true
    }

    func stopTask() {
        send(["type": "stop"])
    }

    func sendChat(_ message: String) {
        send(["type": "chat", "message": message])
    }

    // MARK: - Private

    private func send(_ dict: [String: Any]) {
        guard let data = try? JSONSerialization.data(withJSONObject: dict),
              let text = String(data: data, encoding: .utf8) else { return }
        task?.send(.string(text)) { error in
            if let error {
                print("WS send error: \(error)")
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
                    print("WS error: \(error.localizedDescription)")
                    self.isConnected = false
                    self.isRunning = false
                    self.scheduleReconnect()
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
        case "step":
            let msg = StepMessage(
                step: json["step"] as? Int ?? 0,
                total: json["total"] as? Int ?? 0,
                action: json["action"] as? String ?? "",
                status: json["status"] as? String ?? "running",
                result: json["result"] as? String,
                thinking: json["thinking"] as? String
            )
            onStep?(msg)

        case "complete":
            isRunning = false
            let msg = CompleteMessage(
                result: json["result"] as? String ?? "",
                steps: json["steps"] as? Int ?? 0,
                duration: json["duration"] as? Double ?? 0
            )
            onComplete?(msg)

        case "error":
            isRunning = false
            onError?(json["message"] as? String ?? "Unknown error")

        case "stopped":
            isRunning = false
            onStopped?()

        case "ask_user":
            onAskUser?(json["message"] as? String ?? "")

        default:
            break
        }
    }

    private func scheduleReconnect() {
        reconnectWork?.cancel()
        reconnectAttempts += 1

        guard reconnectAttempts <= maxReconnectAttempts else {
            print("WS: gave up reconnecting after \(maxReconnectAttempts) attempts")
            return
        }

        // Exponential backoff: 3s, 6s, 12s, 24s, 48s
        let delay = 3.0 * pow(2.0, Double(reconnectAttempts - 1))
        print("WS: reconnect attempt \(reconnectAttempts) in \(delay)s")

        let work = DispatchWorkItem { [weak self] in
            Task { @MainActor in
                self?.connect()
            }
        }
        reconnectWork = work
        DispatchQueue.main.asyncAfter(deadline: .now() + delay, execute: work)
    }
}

// MARK: - Message types

struct StepMessage {
    let step: Int
    let total: Int
    let action: String
    let status: String
    let result: String?
    let thinking: String?
}

struct CompleteMessage {
    let result: String
    let steps: Int
    let duration: Double
}
