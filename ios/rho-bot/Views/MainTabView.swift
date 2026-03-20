import SwiftUI

struct MainTabView: View {
    @EnvironmentObject var authVM: AuthViewModel
    @StateObject private var agentVM = AgentViewModel()

    var body: some View {
        TabView {
            ChatView()
                .tabItem {
                    Label("Rho", systemImage: "bubble.left.and.bubble.right")
                }

            DashboardView()
                .environmentObject(agentVM)
                .tabItem {
                    Label("Dashboard", systemImage: "gauge.medium")
                }

            SessionsView()
                .environmentObject(agentVM)
                .tabItem {
                    Label("Sessions", systemImage: "list.bullet.rectangle")
                }

            SettingsView()
                .tabItem {
                    Label("Settings", systemImage: "gear")
                }
        }
        .task {
            await agentVM.fetchStatus()
            agentVM.startPolling()
        }
        .onDisappear {
            agentVM.stopPolling()
        }
    }
}
