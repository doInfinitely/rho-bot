import SwiftUI
import MetalKit

// MARK: - Compact fluid sim config for the record button
fileprivate let N: Int = 32
fileprivate let jacobiIters = 16
fileprivate let dt: Float = 1.0/60.0
fileprivate let particleCapacity = 8000
fileprivate let emitPerFrame = 40
fileprivate let emitRadius: Float = 0.15
fileprivate let pointSizePx: Float = 24.0

/// A circular record button with a fluid particle simulation background.
struct FluidRecordButton: View {
    let isRecording: Bool
    let onTap: () -> Void
    var size: CGFloat = 48

    var body: some View {
        ZStack {
            // Fluid sim behind the button
            FluidCircleView(isActive: isRecording)
                .frame(width: size + 16, height: size + 16)
                .clipShape(Circle())

            // Mic icon
            Image(systemName: isRecording ? "mic.fill" : "mic")
                .font(.system(size: size * 0.4, weight: .medium))
                .foregroundStyle(isRecording ? .white : .blue)
                .frame(width: size, height: size)
                .background(isRecording ? .blue : Color(.systemGray6))
                .clipShape(Circle())
                .scaleEffect(isRecording ? 1.1 : 1.0)
                .animation(.easeInOut(duration: 0.2), value: isRecording)
        }
        .onTapGesture(perform: onTap)
    }
}

// MARK: - Metal-backed circular fluid view

struct FluidCircleView: UIViewRepresentable {
    let isActive: Bool

    func makeUIView(context: Context) -> MTKView {
        let v = MTKView()
        v.device = MTLCreateSystemDefaultDevice()
        v.framebufferOnly = false
        v.isPaused = false
        v.enableSetNeedsDisplay = false
        v.preferredFramesPerSecond = 60
        v.colorPixelFormat = .bgra8Unorm
        v.clearColor = MTLClearColorMake(0, 0, 0, 0)
        v.isOpaque = false
        v.backgroundColor = .clear
        v.layer.isOpaque = false
        v.delegate = context.coordinator
        v.isUserInteractionEnabled = false
        context.coordinator.view = v
        context.coordinator.setup()
        return v
    }

    func updateUIView(_ uiView: MTKView, context: Context) {
        context.coordinator.isActive = isActive
    }

    func makeCoordinator() -> FluidCoordinator { FluidCoordinator() }
}

final class FluidCoordinator: NSObject, MTKViewDelegate {
    var device: MTLDevice!
    var queue: MTLCommandQueue!
    var lib: MTLLibrary!

    var pClear: MTLComputePipelineState!
    var pBrush: MTLComputePipelineState!
    var pAdvect: MTLComputePipelineState!
    var pJacobi: MTLComputePipelineState!
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

    weak var view: MTKView?
    var isActive = false
    var stepCount: UInt32 = 0
    var particleHead = 0
    var particleCount = 0
    var frameCount = 0

    struct SimParams { var N: UInt32; var dt: Float; var visc: Float; var invTexSize: SIMD2<Float>; var dyeDissipation: Float }
    struct Brush { var pos: SIMD2<Float>; var force: SIMD2<Float>; var radius: Float; var strength: Float; var enabled: UInt32 }
    struct Particle { var pos: SIMD2<Float>; var alive: Float }
    struct ParticleRenderParams { var pointSizePx: Float; var darkness: Float; var viewport: SIMD2<Float> }

    func setup() {
        guard let dev = MTLCreateSystemDefaultDevice() else { return }
        device = dev
        queue = device.makeCommandQueue()
        lib = try? device.makeDefaultLibrary(bundle: .main)
        guard lib != nil else { return }

        func cp(_ name: String) -> MTLComputePipelineState {
            try! device.makeComputePipelineState(function: lib.makeFunction(name: name)!)
        }
        pClear = cp("kClear"); pBrush = cp("kBrush"); pAdvect = cp("kAdvect")
        pJacobi = cp("kJacobi"); pDivergence = cp("kDivergence")
        pPressureJacobi = cp("kPressureJacobi"); pSubtractGradient = cp("kSubtractGradient")
        pAdvectParticles = cp("kAdvectParticles")

        let vfn = lib.makeFunction(name: "particleVS")!
        let ffn = lib.makeFunction(name: "particleFS")!
        let rp = MTLRenderPipelineDescriptor()
        rp.vertexFunction = vfn; rp.fragmentFunction = ffn
        rp.colorAttachments[0].pixelFormat = .bgra8Unorm
        let att = rp.colorAttachments[0]!
        att.isBlendingEnabled = true
        att.rgbBlendOperation = .add; att.alphaBlendOperation = .add
        att.sourceRGBBlendFactor = .sourceAlpha; att.sourceAlphaBlendFactor = .sourceAlpha
        att.destinationRGBBlendFactor = .oneMinusSourceAlpha; att.destinationAlphaBlendFactor = .oneMinusSourceAlpha
        psoParticles = try! device.makeRenderPipelineState(descriptor: rp)

        makeTextures(); makeBuffers()
    }

    func makeTextures() {
        func makeTex(_ fmt: MTLPixelFormat) -> MTLTexture {
            let d = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: fmt, width: N, height: N, mipmapped: false)
            d.usage = [.shaderRead, .shaderWrite]; d.storageMode = .private
            return device.makeTexture(descriptor: d)!
        }
        velA = makeTex(.rg16Float); velB = makeTex(.rg16Float)
        divTex = makeTex(.r16Float)
        pressA = makeTex(.r16Float); pressB = makeTex(.r16Float)
        dyeA = makeTex(.rgba8Unorm); dyeB = makeTex(.rgba8Unorm)

        let cmd = queue.makeCommandBuffer()!
        let enc = cmd.makeComputeCommandEncoder()!
        enc.setComputePipelineState(pClear)
        for tex in [velA!, velB!, divTex!, pressA!, pressB!, dyeA!, dyeB!] {
            enc.setTexture(tex, index: 0); dispatch2D(enc, N, N)
        }
        enc.endEncoding(); cmd.commit(); cmd.waitUntilCompleted()
    }

    func makeBuffers() {
        paramsBuf = device.makeBuffer(length: MemoryLayout<SimParams>.stride, options: .storageModeShared)
        brushBuf = device.makeBuffer(length: MemoryLayout<Brush>.stride, options: .storageModeShared)
        stepBuf = device.makeBuffer(length: MemoryLayout<UInt32>.stride, options: .storageModeShared)
        particlesBuf = device.makeBuffer(length: MemoryLayout<Particle>.stride * particleCapacity, options: .storageModeShared)
        particleRenderParams = device.makeBuffer(length: MemoryLayout<ParticleRenderParams>.stride, options: .storageModeShared)

        let p = particlesBuf.contents().bindMemory(to: Particle.self, capacity: particleCapacity)
        for i in 0..<particleCapacity { p[i] = Particle(pos: SIMD2<Float>(-1, -1), alive: 0) }
    }

    func emitParticles(at center: SIMD2<Float>, count: Int) {
        let pbuf = particlesBuf.contents().bindMemory(to: Particle.self, capacity: particleCapacity)
        for _ in 0..<count {
            let ang = Float.random(in: 0..<(2 * .pi))
            let r = Float.random(in: 0..<1).squareRoot() * emitRadius
            let pos = center + SIMD2<Float>(r * cos(ang), r * sin(ang))
            pbuf[particleHead] = Particle(pos: pos, alive: 1)
            particleHead = (particleHead + 1) % particleCapacity
            particleCount = min(particleCount + 1, particleCapacity)
        }
    }

    // MARK: - MTKViewDelegate

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        guard let cmd = queue.makeCommandBuffer(),
              let compute = cmd.makeComputeCommandEncoder() else { return }

        var P = SimParams(N: UInt32(N), dt: dt, visc: 0, invTexSize: SIMD2<Float>(1.0/Float(N), 1.0/Float(N)), dyeDissipation: 1.0)
        memcpy(paramsBuf.contents(), &P, MemoryLayout<SimParams>.stride)

        // When recording, continuously emit from center with swirl force
        let center = SIMD2<Float>(0.5, 0.5)
        if isActive {
            emitParticles(at: center, count: emitPerFrame)
            let angle = Float(frameCount) * 0.05
            let swirlForce = SIMD2<Float>(cos(angle), sin(angle)) * 0.4 * Float(N)
            var B = Brush(pos: center, force: swirlForce, radius: 0.3, strength: 1.0, enabled: 1)
            memcpy(brushBuf.contents(), &B, MemoryLayout<Brush>.stride)
        } else {
            var B = Brush(pos: center, force: .zero, radius: 0, strength: 0, enabled: 0)
            memcpy(brushBuf.contents(), &B, MemoryLayout<Brush>.stride)
        }

        stepCount &+= 1
        memcpy(stepBuf.contents(), &stepCount, MemoryLayout<UInt32>.stride)

        // Fluid sim steps
        compute.setComputePipelineState(pBrush)
        compute.setTexture(velA, index: 0); compute.setTexture(dyeA, index: 1)
        compute.setTexture(velB, index: 2); compute.setTexture(dyeB, index: 3)
        compute.setBuffer(paramsBuf, offset: 0, index: 0); compute.setBuffer(brushBuf, offset: 0, index: 1)
        dispatch2D(compute, N, N)
        swap(&velA, &velB); swap(&dyeA, &dyeB)

        compute.setComputePipelineState(pDivergence)
        compute.setTexture(velA, index: 0); compute.setTexture(divTex, index: 1)
        compute.setBuffer(paramsBuf, offset: 0, index: 0)
        dispatch2D(compute, N, N)

        compute.setComputePipelineState(pClear)
        compute.setTexture(pressA, index: 0); dispatch2D(compute, N, N)
        compute.setTexture(pressB, index: 0); dispatch2D(compute, N, N)

        for _ in 0..<jacobiIters {
            compute.setComputePipelineState(pPressureJacobi)
            compute.setTexture(pressA, index: 0); compute.setTexture(divTex, index: 1)
            compute.setTexture(pressB, index: 2); compute.setBuffer(paramsBuf, offset: 0, index: 0)
            dispatch2D(compute, N, N)
            swap(&pressA, &pressB)
        }

        compute.setComputePipelineState(pSubtractGradient)
        compute.setTexture(pressA, index: 0); compute.setTexture(velA, index: 1)
        compute.setTexture(velB, index: 2); compute.setBuffer(paramsBuf, offset: 0, index: 0)
        dispatch2D(compute, N, N)
        swap(&velA, &velB)

        compute.setComputePipelineState(pAdvect)
        compute.setTexture(velA, index: 0); compute.setTexture(velA, index: 1)
        compute.setTexture(velB, index: 2); compute.setBuffer(paramsBuf, offset: 0, index: 0)
        dispatch2D(compute, N, N)
        swap(&velA, &velB)

        compute.setComputePipelineState(pAdvectParticles)
        compute.setTexture(velA, index: 0)
        compute.setBuffer(particlesBuf, offset: 0, index: 0)
        compute.setBuffer(paramsBuf, offset: 0, index: 1)
        compute.setBuffer(stepBuf, offset: 0, index: 2)
        dispatch1D(compute, particleCapacity)

        compute.endEncoding()
        cmd.commit(); cmd.waitUntilCompleted()

        guard let renderCmd = queue.makeCommandBuffer(),
              let drawable = view.currentDrawable else { return }

        let rp = MTLRenderPassDescriptor()
        rp.colorAttachments[0].texture = drawable.texture
        rp.colorAttachments[0].loadAction = .clear
        rp.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0)
        rp.colorAttachments[0].storeAction = .store

        let renc = renderCmd.makeRenderCommandEncoder(descriptor: rp)!
        renc.setRenderPipelineState(psoParticles)

        var PR = ParticleRenderParams(
            pointSizePx: pointSizePx,
            darkness: 1.0,
            viewport: SIMD2<Float>(Float(view.drawableSize.width), Float(view.drawableSize.height))
        )
        memcpy(particleRenderParams.contents(), &PR, MemoryLayout<ParticleRenderParams>.stride)

        renc.setVertexBuffer(particlesBuf, offset: 0, index: 0)
        renc.setVertexBuffer(particleRenderParams, offset: 0, index: 1)
        renc.drawPrimitives(type: .point, vertexStart: 0, vertexCount: particleCapacity)
        renc.endEncoding()

        renderCmd.present(drawable)
        renderCmd.commit()
        frameCount += 1
    }

    func dispatch2D(_ enc: MTLComputeCommandEncoder, _ w: Int, _ h: Int) {
        let tw = 16, th = 16
        enc.dispatchThreadgroups(
            MTLSize(width: (w + tw - 1) / tw, height: (h + th - 1) / th, depth: 1),
            threadsPerThreadgroup: MTLSize(width: tw, height: th, depth: 1)
        )
    }
    func dispatch1D(_ enc: MTLComputeCommandEncoder, _ count: Int) {
        let tw = 64
        enc.dispatchThreadgroups(
            MTLSize(width: (count + tw - 1) / tw, height: 1, depth: 1),
            threadsPerThreadgroup: MTLSize(width: tw, height: 1, depth: 1)
        )
    }
}
