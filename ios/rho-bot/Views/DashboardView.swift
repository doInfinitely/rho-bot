import AVFoundation
import SwiftUI
import UIKit

struct DashboardView: View {
    @EnvironmentObject var agentVM: AgentViewModel
    @StateObject private var tts = ElevenLabsService.shared
    @State private var quickTask = ""
    @State private var isRecording = false

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    statusCard
                    goalSection
                    quickTaskSection
                    statsSection
                }
                .padding()
            }
            .onTapGesture {
                UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
            }
            .navigationTitle("Dashboard")
            .refreshable {
                await agentVM.fetchStatus()
            }
        }
    }

    // MARK: - Status Card

    private var statusCard: some View {
        VStack(spacing: 16) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 8) {
                        Circle()
                            .fill(agentVM.status.is_online ? .green : .gray)
                            .frame(width: 12, height: 12)
                        Text(agentVM.status.is_online ? "Agent Online" : "Agent Offline")
                            .font(.headline)
                    }
                    if let sessionId = agentVM.status.session_id {
                        Text("Session: \(String(sessionId.prefix(8)))...")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                Spacer()

                Toggle("", isOn: Binding(
                    get: { agentVM.status.is_online },
                    set: { _ in
                        Task { await agentVM.toggleAgent() }
                    }
                ))
                .labelsHidden()
                .tint(.green)
                .disabled(agentVM.isLoading)
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 16))
    }

    // MARK: - Goal Section

    private var goalSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Agent Goal")
                .font(.headline)

            GoalInputView(
                goalText: $agentVM.goalText,
                isEditing: $agentVM.isEditingGoal
            ) {
                Task { await agentVM.setGoal() }
            }

            if let error = agentVM.errorMessage {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.red)
            }
        }
    }

    // MARK: - Quick Task with Record

    private var quickTaskSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Quick Task")
                .font(.headline)

            HStack(spacing: 8) {
                TextField("Tell Rho what to do...", text: $quickTask)
                    .textFieldStyle(.plain)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 10)
                    .background(.ultraThinMaterial)
                    .clipShape(RoundedRectangle(cornerRadius: 16))

                FluidRecordButton(
                    isRecording: isRecording,
                    onTap: { toggleRecording() },
                    size: 36
                )

                Button {
                    guard !quickTask.trimmingCharacters(in: .whitespaces).isEmpty else { return }
                    WebSocketClient.shared.runTask(quickTask)
                    quickTask = ""
                } label: {
                    Image(systemName: "arrow.up")
                        .font(.body.weight(.semibold))
                        .foregroundStyle(.white)
                        .frame(width: 36, height: 36)
                        .background(quickTask.trimmingCharacters(in: .whitespaces).isEmpty ? .gray : .blue)
                        .clipShape(Circle())
                }
                .disabled(quickTask.trimmingCharacters(in: .whitespaces).isEmpty)
            }
        }
    }

    // MARK: - Stats Section

    private var statsSection: some View {
        HStack(spacing: 16) {
            StatCard(
                title: "Actions",
                value: "\(agentVM.status.total_actions)",
                icon: "bolt.fill"
            )
            StatCard(
                title: "Status",
                value: agentVM.status.is_online ? "Active" : "Idle",
                icon: "power"
            )
        }
    }

    // MARK: - Recording

    private func toggleRecording() {
        if isRecording {
            isRecording = false
            Task {
                if let text = await tts.stopRecordingAndTranscribe() {
                    quickTask = text
                }
            }
        } else {
            AVAudioApplication.requestRecordPermission { granted in
                Task { @MainActor in
                    guard granted else { return }
                    tts.startRecording()
                    isRecording = true
                }
            }
        }
    }
}

struct StatCard: View {
    let title: String
    let value: String
    let icon: String

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundStyle(.blue)
            Text(value)
                .font(.title3.bold())
            Text(title)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

#Preview {
    DashboardView()
        .environmentObject(AgentViewModel())
}
