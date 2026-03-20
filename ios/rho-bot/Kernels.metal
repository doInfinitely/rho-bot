#include <metal_stdlib>
using namespace metal;

// =================== Shared structs ===================
struct SimParams {
    uint   N;
    float  dt;
    float  visc;
    float2 invTexSize;
    float  dyeDissipation;
};

struct Brush {
    float2 pos;
    float2 force;
    float  radius;
    float  strength;
    uint   enabled;
};

struct Particle {
    float2 pos;
    float  alive;
};

constant sampler linClamped(address::clamp_to_edge, filter::linear);

inline float2 uvFromIJ(uint2 ij, constant SimParams& P) {
    return saturate((float2(ij) + 0.5f) * P.invTexSize);
}
inline float2 sampleVel(texture2d<half, access::sample> vel, float2 uv) {
    return float2(vel.sample(linClamped, uv).rg);
}
inline float sampleScalar(texture2d<half, access::sample> t, float2 uv) {
    return float(t.sample(linClamped, uv).r);
}

// =================== Clear ===================
kernel void kClear(
    texture2d<half, access::write> tex [[texture(0)]],
    uint2 gid [[thread_position_in_grid]])
{
    tex.write(half4(0, 0, 0, 0), gid);
}

// =================== Brush ===================
kernel void kBrush(
    texture2d<half, access::sample> velIn   [[texture(0)]],
    texture2d<half, access::sample> dyeIn   [[texture(1)]],
    texture2d<half, access::write>  velOut  [[texture(2)]],
    texture2d<half, access::write>  dyeOut  [[texture(3)]],
    constant SimParams& P                    [[buffer(0)]],
    constant Brush& B                        [[buffer(1)]],
    uint2 gid                                [[thread_position_in_grid]])
{
    const float2 uv = uvFromIJ(gid, P);
    half2 v = half2(sampleVel(velIn, uv));
    half4 dye = dyeIn.sample(linClamped, uv);

    if (B.enabled) {
        const float2 d  = uv - B.pos;
        const float  r  = B.radius;
        if (r > 0.0f) {
            const float d2 = dot(d,d);
            if (d2 <= r*r) {
                float w = 1.0f - (d2 / (r*r));
                w *= w;
                v += half2(B.force * (B.strength * w));
                half nd = clamp(dye.r + half(B.strength * 0.6f * w), half(0), half(1));
                dye = half4(nd, nd, nd, half(1));
            }
        }
    }

    if (isnan(v.x) || isnan(v.y) || isinf(v.x) || isinf(v.y)) {
        v = half2(0, 0);
    }
    velOut.write(half4(v,0,1), gid);
    dyeOut.write(dye, gid);
}

// =================== Advect ===================
kernel void kAdvect(
    texture2d<half, access::sample>  src     [[texture(0)]],
    texture2d<half, access::sample>  velTex  [[texture(1)]],
    texture2d<half, access::write>   dst     [[texture(2)]],
    constant SimParams& P                     [[buffer(0)]],
    uint2 gid                                 [[thread_position_in_grid]])
{
    float2 uv   = uvFromIJ(gid, P);
    float2 v    = sampleVel(velTex, uv);
    float2 prev = uv - v * (P.dt * P.invTexSize);
    half4 s = src.sample(linClamped, prev);
    dst.write(s, gid);
}

// =================== Jacobi ===================
kernel void kJacobi(
    texture2d<half, access::sample>  xTex    [[texture(0)]],
    texture2d<half, access::sample>  bTex    [[texture(1)]],
    texture2d<half, access::write>   xOut    [[texture(2)]],
    constant SimParams& P                     [[buffer(0)]],
    uint2 gid                                 [[thread_position_in_grid]])
{
    const float2 uv = uvFromIJ(gid, P);
    const float2 du = float2(P.invTexSize.x, 0);
    const float2 dv = float2(0, P.invTexSize.y);
    float4 l = float4(xTex.sample(linClamped, uv - du));
    float4 r = float4(xTex.sample(linClamped, uv + du));
    float4 d = float4(xTex.sample(linClamped, uv - dv));
    float4 u = float4(xTex.sample(linClamped, uv + dv));
    float4 b = float4(bTex.sample(linClamped, uv));
    float a = P.visc * P.dt * float(P.N * P.N);
    float c = 1.0f + 4.0f * a;
    float4 x = (b + a * (l + r + d + u)) / c;
    xOut.write(half4(x), gid);
}

// =================== Divergence ===================
kernel void kDivergence(
    texture2d<half, access::sample>  velTex  [[texture(0)]],
    texture2d<half, access::write>   divOut  [[texture(1)]],
    constant SimParams& P                     [[buffer(0)]],
    uint2 gid                                 [[thread_position_in_grid]])
{
    const float2 uv = uvFromIJ(gid, P);
    const float2 du = float2(P.invTexSize.x, 0);
    const float2 dv = float2(0, P.invTexSize.y);
    const float2 vl = sampleVel(velTex, uv - du);
    const float2 vr = sampleVel(velTex, uv + du);
    const float2 vd = sampleVel(velTex, uv - dv);
    const float2 vu = sampleVel(velTex, uv + dv);
    const float div = 0.5f * ((vr.x - vl.x) + (vu.y - vd.y));
    divOut.write(half4(div,0,0,1), gid);
}

// =================== Pressure Jacobi ===================
kernel void kPressureJacobi(
    texture2d<half, access::sample>  pTex    [[texture(0)]],
    texture2d<half, access::sample>  divTex  [[texture(1)]],
    texture2d<half, access::write>   pOut    [[texture(2)]],
    constant SimParams& P                     [[buffer(0)]],
    uint2 gid                                 [[thread_position_in_grid]])
{
    const float2 uv = uvFromIJ(gid, P);
    const float2 du = float2(P.invTexSize.x, 0);
    const float2 dv = float2(0, P.invTexSize.y);
    const float pl = sampleScalar(pTex, uv - du);
    const float pr = sampleScalar(pTex, uv + du);
    const float pd = sampleScalar(pTex, uv - dv);
    const float pu = sampleScalar(pTex, uv + dv);
    const float b  = sampleScalar(divTex, uv);
    const float p = (pl + pr + pd + pu - b) * 0.25f;
    pOut.write(half4(p,0,0,1), gid);
}

// =================== Subtract Gradient ===================
kernel void kSubtractGradient(
    texture2d<half, access::sample>  pTex    [[texture(0)]],
    texture2d<half, access::sample>  velIn   [[texture(1)]],
    texture2d<half, access::write>   velOut  [[texture(2)]],
    constant SimParams& P                     [[buffer(0)]],
    uint2 gid                                 [[thread_position_in_grid]])
{
    const float2 uv = uvFromIJ(gid, P);
    const float2 du = float2(P.invTexSize.x, 0);
    const float2 dv = float2(0, P.invTexSize.y);
    const float pl = sampleScalar(pTex, uv - du);
    const float pr = sampleScalar(pTex, uv + du);
    const float pd = sampleScalar(pTex, uv - dv);
    const float pu = sampleScalar(pTex, uv + dv);
    const float2 grad = 0.5f * float2(pr - pl, pu - pd);
    float2 v = sampleVel(velIn, uv);
    v -= grad;
    if (isnan(v.x) || isnan(v.y) || isinf(v.x) || isinf(v.y)) { v = float2(0, 0); }
    velOut.write(half4(half2(v),0,1), gid);
}

// =================== Particles ===================
inline float rand01(uint id, uint step) {
    uint x = id * 1664525u + 1013904223u + step * 374761393u;
    x ^= x >> 17; x *= 0x85ebca6bu; x ^= x >> 13; x *= 0xc2b2ae35u; x ^= x >> 16;
    return float(x) / float(0xffffffffu);
}

kernel void kAdvectParticles(
    texture2d<half, access::sample> velTex    [[texture(0)]],
    device Particle*                particles [[buffer(0)]],
    constant SimParams&             P         [[buffer(1)]],
    constant uint&                  stepCount [[buffer(2)]],
    uint gid                                    [[thread_position_in_grid]])
{
    device Particle& p = particles[gid];
    if (p.alive <= 0.0f) return;

    p.alive -= 0.003f;
    if (p.alive <= 0.0f) { p.alive = 0.0f; return; }

    float2 vel = sampleVel(velTex, p.pos);
    if (isnan(vel.x) || isnan(vel.y) || isinf(vel.x) || isinf(vel.y)) { vel = float2(0, 0); }
    vel = clamp(vel, float2(-50.0), float2(50.0));

    float2 displacement = P.dt * vel * P.invTexSize * 5.0f;
    p.pos += displacement;

    if (p.pos.x < 0.0f) p.pos.x += 1.0f;
    if (p.pos.x > 1.0f) p.pos.x -= 1.0f;
    if (p.pos.y < 0.0f) p.pos.y += 1.0f;
    if (p.pos.y > 1.0f) p.pos.y -= 1.0f;

    if (isnan(p.pos.x) || isnan(p.pos.y)) { p.alive = 0.0f; }
}

// =================== Particle render ===================
struct VSOut {
    float4 pos [[position]];
    float  size [[point_size]];
    float  alpha;
};

struct ParticleRenderParams {
    float pointSizePx;
    float darkness;
    float2 viewport;
};

vertex VSOut particleVS(
    const device Particle* particles   [[buffer(0)]],
    constant ParticleRenderParams& RP  [[buffer(1)]],
    uint vid                           [[vertex_id]])
{
    Particle p = particles[vid];
    VSOut o;
    if (p.alive <= 0.0f) {
        o.pos = float4(-2.0, -2.0, 0.0, 1.0);
        o.size = 0.0;
        o.alpha = 0.0;
        return o;
    }
    float2 clip = float2(p.pos.x * 2.0 - 1.0, 1.0 - p.pos.y * 2.0);
    o.pos = float4(clip, 0.0, 1.0);
    o.size = RP.pointSizePx;
    o.alpha = clamp(p.alive, 0.0, 1.0) * RP.darkness;
    return o;
}

fragment half4 particleFS(VSOut in [[stage_in]],
                          float2 pointCoord [[point_coord]])
{
    float2 d = pointCoord * 2.0 - 1.0;
    float r2 = dot(d,d);
    if (r2 > 1.0) discard_fragment();
    const float sigma = 0.45;
    float g = exp(-r2 / (2.0 * sigma * sigma));
    float a = clamp(g, 0.0, 1.0) * in.alpha;
    // Blue-tinted particles for the mic button
    return half4(0.2, 0.5, 1.0, half(a));
}
