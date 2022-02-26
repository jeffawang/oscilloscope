struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main_vs(
    [[location(0)]] pos: vec2<f32>
) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(pos.x, pos.y, 0.0, 1.0),
        vec4<f32>(1.0, 0.0, 0.0, 1.0)
    );
}

[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}