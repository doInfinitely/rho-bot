import SwiftUI

struct SessionsView: View {
    @EnvironmentObject var agentVM: AgentViewModel

    var body: some View {
        NavigationStack {
            Group {
                if agentVM.sessions.isEmpty {
                    ContentUnavailableView(
                        "No Sessions",
                        systemImage: "list.bullet.rectangle",
                        description: Text("Sessions will appear here once the desktop agent runs.")
                    )
                } else {
                    List(agentVM.sessions) { session in
                        SessionRow(session: session)
                    }
                    .listStyle(.insetGrouped)
                }
            }
            .navigationTitle("Sessions")
            .task {
                await agentVM.fetchSessions()
            }
            .refreshable {
                await agentVM.fetchSessions()
            }
        }
    }
}

struct SessionRow: View {
    let session: SessionSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Image(systemName: session.ended_at == nil ? "circle.fill" : "circle")
                    .font(.caption2)
                    .foregroundStyle(session.ended_at == nil ? .green : .gray)

                Text(session.startedDate, style: .date)
                    .font(.subheadline.bold())
                Text(session.startedDate, style: .time)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                Spacer()

                Text("\(session.action_count) actions")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            if !session.goal.isEmpty {
                Text(session.goal)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Text(session.durationString)
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
        .padding(.vertical, 4)
    }
}
