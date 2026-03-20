import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var authVM: AuthViewModel
    @StateObject private var tts = ElevenLabsService.shared

    var body: some View {
        NavigationStack {
            Form {
                Section("Account") {
                    if let email = authVM.userEmail {
                        HStack {
                            Text("Email")
                            Spacer()
                            Text(email)
                                .foregroundStyle(.secondary)
                        }
                    }
                }

                Section("Text to Speech") {
                    Toggle("Enable TTS", isOn: $tts.ttsEnabled)

                    if tts.ttsEnabled {
                        VoicePickerView(voices: tts.voices, selectedVoiceId: $tts.selectedVoiceId)

                        Button("Test Voice") {
                            Task { await tts.speak("Hello, I'm Rho. How can I help you today?") }
                        }
                        .disabled(tts.isPlaying)
                    }
                }

                Section("About") {
                    HStack {
                        Text("Version")
                        Spacer()
                        Text("1.0.0")
                            .foregroundStyle(.secondary)
                    }
                }

                Section {
                    Button("Log Out", role: .destructive) {
                        authVM.logout()
                    }
                }
            }
            .navigationTitle("Settings")
            .task {
                if tts.voices.isEmpty {
                    await tts.fetchVoices()
                }
            }
        }
    }
}

#Preview {
    SettingsView()
        .environmentObject(AuthViewModel())
}
