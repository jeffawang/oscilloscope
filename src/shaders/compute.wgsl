struct Params {
  blah: f32;
};

struct Particle {
    blah: f32;
};

struct SimParams {
  deltaT : f32;
  rule1Distance : f32;
  rule2Distance : f32;
  rule3Distance : f32;
  rule1Scale : f32;
  rule2Scale : f32;
  rule3Scale : f32;
};

struct Particles {
  particles : [[stride(16)]] array<Params>;
};

[[group(0), binding(0)]] var<uniform> params : Params;
[[group(0), binding(1)]] var<storage, read> particlesSrc : Particle;
[[group(0), binding(2)]] var<storage, read_write> particlesDst : Particle;

// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp
[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  
}
