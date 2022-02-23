[[stage(vertex)]]
fn main_vs(
    [[location(0)]] start: vec2<f32>,
    [[location(1)]] len: f32,
    [[location(2)]] angle: f32,
    [[location(3)]] vpos: vec2<f32>,
    [[builtin(vertex_index)]] vertex_index: u32,
) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(vpos, 0.0, 1.0);
}

[[stage(fragment)]]
fn main_fs() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
