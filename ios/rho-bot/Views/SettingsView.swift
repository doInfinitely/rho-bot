import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var authVM: AuthViewModel
    @State private var serverURL: String = APIClient.shared.baseURL

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

                Section("Server") {
                    TextField("Server URL", text: $serverURL)
                        .autocapitalization(.none)
                        .keyboardType(.URL)
                        .textContentType(.URL)
                        .onSubmit {
                            APIClient.shared.baseURL = serverURL
                        }

                    Button("Reset to Default") {
                        serverURL = "https://rho-bot-production.up.railway.app"
                        APIClient.shared.baseURL = serverURL
                    }
                    .font(.caption)
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
        }
    }
}

#Preview {
    SettingsView()
        .environmentObject(AuthViewModel())
}
