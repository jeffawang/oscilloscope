// Vertex and Fragment shader to draw lines.

// From compute shader
struct Line {
    [[location(0)]] start: vec2<f32>;
    [[location(1)]] len: f32;
    [[location(2)]] angle: f32;
};

// From vertex buffer (constant, since we are instanced)
struct Vertex {
    [[location(3)]] pos: vec2<f32>;
};


[[stage(vertex)]]
fn main_vs(
    line: Line,
    v: Vertex,
    [[builtin(vertex_index)]] vertex_index: u32,
) -> [[builtin(position)]] vec4<f32> {
    var offset = vec2<f32>(v.pos.x * 0.1, v.pos.y * line.len);
    return vec4<f32>(line.start * 0.5 + offset, 0.0, 1.0);
}

[[stage(fragment)]]
fn main_fs() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
