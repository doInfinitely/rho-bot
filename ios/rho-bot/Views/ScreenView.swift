import SwiftUI

struct ScreenView: View {
    @StateObject private var vm = ScreenViewModel()

    var body: some View {
        NavigationStack {
            ZStack {
                Color.black.ignoresSafeArea()

                if let frame = vm.currentFrame {
                    Image(uiImage: frame)
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                } else if vm.isConnected && !vm.isDesktopOnline {
                    // Connected to server but desktop is offline
                    VStack(spacing: 16) {
                        Image(systemName: "desktopcomputer.trianglebadge.exclamationmark")
                            .font(.system(size: 48))
                            .foregroundStyle(.secondary)
                        Text("Desktop Offline")
                            .font(.title2)
                            .foregroundStyle(.secondary)
                        Text("Start rho-bot on your desktop to stream your screen")
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                            .multilineTextAlignment(.center)
                    }
                    .padding()
                } else {
                    // Connecting
                    VStack(spacing: 16) {
                        ProgressView()
                            .controlSize(.large)
                            .tint(.white)
                        Text("Connecting...")
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    HStack(spacing: 6) {
                        Circle()
                            .fill(vm.isDesktopOnline ? .green : .gray)
                            .frame(width: 8, height: 8)
                        Text(vm.isDesktopOnline ? "Live" : "Offline")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                ToolbarItem(placement: .topBarTrailing) {
                    Menu {
                        ForEach(FrameRate.allCases, id: \.self) { rate in
                            Button {
                                vm.setFrameRate(rate)
                            } label: {
                                HStack {
                                    Text(rate.rawValue)
                                    if vm.frameRate == rate {
                                        Image(systemName: "checkmark")
                                    }
                                }
                            }
                        }
                    } label: {
                        Image(systemName: "gearshape")
                    }
                }
            }
        }
        .onAppear {
            vm.connect()
        }
        .onDisappear {
            vm.disconnect()
        }
    }
}
