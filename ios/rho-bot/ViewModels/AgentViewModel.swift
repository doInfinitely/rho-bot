import Foundation
import SwiftUI
import Combine

@MainActor
class AgentViewModel: ObservableObject {
    @Published var status = AgentStatus()
    @Published var goalText: String = ""
    @Published var isLoading = false
    @Published var errorMessage: String?
    @Published var sessions: [SessionSummary] = []

    /// True while the user is actively editing the goal field.
    @Published var isEditingGoal = false

    private let api = APIClient.shared
    private let ws = WebSocketClient.shared
    private var cancellables = Set<AnyCancellable>()
    private var pollTimer: Timer?
    private var didInitialGoalSync = false

    init() {
        ws.$isConnected
            .receive(on: RunLoop.main)
            .sink { [weak self] connected in
                self?.status.is_online = connected
            }
            .store(in: &cancellables)
    }

    func fetchStatus() async {
        do {
            status = try await api.getAgentStatus()
            // Force WS truth so REST doesn't override connection state
            status.is_online = ws.isConnected
            // Only overwrite the text field on the first fetch or when the
            // user isn't actively editing.  This prevents the 5-second poll
            // from clobbering what they're typing.
            if !isEditingGoal && !didInitialGoalSync {
                goalText = status.goal
                didInitialGoalSync = true
            }
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    func setGoal() async {
        isLoading = true
        errorMessage = nil
        do {
            let response = try await api.setGoal(goalText)
            status = AgentStatus(
                session_id: response.session_id,
                is_online: ws.isConnected,
                last_seen: status.last_seen,
                total_actions: status.total_actions,
                goal: response.goal
            )
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    func startAgent() {
        ws.connect()
    }

    func stopAgent() {
        ws.disconnect()
    }

    func toggleAgent() {
        if ws.isConnected {
            stopAgent()
        } else {
            startAgent()
        }
    }

    func fetchSessions() async {
        do {
            sessions = try await api.getSessions()
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    func startPolling() {
        pollTimer?.invalidate()
        pollTimer = Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { [weak self] _ in
            guard let self else { return }
            Task { @MainActor in
                await self.fetchStatus()
            }
        }
    }

    func stopPolling() {
        pollTimer?.invalidate()
        pollTimer = nil
    }
}
