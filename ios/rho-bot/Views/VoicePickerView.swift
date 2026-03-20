import SwiftUI

/// Searchable dropdown for picking an ElevenLabs voice.
struct VoicePickerView: View {
    let voices: [ElevenLabsService.Voice]
    @Binding var selectedVoiceId: String
    @State private var isPresented = false
    @State private var search = ""

    private var selectedName: String {
        voices.first(where: { $0.voice_id == selectedVoiceId })?.name ?? "Select voice"
    }

    var body: some View {
        Button {
            isPresented = true
        } label: {
            HStack {
                Text("Voice")
                    .foregroundStyle(.primary)
                Spacer()
                Text(selectedName)
                    .foregroundStyle(.secondary)
                Image(systemName: "chevron.up.chevron.down")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .sheet(isPresented: $isPresented) {
            NavigationStack {
                List {
                    if voices.isEmpty {
                        Text("Loading voices...")
                            .foregroundStyle(.secondary)
                    }
                    ForEach(filteredVoices) { voice in
                        Button {
                            selectedVoiceId = voice.voice_id
                            isPresented = false
                        } label: {
                            HStack {
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(voice.name)
                                        .foregroundStyle(.primary)
                                    Text(voice.category)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                                if voice.voice_id == selectedVoiceId {
                                    Image(systemName: "checkmark")
                                        .foregroundStyle(.blue)
                                }
                            }
                        }
                    }
                }
                .searchable(text: $search, prompt: "Search voices")
                .navigationTitle("Choose Voice")
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button("Done") { isPresented = false }
                    }
                }
            }
            .presentationDetents([.medium, .large])
        }
    }

    private var filteredVoices: [ElevenLabsService.Voice] {
        if search.isEmpty { return voices }
        return voices.filter { $0.name.localizedCaseInsensitiveContains(search) }
    }
}
