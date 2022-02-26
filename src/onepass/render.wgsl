struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main_vs(
    [[location(0)]] in_pos: vec2<f32>,
    [[builtin(vertex_index)]] vertex_index: u32
) -> VertexOutput {
    var pos: vec4<f32>;
    var col: vec4<f32>;
    switch (vertex_index) {
        case 0: {
            col = vec4<f32>(1.0, 0.0, 0.0, 1.0);
            pos = vec4<f32>(in_pos.x, in_pos.y, 0.0, 1.0);
        }
        case 1: {
            col = vec4<f32>(0.0, 1.0, 0.0, 1.0);
            pos = vec4<f32>(in_pos.x, in_pos.y, 0.0, 1.0);
        }
        case 2: {
            col = vec4<f32>(0.0, 0.0, 1.0, 1.0);
            pos = vec4<f32>(in_pos.x, in_pos.y, 1.0, 1.0);
        }
        case 3: {
            col = vec4<f32>(0.0, 1.0, 1.0, 1.0);
            pos = vec4<f32>(in_pos.x, in_pos.y, 1.0, 1.0);
        }
        default: {
            col = vec4<f32>(1.0, 1.0, 1.0, 1.0);
            pos = vec4<f32>(in_pos.x, in_pos.y, 1.0, 1.0);
        }
    };

    return VertexOutput(
        pos,
        col
    );
}

[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}