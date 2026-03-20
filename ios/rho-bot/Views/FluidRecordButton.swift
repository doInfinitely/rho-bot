import SwiftUI
import MetalKit

// MARK: - Fluid sim config
fileprivate let gridN: Int = 32
fileprivate let jacobiIters = 16
fileprivate let simDt: Float = 1.0/60.0
fileprivate let particleCap = 8000
fileprivate let emitPerFrame = 40
fileprivate let emitRad: Float = 0.15
fileprivate let ptSize: Float = 24.0

/// A record button with GPU fluid particle sim when recording.
struct FluidRecordButton: View {
    let isRecording: Bool
    let onTap: () -> Void
    var size: CGFloat = 48

    @State private var pulse = false
    @State private var rotation: Double = 0

    var body: some View {
        Button(action: onTap) {
            ZStack {
                // Pulsing ring animation (always visible when recording)
                if isRecording {
                    Circle()
                        .stroke(
                            AngularGradient(
                                colors: [.blue, .cyan, .blue.opacity(0.3), .cyan, .blue],
                                center: .center
                            ),
                            lineWidth: 3
                        )
                        .frame(width: size + 14, height: size + 14)
                        .rotationEffect(.degrees(rotation))

                    Circle()
                        .fill(Color.blue.opacity(0.15))
                        .frame(width: size + 8, height: size + 8)
                        .scaleEffect(pulse ? 1.15 : 0.95)
                }

                // Metal fluid sim layer (behind the mic icon, clipped to circle)
                if isRecording {
                    FluidCircleView(isActive: true)
                        .frame(width: size + 20, height: size + 20)
                        .clipShape(Circle())
                        .allowsHitTesting(false)
                }

                // Mic icon
                Circle()
                    .fill(isRecording ? Color.blue.opacity(0.85) : Color(.systemGray6))
                    .frame(width: size, height: size)
                    .overlay(
                        Image(systemName: isRecording ? "mic.fill" : "mic")
                            .font(.system(size: size * 0.4, weight: .medium))
                            .foregroundStyle(isRecording ? .white : .blue)
                    )
            }
            .frame(width: size + 22, height: size + 22)
        }
        .buttonStyle(.plain)
        .onChange(of: isRecording) {
            if isRecording {
                withAnimation(.easeInOut(duration: 0.8).repeatForever(autoreverses: true)) {
                    pulse = true
                }
                withAnimation(.linear(duration: 3).repeatForever(autoreverses: false)) {
                    rotation = 360
                }
            } else {
                withAnimation(.default) {
                    pulse = false
                    rotation = 0
                }
            }
        }
    }
}

// MARK: - Metal fluid view (UIViewRepresentable)

struct FluidCircleView: UIViewRepresentable {
    let isActive: Bool

    func makeUIView(context: Context) -> UIView {
        let container = UIView()
        container.backgroundColor = .clear
        container.isUserInteractionEnabled = false

        guard let device = MTLCreateSystemDefaultDevice() else { return container }

        let metalView = MTKView(frame: .zero, device: device)
        metalView.framebufferOnly = false
        metalView.isPaused = false
        metalView.enableSetNeedsDisplay = false
        metalView.preferredFramesPerSecond = 60
        metalView.colorPixelFormat = .bgra8Unorm
        metalView.clearColor = MTLClearColorMake(0, 0, 0, 0)
        metalView.isOpaque = false
        metalView.backgroundColor = .clear
        metalView.layer.isOpaque = false
        metalView.isUserInteractionEnabled = false
        metalView.autoresizingMask = [.flexibleWidth, .flexibleHeight]

        let coordinator = context.coordinator
        coordinator.metalView = metalView
        coordinator.setup(device: device)
        metalView.delegate = coordinator

        container.addSubview(metalView)
        metalView.translatesAutoresizingMaskIntoConstraints = false
        NSLayoutConstraint.activate([
            metalView.leadingAnchor.constraint(equalTo: container.leadingAnchor),
            metalView.trailingAnchor.constraint(equalTo: container.trailingAnchor),
            metalView.topAnchor.constraint(equalTo: container.topAnchor),
            metalView.bottomAnchor.constraint(equalTo: container.bottomAnchor),
        ])

        return container
    }

    func updateUIView(_ uiView: UIView, context: Context) {
        context.coordinator.isActive = isActive
    }

    func makeCoordinator() -> FluidCoordinator { FluidCoordinator() }
}

final class FluidCoordinator: NSObject, MTKViewDelegate {
    var device: MTLDevice!
    var queue: MTLCommandQueue!
    var lib: MTLLibrary!
    var ready = false
    weak var metalView: MTKView?

    var pClear: MTLComputePipelineState!
    var pBrush: MTLComputePipelineState!
    var pAdvect: MTLComputePipelineState!
    var pDivergence: MTLComputePipelineState!
    var pPressureJacobi: MTLComputePipelineState!
    var pSubtractGradient: MTLComputePipelineState!
    var pAdvectParticles: MTLComputePipelineState!
    var psoParticles: MTLRenderPipelineState!

    var velA: MTLTexture!, velB: MTLTexture!
    var divTex: MTLTexture!
    var pressA: MTLTexture!, pressB: MTLTexture!
    var dyeA: MTLTexture!, dyeB: MTLTexture!

    var paramsBuf: MTLBuffer!
    var brushBuf: MTLBuffer!
    var particlesBuf: MTLBuffer!
    var particleRenderParams: MTLBuffer!
    var stepBuf: MTLBuffer!

    var isActive = false
    var stepCount: UInt32 = 0
    var particleHead = 0
    var particleCount = 0
    var frameCount = 0

    struct SimParams { var N: UInt32; var dt: Float; var visc: Float; var invTexSize: SIMD2<Float>; var dyeDissipation: Float }
    struct Brush { var pos: SIMD2<Float>; var force: SIMD2<Float>; var radius: Float; var strength: Float; var enabled: UInt32 }
    struct Particle { var pos: SIMD2<Float>; var alive: Float }
    struct PRParams { var pointSizePx: Float; var darkness: Float; var viewport: SIMD2<Float> }

    func setup(device: MTLDevice) {
        self.device = device
        queue = device.makeCommandQueue()
        guard let library = device.makeDefaultLibrary() else { return }
        lib = library

        func cp(_ n: String) -> MTLComputePipelineState? {
            guard let f = lib.makeFunction(name: n) else { return nil }
            return try? device.makeComputePipelineState(function: f)
        }
        pClear = cp("kClear"); pBrush = cp("kBrush"); pAdvect = cp("kAdvect")
        pDivergence = cp("kDivergence"); pPressureJacobi = cp("kPressureJacobi")
        pSubtractGradient = cp("kSubtractGradient"); pAdvectParticles = cp("kAdvectParticles")

        guard pClear != nil, pBrush != nil, pAdvectParticles != nil else { return }

        guard let vfn = lib.makeFunction(name: "particleVS"),
              let ffn = lib.makeFunction(name: "particleFS") else { return }
        let rpd = MTLRenderPipelineDescriptor()
        rpd.vertexFunction = vfn; rpd.fragmentFunction = ffn
        rpd.colorAttachments[0].pixelFormat = .bgra8Unorm
        let att = rpd.colorAttachments[0]!
        att.isBlendingEnabled = true
        att.rgbBlendOperation = .add; att.alphaBlendOperation = .add
        att.sourceRGBBlendFactor = .sourceAlpha; att.sourceAlphaBlendFactor = .sourceAlpha
        att.destinationRGBBlendFactor = .oneMinusSourceAlpha; att.destinationAlphaBlendFactor = .oneMinusSourceAlpha
        guard let pso = try? device.makeRenderPipelineState(descriptor: rpd) else { return }
        psoParticles = pso

        makeTextures(); makeBuffers()
        ready = true
    }

    func makeTextures() {
        func makeTex(_ fmt: MTLPixelFormat) -> MTLTexture {
            let d = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: fmt, width: gridN, height: gridN, mipmapped: false)
            d.usage = [.shaderRead, .shaderWrite]; d.storageMode = .private
            return device.makeTexture(descriptor: d)!
        }
        velA = makeTex(.rg16Float); velB = makeTex(.rg16Float)
        divTex = makeTex(.r16Float); pressA = makeTex(.r16Float); pressB = makeTex(.r16Float)
        dyeA = makeTex(.rgba8Unorm); dyeB = makeTex(.rgba8Unorm)

        let cmd = queue.makeCommandBuffer()!; let enc = cmd.makeComputeCommandEncoder()!
        enc.setComputePipelineState(pClear)
        for tex in [velA!, velB!, divTex!, pressA!, pressB!, dyeA!, dyeB!] {
            enc.setTexture(tex, index: 0); d2D(enc, gridN, gridN)
        }
        enc.endEncoding(); cmd.commit(); cmd.waitUntilCompleted()
    }

    func makeBuffers() {
        paramsBuf = device.makeBuffer(length: MemoryLayout<SimParams>.stride, options: .storageModeShared)
        brushBuf = device.makeBuffer(length: MemoryLayout<Brush>.stride, options: .storageModeShared)
        stepBuf = device.makeBuffer(length: MemoryLayout<UInt32>.stride, options: .storageModeShared)
        particlesBuf = device.makeBuffer(length: MemoryLayout<Particle>.stride * particleCap, options: .storageModeShared)
        particleRenderParams = device.makeBuffer(length: MemoryLayout<PRParams>.stride, options: .storageModeShared)
        let p = particlesBuf.contents().bindMemory(to: Particle.self, capacity: particleCap)
        for i in 0..<particleCap { p[i] = Particle(pos: .init(-1, -1), alive: 0) }
    }

    func emit(at c: SIMD2<Float>, n: Int) {
        let buf = particlesBuf.contents().bindMemory(to: Particle.self, capacity: particleCap)
        for _ in 0..<n {
            let a = Float.random(in: 0..<(2 * .pi))
            let r = Float.random(in: 0..<1).squareRoot() * emitRad
            buf[particleHead] = Particle(pos: c + .init(r * cos(a), r * sin(a)), alive: 1)
            particleHead = (particleHead + 1) % particleCap
            particleCount = min(particleCount + 1, particleCap)
        }
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        guard ready, let cmd = queue.makeCommandBuffer(),
              let enc = cmd.makeComputeCommandEncoder() else { return }

        var P = SimParams(N: UInt32(gridN), dt: simDt, visc: 0,
                          invTexSize: .init(1/Float(gridN), 1/Float(gridN)), dyeDissipation: 1)
        memcpy(paramsBuf.contents(), &P, MemoryLayout<SimParams>.stride)

        let c = SIMD2<Float>(0.5, 0.5)
        if isActive {
            emit(at: c, n: emitPerFrame)
            let ang = Float(frameCount) * 0.05
            var B = Brush(pos: c, force: .init(cos(ang), sin(ang)) * 0.4 * Float(gridN),
                          radius: 0.3, strength: 1, enabled: 1)
            memcpy(brushBuf.contents(), &B, MemoryLayout<Brush>.stride)
        } else {
            var B = Brush(pos: c, force: .zero, radius: 0, strength: 0, enabled: 0)
            memcpy(brushBuf.contents(), &B, MemoryLayout<Brush>.stride)
        }

        stepCount &+= 1
        memcpy(stepBuf.contents(), &stepCount, MemoryLayout<UInt32>.stride)

        // Fluid sim
        enc.setComputePipelineState(pBrush)
        enc.setTexture(velA, index: 0); enc.setTexture(dyeA, index: 1)
        enc.setTexture(velB, index: 2); enc.setTexture(dyeB, index: 3)
        enc.setBuffer(paramsBuf, offset: 0, index: 0); enc.setBuffer(brushBuf, offset: 0, index: 1)
        d2D(enc, gridN, gridN); swap(&velA, &velB); swap(&dyeA, &dyeB)

        enc.setComputePipelineState(pDivergence)
        enc.setTexture(velA, index: 0); enc.setTexture(divTex, index: 1)
        enc.setBuffer(paramsBuf, offset: 0, index: 0); d2D(enc, gridN, gridN)

        enc.setComputePipelineState(pClear)
        enc.setTexture(pressA, index: 0); d2D(enc, gridN, gridN)
        enc.setTexture(pressB, index: 0); d2D(enc, gridN, gridN)

        for _ in 0..<jacobiIters {
            enc.setComputePipelineState(pPressureJacobi)
            enc.setTexture(pressA, index: 0); enc.setTexture(divTex, index: 1)
            enc.setTexture(pressB, index: 2); enc.setBuffer(paramsBuf, offset: 0, index: 0)
            d2D(enc, gridN, gridN); swap(&pressA, &pressB)
        }

        enc.setComputePipelineState(pSubtractGradient)
        enc.setTexture(pressA, index: 0); enc.setTexture(velA, index: 1)
        enc.setTexture(velB, index: 2); enc.setBuffer(paramsBuf, offset: 0, index: 0)
        d2D(enc, gridN, gridN); swap(&velA, &velB)

        enc.setComputePipelineState(pAdvect)
        enc.setTexture(velA, index: 0); enc.setTexture(velA, index: 1)
        enc.setTexture(velB, index: 2); enc.setBuffer(paramsBuf, offset: 0, index: 0)
        d2D(enc, gridN, gridN); swap(&velA, &velB)

        enc.setComputePipelineState(pAdvectParticles)
        enc.setTexture(velA, index: 0)
        enc.setBuffer(particlesBuf, offset: 0, index: 0)
        enc.setBuffer(paramsBuf, offset: 0, index: 1)
        enc.setBuffer(stepBuf, offset: 0, index: 2)
        d1D(enc, particleCap)

        enc.endEncoding(); cmd.commit(); cmd.waitUntilCompleted()

        // Render particles
        guard let rc = queue.makeCommandBuffer(), let drawable = view.currentDrawable else { return }
        let rpd = MTLRenderPassDescriptor()
        rpd.colorAttachments[0].texture = drawable.texture
        rpd.colorAttachments[0].loadAction = .clear
        rpd.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0)
        rpd.colorAttachments[0].storeAction = .store

        let renc = rc.makeRenderCommandEncoder(descriptor: rpd)!
        renc.setRenderPipelineState(psoParticles)
        var pr = PRParams(pointSizePx: ptSize, darkness: 1,
                          viewport: .init(Float(view.drawableSize.width), Float(view.drawableSize.height)))
        memcpy(particleRenderParams.contents(), &pr, MemoryLayout<PRParams>.stride)
        renc.setVertexBuffer(particlesBuf, offset: 0, index: 0)
        renc.setVertexBuffer(particleRenderParams, offset: 0, index: 1)
        renc.drawPrimitives(type: .point, vertexStart: 0, vertexCount: particleCap)
        renc.endEncoding()
        rc.present(drawable); rc.commit()
        frameCount += 1
    }

    func d2D(_ e: MTLComputeCommandEncoder, _ w: Int, _ h: Int) {
        e.dispatchThreadgroups(MTLSize(width: (w+15)/16, height: (h+15)/16, depth: 1),
                               threadsPerThreadgroup: MTLSize(width: 16, height: 16, depth: 1))
    }
    func d1D(_ e: MTLComputeCommandEncoder, _ c: Int) {
        e.dispatchThreadgroups(MTLSize(width: (c+63)/64, height: 1, depth: 1),
                               threadsPerThreadgroup: MTLSize(width: 64, height: 1, depth: 1))
    }
}

#Preview {
    VStack(spacing: 20) {
        FluidRecordButton(isRecording: false, onTap: {}, size: 72)
        FluidRecordButton(isRecording: true, onTap: {}, size: 72)
    }
}
