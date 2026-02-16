import Foundation

struct TokenResponse: Codable {
    let access_token: String
    let token_type: String
}

struct UserInfo: Codable {
    let id: String
    let email: String
}

struct AuthRequest: Codable {
    let email: String
    let password: String
}
