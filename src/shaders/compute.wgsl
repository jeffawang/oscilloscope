struct Params {
  blah: f32;
};

struct Particle {
    blah: f32;
};

[[group(0), binding(0)]] var<uniform> params : Params;
[[group(0), binding(1)]] var<storage, read> particlesSrc : Particle;
[[group(0), binding(2)]] var<storage, read_write> particlesDst : Particle;

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  
}
