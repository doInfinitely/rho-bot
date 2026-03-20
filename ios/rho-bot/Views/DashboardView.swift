import AVFoundation
import SwiftUI
import UIKit

struct DashboardView: View {
    @EnvironmentObject var agentVM: AgentViewModel
    @State private var isRecording = false

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                ScrollView {
                    VStack(spacing: 24) {
                        statusCard
                        goalSection
                        statsSection
                    }
                    .padding()
                }
                .onTapGesture {
                    UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
                }

                // Record button centered above tab bar (matches ChatView position)
                FluidRecordButton(
                    isRecording: isRecording,
                    onTap: { toggleRecording() },
                    size: 72
                )
                .frame(maxWidth: .infinity)
                .padding(.top, 4)

                // Spacer to match ChatView's input bar height
                Spacer()
                    .frame(height: 58)
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
                if let text = await ElevenLabsService.shared.stopRecordingAndTranscribe() {
                    agentVM.goalText = text
                    agentVM.isEditingGoal = true
                }
            }
        } else {
            AVAudioApplication.requestRecordPermission { granted in
                Task { @MainActor in
                    guard granted else { return }
                    ElevenLabsService.shared.startRecording()
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
