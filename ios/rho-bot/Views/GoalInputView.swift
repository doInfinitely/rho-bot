import SwiftUI

struct GoalInputView: View {
    @Binding var goalText: String
    @Binding var isEditing: Bool
    var onSubmit: () -> Void

    @FocusState private var isFocused: Bool

    private let placeholders = [
        "Open Safari and book a flight to NYC for March 15",
        "Compose an email to John about the meeting notes",
        "Organize my Downloads folder by file type",
        "Search for restaurant reservations near me",
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            ZStack(alignment: .topLeading) {
                if goalText.isEmpty && !isFocused {
                    Text(placeholders.randomElement() ?? placeholders[0])
                        .foregroundStyle(.tertiary)
                        .padding(.horizontal, 4)
                        .padding(.vertical, 8)
                }

                TextEditor(text: $goalText)
                    .focused($isFocused)
                    .frame(minHeight: 80, maxHeight: 160)
                    .scrollContentBackground(.hidden)
                    .padding(4)
            }
            .background(.ultraThinMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 12))
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(isFocused ? .blue : .clear, lineWidth: 2)
            )
            .onChange(of: isFocused) { _, focused in
                isEditing = focused
            }

            HStack {
                Text("Describe what the agent should do")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                Spacer()

                Button("Set Goal") {
                    isFocused = false
                    isEditing = false
                    onSubmit()
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(goalText.isEmpty)
            }
        }
    }
}

#Preview {
    GoalInputView(goalText: .constant(""), isEditing: .constant(false), onSubmit: {})
        .padding()
}
