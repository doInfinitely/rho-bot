import SwiftUI

struct LoginView: View {
    @EnvironmentObject var authVM: AuthViewModel

    @State private var email = ""
    @State private var password = ""
    @State private var isSignup = false

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                Spacer()

                // Logo / branding
                VStack(spacing: 8) {
                    Image(systemName: "brain.head.profile")
                        .font(.system(size: 64))
                        .foregroundStyle(.blue)
                    Text("rho-bot")
                        .font(.largeTitle.bold())
                    Text("AI Desktop Agent")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
                .padding(.bottom, 48)

                // Form
                VStack(spacing: 16) {
                    TextField("Email", text: $email)
                        .textContentType(.emailAddress)
                        .keyboardType(.emailAddress)
                        .autocapitalization(.none)
                        .textFieldStyle(.roundedBorder)

                    SecureField("Password", text: $password)
                        .textContentType(isSignup ? .newPassword : .password)
                        .textFieldStyle(.roundedBorder)

                    if let error = authVM.errorMessage {
                        Text(error)
                            .font(.caption)
                            .foregroundStyle(.red)
                            .multilineTextAlignment(.center)
                    }

                    Button {
                        Task {
                            if isSignup {
                                await authVM.signup(email: email, password: password)
                            } else {
                                await authVM.login(email: email, password: password)
                            }
                        }
                    } label: {
                        if authVM.isLoading {
                            ProgressView()
                                .frame(maxWidth: .infinity)
                        } else {
                            Text(isSignup ? "Sign Up" : "Log In")
                                .frame(maxWidth: .infinity)
                        }
                    }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)
                    .disabled(email.isEmpty || password.isEmpty || authVM.isLoading)
                }
                .padding(.horizontal, 32)

                Spacer()

                // Toggle signup/login
                Button {
                    isSignup.toggle()
                    authVM.errorMessage = nil
                } label: {
                    Text(isSignup
                         ? "Already have an account? Log in"
                         : "Don't have an account? Sign up")
                        .font(.footnote)
                }
                .padding(.bottom, 32)
            }
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}

#Preview {
    LoginView()
        .environmentObject(AuthViewModel())
}
