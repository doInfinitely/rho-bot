import Foundation

enum APIError: LocalizedError {
    case invalidURL
    case httpError(Int, String)
    case decodingError(Error)
    case noToken
    case unknown(Error)

    var errorDescription: String? {
        switch self {
        case .invalidURL: return "Invalid server URL"
        case .httpError(let code, let msg): return "Server error \(code): \(msg)"
        case .decodingError(let err): return "Decode error: \(err.localizedDescription)"
        case .noToken: return "Not authenticated"
        case .unknown(let err): return err.localizedDescription
        }
    }
}

class APIClient: ObservableObject {
    static let shared = APIClient()

    @Published var baseURL: String {
        didSet { UserDefaults.standard.set(baseURL, forKey: "server_url") }
    }

    var token: String? {
        get { KeychainHelper.load(key: "auth_token") }
        set {
            if let newValue {
                KeychainHelper.save(key: "auth_token", value: newValue)
            } else {
                KeychainHelper.delete(key: "auth_token")
            }
        }
    }

    private init() {
        self.baseURL = UserDefaults.standard.string(forKey: "server_url")
            ?? "https://rho-bot-production.up.railway.app"
    }

    // MARK: - Auth

    func signup(email: String, password: String) async throws -> TokenResponse {
        let body = AuthRequest(email: email, password: password)
        return try await post("/auth/signup", body: body)
    }

    func login(email: String, password: String) async throws -> TokenResponse {
        let body = AuthRequest(email: email, password: password)
        return try await post("/auth/login", body: body)
    }

    // MARK: - Agent

    func getAgentStatus() async throws -> AgentStatus {
        try await get("/api/agent/status")
    }

    func getGoal() async throws -> GoalResponse {
        try await get("/api/agent/goal")
    }

    func setGoal(_ goal: String) async throws -> GoalResponse {
        let body = GoalRequest(goal: goal)
        return try await post("/api/agent/goal", body: body)
    }

    func startAgent() async throws -> AgentStatus {
        try await post("/api/agent/start", body: EmptyBody())
    }

    func stopAgent() async throws -> AgentStatus {
        try await post("/api/agent/stop", body: EmptyBody())
    }

    // MARK: - Sessions

    func getSessions(limit: Int = 20, offset: Int = 0) async throws -> [SessionSummary] {
        try await get("/api/sessions?limit=\(limit)&offset=\(offset)")
    }

    // MARK: - Networking

    private func get<T: Decodable>(_ path: String) async throws -> T {
        let request = try buildRequest(path: path, method: "GET")
        return try await execute(request)
    }

    private func post<T: Decodable, B: Encodable>(_ path: String, body: B) async throws -> T {
        var request = try buildRequest(path: path, method: "POST")
        request.httpBody = try JSONEncoder().encode(body)
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        return try await execute(request)
    }

    private func buildRequest(path: String, method: String) throws -> URLRequest {
        guard let url = URL(string: baseURL + path) else {
            throw APIError.invalidURL
        }
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.timeoutInterval = 30

        if let token {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        return request
    }

    private func execute<T: Decodable>(_ request: URLRequest) async throws -> T {
        let (data, response): (Data, URLResponse)
        do {
            (data, response) = try await URLSession.shared.data(for: request)
        } catch {
            throw APIError.unknown(error)
        }

        guard let http = response as? HTTPURLResponse else {
            throw APIError.unknown(NSError(domain: "", code: -1))
        }

        guard (200...299).contains(http.statusCode) else {
            let body = String(data: data, encoding: .utf8) ?? ""
            throw APIError.httpError(http.statusCode, body)
        }

        do {
            return try JSONDecoder().decode(T.self, from: data)
        } catch {
            throw APIError.decodingError(error)
        }
    }
}

private struct EmptyBody: Encodable {}
