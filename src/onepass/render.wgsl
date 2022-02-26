struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main_vs(
    [[location(0)]] inst_pos: vec2<f32>,
    [[builtin(vertex_index)]] vertex_index: u32
) -> VertexOutput {
    //  v | bin(v) | tx | ty
    // ---|--------|----|----
    //  0 |   00   |  0 |  0
    //  1 |   01   |  0 |  1
    //  2 |   10   |  1 |  0
    //  3 |   11   |  1 |  1

    var one = u32(1);
    var tx = (vertex_index >> one) & one;
    var ty = (vertex_index & one);

    var x = -1.0 + 2.0 * f32(tx);
    var y = f32(ty);

    var r = f32(~(tx | ty));
    var g = y;
    var b = x;
    return VertexOutput(
        vec4<f32>(x, y, 0.0, 1.0),
        vec4<f32>(r, g, b,   1.0)
    );
}

[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}