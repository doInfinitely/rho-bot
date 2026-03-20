import Foundation

struct AgentStatus: Codable {
    let session_id: String?
    var is_online: Bool
    let last_seen: Double?
    let total_actions: Int
    let goal: String

    init(
        session_id: String? = nil,
        is_online: Bool = false,
        last_seen: Double? = nil,
        total_actions: Int = 0,
        goal: String = ""
    ) {
        self.session_id = session_id
        self.is_online = is_online
        self.last_seen = last_seen
        self.total_actions = total_actions
        self.goal = goal
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        session_id = try c.decodeIfPresent(String.self, forKey: .session_id)
        is_online = (try? c.decode(Bool.self, forKey: .is_online)) ?? false
        last_seen = try c.decodeIfPresent(Double.self, forKey: .last_seen)
        total_actions = (try? c.decode(Int.self, forKey: .total_actions)) ?? 0
        goal = (try? c.decode(String.self, forKey: .goal)) ?? ""
    }
}

struct GoalRequest: Codable {
    let goal: String
}

struct GoalResponse: Codable {
    let goal: String
    let session_id: String?

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        goal = (try? c.decode(String.self, forKey: .goal)) ?? ""
        session_id = try c.decodeIfPresent(String.self, forKey: .session_id)
    }
}

struct SessionSummary: Codable, Identifiable {
    let session_id: String
    let started_at: Double
    let ended_at: Double?
    let action_count: Int
    let goal: String

    var id: String { session_id }

    var startedDate: Date {
        Date(timeIntervalSince1970: started_at)
    }

    var endedDate: Date? {
        guard let ended_at else { return nil }
        return Date(timeIntervalSince1970: ended_at)
    }

    var durationString: String {
        guard let endedDate else { return "Active" }
        let interval = endedDate.timeIntervalSince(startedDate)
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.hour, .minute, .second]
        formatter.unitsStyle = .abbreviated
        return formatter.string(from: interval) ?? ""
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        session_id = try c.decode(String.self, forKey: .session_id)
        started_at = try c.decode(Double.self, forKey: .started_at)
        ended_at = try c.decodeIfPresent(Double.self, forKey: .ended_at)
        action_count = (try? c.decode(Int.self, forKey: .action_count)) ?? 0
        goal = (try? c.decode(String.self, forKey: .goal)) ?? ""
    }
}
