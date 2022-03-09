struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

struct VertexInput {
    [[location(0)]] curr: vec2<f32>;
    [[location(1)]] next: vec2<f32>;
    [[builtin(vertex_index)]] v_id: u32;
};

struct Uniforms {
    frame: u32;
    time: f32;
    line_thickness: f32;
    count: f32;
};

[[group(0), binding(0)]] var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn main_vs(in: VertexInput) -> VertexOutput {
    var x = f32(in.v_id % 2u);
    x = -1.0 + 2.0 * x;
    var y = f32(in.v_id / 2u);

    var between = in.next - in.curr;
    var w = 1.0 - 10.0*length(between);
    w = w * step(0.6, w);

    var color = vec3<f32>( 1.0, 1.0, 0.0 );
    var pos = in.curr + vec2<f32>(x,y);

    var norm = normalize(vec2<f32>(-between.y, between.x)) * uniforms.line_thickness * w;

    pos = (1.0-y) * in.curr + y * in.next;
    pos = pos + norm * x;

    var z = mix(0.0, f32(in.v_id), uniforms.count);
    color = mix(vec3<f32>(0.0, 0.0, 0.0), color, w);

    return VertexOutput(
        vec4<f32>(pos, 0.0, 1.0),
        vec4<f32>(color, 1.0),
    );
}

[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}