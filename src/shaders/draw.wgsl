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

struct VOut {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main_vs(
    line: Line,
    v: Vertex,
    [[builtin(vertex_index)]] vertex_index: u32,
) -> VOut {
    var offset = vec2<f32>(
        0.01 * v.pos.x * cos(line.angle) - line.len * v.pos.y * sin(line.angle),
        0.01 * v.pos.x * sin(line.angle) + line.len * v.pos.y * cos(line.angle),
    );

    var col: vec4<f32>;
    switch (vertex_index) {
        case 0: {col = vec4<f32>(1.0, 0.0, 0.0, 1.0);}
        case 1: {col = vec4<f32>(0.0, 1.0, 0.0, 1.0);}
        case 2: {col = vec4<f32>(0.0, 0.0, 1.0, 1.0);}
        case 3: {col = vec4<f32>(0.0, 1.0, 1.0, 1.0);}
        default: {col = vec4<f32>(1.0, 1.0, 1.0, 1.0);}
    };

    return VOut(
        vec4<f32>(line.start + offset, 0.0, 1.0),
        col
    );
}

[[stage(fragment)]]
fn main_fs(in: VOut) -> [[location(0)]] vec4<f32> {
    return in.color;
    // return vec4<f32>(0.2, 0.1, 0.5, 1.0);
}
