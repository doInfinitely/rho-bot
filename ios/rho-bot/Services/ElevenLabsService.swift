import AVFoundation
import Foundation

/// Proxies TTS/STT through the marionette server so the API key stays server-side.
@MainActor
class ElevenLabsService: ObservableObject {
    static let shared = ElevenLabsService()

    @Published var voices: [Voice] = []
    @Published var selectedVoiceId: String {
        didSet { UserDefaults.standard.set(selectedVoiceId, forKey: "tts_voice_id") }
    }
    @Published var ttsEnabled: Bool {
        didSet { UserDefaults.standard.set(ttsEnabled, forKey: "tts_enabled") }
    }
    @Published var isPlaying = false
    @Published var isRecording = false
    @Published var isTranscribing = false
    @Published var waveform: [CGFloat] = Array(repeating: 0, count: 128)

    private let baseURL = "https://marionette-production.up.railway.app"
    private var audioPlayer: AVAudioPlayer?
    private var audioRecorder: AVAudioRecorder?
    private var recordingURL: URL?
    private let audioEngine = AVAudioEngine()

    struct Voice: Identifiable, Codable {
        let voice_id: String
        let name: String
        let category: String
        var id: String { voice_id }
    }

    private init() {
        self.selectedVoiceId = UserDefaults.standard.string(forKey: "tts_voice_id") ?? "JBFqnCBsd6RMkjVDRZzb"
        self.ttsEnabled = UserDefaults.standard.bool(forKey: "tts_enabled")
        configureAudioSession()
    }

    private func configureAudioSession() {
        do {
            let session = AVAudioSession.sharedInstance()
            try session.setCategory(.playAndRecord, mode: .default, options: [.defaultToSpeaker, .allowBluetooth])
            try session.setActive(true)
        } catch {
            print("Audio session setup failed: \(error)")
        }
    }

    // MARK: - Voices

    func fetchVoices() async {
        guard let url = URL(string: "\(baseURL)/voices") else { return }
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            struct Resp: Decodable { let voices: [Voice] }
            let resp = try JSONDecoder().decode(Resp.self, from: data)
            voices = resp.voices
        } catch {
            print("Failed to fetch voices: \(error)")
        }
    }

    // MARK: - TTS

    func speak(_ text: String) async {
        guard ttsEnabled, !text.isEmpty else { return }

        guard let url = URL(string: "\(baseURL)/tts") else { return }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let body: [String: String] = ["text": text, "voice_id": selectedVoiceId]
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)

        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            guard let http = response as? HTTPURLResponse, http.statusCode == 200 else {
                print("TTS error: \(String(data: data, encoding: .utf8) ?? "?")")
                return
            }
            audioPlayer = try AVAudioPlayer(data: data)
            audioPlayer?.prepareToPlay()
            isPlaying = true
            audioPlayer?.play()

            // Wait for playback to finish
            while audioPlayer?.isPlaying == true {
                try await Task.sleep(nanoseconds: 100_000_000)
            }
            isPlaying = false
        } catch {
            print("TTS playback error: \(error)")
            isPlaying = false
        }
    }

    func stopPlayback() {
        audioPlayer?.stop()
        isPlaying = false
    }

    // MARK: - STT (Recording)

    func startRecording() {
        let tempDir = FileManager.default.temporaryDirectory
        let fileURL = tempDir.appendingPathComponent("stt_recording.m4a")
        recordingURL = fileURL

        let settings: [String: Any] = [
            AVFormatIDKey: Int(kAudioFormatMPEG4AAC),
            AVSampleRateKey: 16000,
            AVNumberOfChannelsKey: 1,
            AVEncoderAudioQualityKey: AVAudioQuality.high.rawValue,
        ]

        do {
            let session = AVAudioSession.sharedInstance()
            try session.setCategory(.playAndRecord, mode: .default, options: [.defaultToSpeaker])
            try session.setActive(true)

            audioRecorder = try AVAudioRecorder(url: fileURL, settings: settings)
            audioRecorder?.record()
            isRecording = true
            startWaveformTap()
        } catch {
            print("Recording failed: \(error)")
        }
    }

    private func startWaveformTap() {
        let inputNode = audioEngine.inputNode
        let format = inputNode.outputFormat(forBus: 0)
        let sampleCount = 128

        inputNode.installTap(onBus: 0, bufferSize: 1024, format: format) { [weak self] buffer, _ in
            guard let data = buffer.floatChannelData?[0] else { return }
            let frameCount = Int(buffer.frameLength)
            let stride = max(1, frameCount / sampleCount)

            var samples = [CGFloat]()
            samples.reserveCapacity(sampleCount)
            for i in Swift.stride(from: 0, to: min(frameCount, sampleCount * stride), by: stride) {
                // Raw PCM sample, typically -1..1
                samples.append(CGFloat(data[i]))
            }
            // Pad if needed
            while samples.count < sampleCount { samples.append(0) }

            Task { @MainActor [weak self] in
                self?.waveform = samples
            }
        }

        do {
            try audioEngine.start()
        } catch {
            print("Audio engine failed: \(error)")
        }
    }

    private func stopWaveformTap() {
        audioEngine.inputNode.removeTap(onBus: 0)
        audioEngine.stop()
        waveform = Array(repeating: 0, count: 128)
    }

    func stopRecordingAndTranscribe() async -> String? {
        stopWaveformTap()
        audioRecorder?.stop()
        isRecording = false

        guard let fileURL = recordingURL else { return nil }

        isTranscribing = true
        defer { isTranscribing = false }

        guard let uploadURL = URL(string: "\(baseURL)/stt") else { return nil }

        do {
            let audioData = try Data(contentsOf: fileURL)
            let boundary = UUID().uuidString
            var request = URLRequest(url: uploadURL)
            request.httpMethod = "POST"
            request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")

            var body = Data()
            // Audio file part
            body.append("--\(boundary)\r\n".data(using: .utf8)!)
            body.append("Content-Disposition: form-data; name=\"file\"; filename=\"audio.m4a\"\r\n".data(using: .utf8)!)
            body.append("Content-Type: audio/m4a\r\n\r\n".data(using: .utf8)!)
            body.append(audioData)
            body.append("\r\n".data(using: .utf8)!)
            // model_id part
            body.append("--\(boundary)\r\n".data(using: .utf8)!)
            body.append("Content-Disposition: form-data; name=\"model_id\"\r\n\r\n".data(using: .utf8)!)
            body.append("scribe_v1".data(using: .utf8)!)
            body.append("\r\n".data(using: .utf8)!)
            body.append("--\(boundary)--\r\n".data(using: .utf8)!)
            request.httpBody = body

            let (data, response) = try await URLSession.shared.data(for: request)
            guard let http = response as? HTTPURLResponse, http.statusCode == 200 else {
                print("STT error: \(String(data: data, encoding: .utf8) ?? "?")")
                return nil
            }

            if let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
               let text = json["text"] as? String {
                return text.trimmingCharacters(in: .whitespacesAndNewlines)
            }
            return nil
        } catch {
            print("STT upload error: \(error)")
            return nil
        }
    }
}
