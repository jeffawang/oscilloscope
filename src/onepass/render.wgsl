struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

struct VertexInput {
    [[location(0)]] curr: vec2<f32>;
    [[location(1)]] next: vec2<f32>;
    [[builtin(vertex_index)]] v_id: u32;
};

[[stage(vertex)]]
fn main_vs(in: VertexInput) -> VertexOutput {
    var x = f32(in.v_id % 2u);
    x = -1.0 + 2.0 * x;
    var y = f32(in.v_id / 2u);

    var color = vec3<f32>( x, y, 1.0 );
    var pos = in.curr + vec2<f32>(x,y);

    var between = in.next - in.curr;
    var norm = normalize(vec2<f32>(-between.y, between.x)) * 0.01;

    pos = (1.0-y) * in.curr + y * in.next;
    pos = pos + norm * x;

    return VertexOutput(
        vec4<f32>(pos, 0.0, 1.0),
        vec4<f32>(color, 1.0),
    );
}

[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}