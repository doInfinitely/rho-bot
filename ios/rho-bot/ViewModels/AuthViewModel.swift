import Foundation
import SwiftUI

@MainActor
class AuthViewModel: ObservableObject {
    @Published var isAuthenticated = false
    @Published var isLoading = false
    @Published var errorMessage: String?
    @Published var userEmail: String?

    private let api = APIClient.shared

    init() {
        if api.token != nil {
            isAuthenticated = true
            Task { await fetchUser() }
        }
    }

    func signup(email: String, password: String) async {
        isLoading = true
        errorMessage = nil
        do {
            let response = try await api.signup(email: email, password: password)
            api.token = response.access_token
            isAuthenticated = true
            userEmail = email
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    func login(email: String, password: String) async {
        isLoading = true
        errorMessage = nil
        do {
            let response = try await api.login(email: email, password: password)
            api.token = response.access_token
            isAuthenticated = true
            userEmail = email
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    func logout() {
        api.token = nil
        isAuthenticated = false
        userEmail = nil
    }

    private func fetchUser() async {
        do {
            let user: UserInfo = try await withCheckedThrowingContinuation { continuation in
                Task {
                    do {
                        let request = try buildAuthedRequest(path: "/api/me")
                        let (data, response) = try await URLSession.shared.data(for: request)
                        guard let http = response as? HTTPURLResponse, http.statusCode == 200 else {
                            throw APIError.httpError(0, "Failed to fetch user")
                        }
                        let user = try JSONDecoder().decode(UserInfo.self, from: data)
                        continuation.resume(returning: user)
                    } catch {
                        continuation.resume(throwing: error)
                    }
                }
            }
            userEmail = user.email
        } catch {
            logout()
        }
    }

    private func buildAuthedRequest(path: String) throws -> URLRequest {
        guard let url = URL(string: api.baseURL + path) else {
            throw APIError.invalidURL
        }
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        if let token = api.token {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        return request
    }
}
