struct Params {
  blah: f32;
};

// From compute shader
struct Line {
    [[location(0)]] start: vec2<f32>;
    [[location(1)]] len: f32;
    [[location(2)]] angle: f32;
};

struct Lines {
    lines: [[stride(16)]] array<Line>;
};

[[group(0), binding(0)]] var<uniform> params : Params;
[[group(0), binding(1)]] var<storage, read> linesSrc : Lines;
[[group(0), binding(2)]] var<storage, read_write> linesDst : Lines;

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let index = global_invocation_id.x;
//   var before : vec2<f32> = particlesSrc.particles[index].pos;
  var before: Line = linesSrc.lines[index];

  linesDst.lines[index] = Line(
      before.start,// + vec2<f32>(0.0001, 0.0001),
      before.len,
      before.angle,
  );
}
