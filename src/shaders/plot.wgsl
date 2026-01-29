// Plot shader - renders markers as instanced quads with circular masking

struct Uniforms {
    viewport_size: vec2<f32>,
    x_range: vec2<f32>,
    y_range: vec2<f32>,
    padding: vec2<f32>,
    marker_radius: f32,
    line_width: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// Per-instance point data
struct PointInput {
    @location(0) position: vec2<f32>,  // Data coordinates
    @location(1) color: vec4<f32>,
}

// Vertex shader output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,  // Position within quad for circle masking
}

// Quad vertices for instanced rendering (2 triangles)
const QUAD_VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>( 1.0,  1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 1.0,  1.0),
    vec2<f32>(-1.0,  1.0),
);

// Transform data coordinates to normalized device coordinates
fn data_to_ndc(data_pos: vec2<f32>) -> vec2<f32> {
    // Calculate plot area (viewport minus padding)
    let plot_width = uniforms.viewport_size.x - 2.0 * uniforms.padding.x;
    let plot_height = uniforms.viewport_size.y - 2.0 * uniforms.padding.y;
    
    // Normalize data position to 0-1 range
    let x_norm = (data_pos.x - uniforms.x_range.x) / (uniforms.x_range.y - uniforms.x_range.x);
    let y_norm = (data_pos.y - uniforms.y_range.x) / (uniforms.y_range.y - uniforms.y_range.x);
    
    // Convert to screen pixels (with padding offset)
    let screen_x = uniforms.padding.x + x_norm * plot_width;
    let screen_y = uniforms.padding.y + (1.0 - y_norm) * plot_height;  // Flip Y
    
    // Convert to NDC (-1 to 1)
    let ndc_x = (screen_x / uniforms.viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (screen_y / uniforms.viewport_size.y) * 2.0;  // Flip Y for NDC
    
    return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_marker(
    @builtin(vertex_index) vertex_index: u32,
    point: PointInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Get quad vertex position (-1 to 1)
    let local_pos = QUAD_VERTICES[vertex_index];
    
    // Transform point to NDC
    let center_ndc = data_to_ndc(point.position);
    
    // Calculate marker size in NDC
    let marker_size_ndc = vec2<f32>(
        (uniforms.marker_radius * 2.0) / uniforms.viewport_size.x,
        (uniforms.marker_radius * 2.0) / uniforms.viewport_size.y
    );
    
    // Offset quad vertices from center
    let final_pos = center_ndc + local_pos * marker_size_ndc;
    
    out.clip_position = vec4<f32>(final_pos, 0.0, 1.0);
    out.color = point.color;
    out.local_pos = local_pos;
    
    return out;
}

@fragment
fn fs_marker(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate distance from center of quad
    let dist = length(in.local_pos);
    
    // Discard pixels outside the circle
    if dist > 1.0 {
        discard;
    }
    
    // Anti-aliasing: smooth edge
    let alpha = 1.0 - smoothstep(0.8, 1.0, dist);
    
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

// Line vertex input (pre-computed screen positions)
struct LineVertexInput {
    @location(0) position: vec2<f32>,  // Already in screen coordinates
    @location(1) color: vec4<f32>,
}

struct LineVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_line(vertex: LineVertexInput) -> LineVertexOutput {
    var out: LineVertexOutput;
    
    // Convert screen position to NDC
    let ndc_x = (vertex.position.x / uniforms.viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (vertex.position.y / uniforms.viewport_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = vertex.color;
    
    return out;
}

@fragment
fn fs_line(in: LineVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
