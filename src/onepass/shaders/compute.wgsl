// TODO: somehow get rid of these?
struct Samples {
    samples: [[stride(8)]] array<vec2<i32>>;
};
struct Vertices {
    vertices: [[stride(8)]] array<vec2<f32>>;
};

[[group(0), binding(0)]] var<storage, read> samples : Samples;
[[group(0), binding(1)]] var<storage, read_write> vertices : Vertices;

// Conversion for i16 to normalized f32 between -1 and 1
// ie. Reciprocal of i16::MAX
//     Equal to 1.0 / 32767.0
let i16_convert: f32 = 0.00003051850947599719;

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let index = global_invocation_id.x;
  vertices.vertices[index] = vec2<f32>(samples.samples[index]) * i16_convert;
}
