#import bevy_ui::ui_vertex_output::UiVertexOutput

struct StrokeUniform {
    color: vec4<f32>,
    metrics: vec4<f32>,
    count: vec4<u32>,
    points: array<vec4<f32>, 64>,
};

@group(1) @binding(0) var<uniform> stroke: StrokeUniform;

fn segment_distance(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let t = clamp(dot(p - a, ab) / max(dot(ab, ab), 0.000001), 0.0, 1.0);
    return length(p - a - t * ab);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Coordinates and thickness stay in logical pixels. Derivatives account
    // for physical UI scale automatically, without a second scale uniform.
    let p = in.uv * stroke.metrics.xy;
    var distance = 1e20;
    for (var i = 1u; i < stroke.count.x; i += 1u) {
        distance = min(distance, segment_distance(p, stroke.points[i - 1u].xy, stroke.points[i].xy));
    }
    distance -= stroke.metrics.z;
    if stroke.metrics.w > 0.0 {
        distance = min(distance, length(p - stroke.points[stroke.count.x - 1u].xy) - stroke.metrics.w);
    }
    let aa = max(fwidth(distance), 0.0001);
    let coverage = 1.0 - smoothstep(-0.5 * aa, 0.5 * aa, distance);
    return vec4(stroke.color.rgb, stroke.color.a * coverage);
}
