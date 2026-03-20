import SwiftUI
import UIKit

struct ChatView: View {
    @StateObject private var vm = ChatViewModel()

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Messages
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(spacing: 8) {
                            ForEach(vm.messages) { msg in
                                ChatBubble(message: msg)
                                    .id(msg.id)
                            }
                        }
                        .padding(.horizontal)
                        .padding(.top, 8)
                    }
                    .onTapGesture {
                        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
                    }
                    .onChange(of: vm.messages.count) {
                        if let last = vm.messages.last {
                            withAnimation {
                                proxy.scrollTo(last.id, anchor: .bottom)
                            }
                        }
                    }
                }

                // Record button centered above input
                FluidRecordButton(
                    isRecording: vm.isRecordingAudio,
                    onTap: { vm.toggleRecording() },
                    size: 72,
                    waveform: vm.waveform
                )
                .frame(maxWidth: .infinity)
                .padding(.top, 4)

                Divider()

                // Input bar
                HStack(spacing: 8) {
                    TextField(
                        vm.awaitingUserResponse ? "Reply to Rho..." : "Ask Rho to do something...",
                        text: $vm.inputText
                    )
                    .textFieldStyle(.plain)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 10)
                    .background(.ultraThinMaterial)
                    .clipShape(RoundedRectangle(cornerRadius: 20))

                    if vm.isRunning {
                        Button { vm.stop() } label: {
                            Image(systemName: "stop.fill")
                                .font(.body)
                                .foregroundStyle(.white)
                                .frame(width: 36, height: 36)
                                .background(.red)
                                .clipShape(Circle())
                        }
                    }

                    Button { vm.send() } label: {
                        Image(systemName: "arrow.up")
                            .font(.body.weight(.semibold))
                            .foregroundStyle(.white)
                            .frame(width: 36, height: 36)
                            .background(vm.inputText.trimmingCharacters(in: .whitespaces).isEmpty ? .gray : .blue)
                            .clipShape(Circle())
                    }
                    .disabled(vm.inputText.trimmingCharacters(in: .whitespaces).isEmpty)
                }
                .padding(.horizontal)
                .padding(.vertical, 8)
            }
            .navigationTitle("Rho")
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    HStack(spacing: 6) {
                        Circle()
                            .fill(vm.isConnected ? .green : .red)
                            .frame(width: 8, height: 8)
                        Text(vm.isConnected ? "Connected" : "Offline")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                ToolbarItem(placement: .topBarTrailing) {
                    Button { vm.clearMessages() } label: {
                        Image(systemName: "trash")
                    }
                    .disabled(vm.messages.isEmpty)
                }
            }
            .onDisappear {
                vm.disconnect()
            }
        }
    }
}

// MARK: - Chat Bubble

struct ChatBubble: View {
    let message: ChatMessage

    var body: some View {
        switch message.type {
        case .user:
            userBubble
        case .thinking:
            agentBubble(icon: "brain", iconColor: .gray, bg: Color(.systemGray6), text: message.content)
        case .action:
            agentBubble(icon: "bolt.fill", iconColor: .orange, bg: Color.orange.opacity(0.1), text: message.content, mono: true)
        case .result:
            agentBubble(icon: "checkmark.circle.fill", iconColor: .green, bg: Color.green.opacity(0.1), text: message.content)
        case .error:
            agentBubble(icon: "xmark.circle.fill", iconColor: .red, bg: Color.red.opacity(0.1), text: message.content)
        case .askUser:
            agentBubble(icon: "questionmark.circle.fill", iconColor: .blue, bg: Color.blue.opacity(0.1), text: message.content)
        case .complete:
            agentBubble(icon: "checkmark.seal.fill", iconColor: .green, bg: Color.green.opacity(0.15), text: message.content)
        }
    }

    private var userBubble: some View {
        HStack {
            Spacer()
            Text(message.content)
                .font(.body)
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .background(.blue)
                .foregroundStyle(.white)
                .clipShape(RoundedRectangle(cornerRadius: 16))
        }
    }

    private func agentBubble(icon: String, iconColor: Color, bg: Color, text: String, mono: Bool = false) -> some View {
        HStack(alignment: .top, spacing: 8) {
            Image(systemName: icon)
                .font(.caption)
                .foregroundStyle(iconColor)
                .frame(width: 20, height: 20)
                .padding(.top, 2)

            Text(text)
                .font(mono ? .system(.callout, design: .monospaced) : .callout)
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(bg)
                .clipShape(RoundedRectangle(cornerRadius: 12))

            Spacer(minLength: 40)
        }
    }
}

#Preview {
    ChatView()
}
