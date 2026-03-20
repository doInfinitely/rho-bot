import Foundation
import SwiftUI
import AVFoundation

struct ChatMessage: Identifiable {
    let id = UUID()
    let type: MessageType
    let content: String
    let timestamp = Date()

    enum MessageType {
        case user
        case thinking
        case action
        case result
        case error
        case askUser
        case complete
    }
}

@MainActor
class ChatViewModel: ObservableObject {
    @Published var messages: [ChatMessage] = []
    @Published var inputText = ""
    @Published var isRunning = false
    @Published var isConnected = false
    @Published var awaitingUserResponse = false
    @Published var isRecordingAudio = false
    @Published var waveform: [CGFloat] = Array(repeating: 0, count: 128)

    private let ws = WebSocketClient.shared
    private let tts = ElevenLabsService.shared

    init() {
        // Forward audio levels from ElevenLabsService
        tts.$waveform.assign(to: &$waveform)
        ws.onStep = { [weak self] step in
            guard let self else { return }
            if let thinking = step.thinking, !thinking.isEmpty {
                self.messages.append(ChatMessage(type: .thinking, content: thinking))
            }
            self.messages.append(ChatMessage(type: .action, content: step.action))
            if let result = step.result, !result.isEmpty {
                let type: ChatMessage.MessageType = step.status == "failed" ? .error : .result
                self.messages.append(ChatMessage(type: type, content: result))
            }
        }

        ws.onComplete = { [weak self] complete in
            guard let self else { return }
            let duration = String(format: "%.1fs", complete.duration)
            self.messages.append(ChatMessage(
                type: .complete,
                content: "\(complete.result)\n\(complete.steps) steps in \(duration)"
            ))
            self.isRunning = false
            // Speak the result via TTS if enabled
            Task { await self.tts.speak(complete.result) }
        }

        ws.onError = { [weak self] msg in
            guard let self else { return }
            self.messages.append(ChatMessage(type: .error, content: msg))
            self.isRunning = false
        }

        ws.onStopped = { [weak self] in
            guard let self else { return }
            self.messages.append(ChatMessage(type: .result, content: "Task stopped"))
            self.isRunning = false
        }

        ws.onAskUser = { [weak self] prompt in
            guard let self else { return }
            self.messages.append(ChatMessage(type: .askUser, content: prompt))
            self.awaitingUserResponse = true
            // Speak the question
            Task { await self.tts.speak(prompt) }
        }
    }

    func connect() {
        ws.ensureConnected()
        isConnected = ws.isConnected
    }

    func disconnect() {
        ws.disconnect()
        isConnected = false
    }

    func send() {
        let text = inputText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }

        messages.append(ChatMessage(type: .user, content: text))
        inputText = ""

        if awaitingUserResponse {
            ws.sendChat(text)
            awaitingUserResponse = false
        } else if !isRunning {
            ws.runTask(text)
            isRunning = true
            isConnected = true
        } else {
            ws.sendChat(text)
        }
    }

    func stop() {
        ws.stopTask()
        tts.stopPlayback()
    }

    func clearMessages() {
        messages.removeAll()
    }

    // MARK: - Recording

    func toggleRecording() {
        if isRecordingAudio {
            stopRecording()
        } else {
            startRecording()
        }
    }

    private func startRecording() {
        AVAudioApplication.requestRecordPermission { [weak self] granted in
            Task { @MainActor in
                guard let self, granted else { return }
                self.tts.startRecording()
                self.isRecordingAudio = true
            }
        }
    }

    private func stopRecording() {
        isRecordingAudio = false
        Task {
            if let text = await tts.stopRecordingAndTranscribe() {
                inputText = text
            }
        }
    }
}
