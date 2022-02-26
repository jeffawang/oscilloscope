struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main_vs(
    [[location(0)]] curr: vec2<f32>,
    [[location(1)]] next: vec2<f32>,
    [[builtin(vertex_index)]] v_id: u32
) -> VertexOutput {
    //  v | bin(v) | tx | ty
    // ---|--------|----|----
    //  0 |   00   |  0 |  0
    //  1 |   01   |  0 |  1
    //  2 |   10   |  1 |  0
    //  3 |   11   |  1 |  1
    //  4 |  100   |  

    var one = u32(1);
    var x = f32(v_id % 2u);
    x = -1.0 + 2.0 * x;
    var y = f32(v_id / 2u);

    var color = vec3<f32>(
        x,
        y,
        1.0
    );

    var pos = curr + vec2<f32>(x,y);

    var boink = next - curr;
    var norm = normalize(vec2<f32>(-boink.y, boink.x)) * 0.01;

    pos = (1.0-y) * curr + y * next;
    pos = pos + norm * x;
    // pos = vec2<f32>(x,y) - vec2<f32>(0.5,0.5);

    return VertexOutput(
        vec4<f32>(pos, 0.0, 1.0),
        vec4<f32>(color,   1.0)
    );
}

[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}