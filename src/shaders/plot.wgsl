// Plot shader - renders markers as instanced quads with various shapes, and lines

// Marker shape constants
const SHAPE_CIRCLE: u32 = 0u;
const SHAPE_SQUARE: u32 = 1u;
const SHAPE_DIAMOND: u32 = 2u;
const SHAPE_TRIANGLE_UP: u32 = 3u;
const SHAPE_TRIANGLE_DOWN: u32 = 4u;
const SHAPE_CROSS: u32 = 5u;
const SHAPE_PLUS: u32 = 6u;
const SHAPE_NONE: u32 = 7u;

// Line pattern constants
const PATTERN_SOLID: u32 = 0u;
const PATTERN_DASHED: u32 = 1u;
const PATTERN_DOTTED: u32 = 2u;
const PATTERN_DASHDOT: u32 = 3u;
const PATTERN_NONE: u32 = 4u;

struct Uniforms {
    viewport_size: vec2<f32>,
    x_range: vec2<f32>,
    y_range: vec2<f32>,
    padding: vec2<f32>,
    marker_radius: f32,
    line_width: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// Per-instance point data for markers
struct PointInput {
    @location(0) position: vec2<f32>,  // Data coordinates
    @location(1) color: vec4<f32>,
    @location(2) shape: u32,           // Marker shape
    @location(3) _padding: u32,
}

// Vertex shader output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,  // Position within quad for shape rendering
    @location(2) shape: u32,            // Marker shape
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

// Signed distance functions for various marker shapes
fn sdf_circle(p: vec2<f32>) -> f32 {
    return length(p) - 1.0;
}

fn sdf_square(p: vec2<f32>) -> f32 {
    let d = abs(p) - vec2<f32>(0.7);
    return max(d.x, d.y);
}

fn sdf_diamond(p: vec2<f32>) -> f32 {
    return (abs(p.x) + abs(p.y)) - 1.0;
}

fn sdf_triangle_up(p: vec2<f32>) -> f32 {
    // Equilateral triangle pointing up
    let h = 0.866; // sqrt(3)/2
    let d0 = abs(p.x) - 0.7;
    let d1 = p.y + 0.5;
    let d2 = (abs(p.x) * 0.866 - p.y) * 0.5 - 0.5;
    return max(max(d0, d1), d2);
}

fn sdf_triangle_down(p: vec2<f32>) -> f32 {
    // Triangle pointing down
    let h = 0.866;
    let d0 = abs(p.x) - 0.7;
    let d1 = -p.y - 0.5;
    let d2 = (abs(p.x) * 0.866 + p.y) * 0.5 - 0.5;
    return max(max(d0, d1), d2);
}

fn sdf_cross(p: vec2<f32>) -> f32 {
    // Cross/X shape
    let thickness = 0.2;
    let d1 = abs(abs(p.x) - abs(p.y)) - thickness;
    let d2 = max(abs(p.x), abs(p.y)) - 1.0;
    return max(d1, d2);
}

fn sdf_plus(p: vec2<f32>) -> f32 {
    // Plus/+ shape
    let thickness = 0.2;
    let d1 = max(abs(p.x), abs(p.y)) - thickness;
    let d2 = min(abs(p.x), abs(p.y)) - thickness;
    let d3 = max(abs(p.x), abs(p.y)) - 1.0;
    return max(min(d1, d2), d3);
}

fn evaluate_sdf(p: vec2<f32>, shape: u32) -> f32 {
    switch shape {
        case SHAPE_CIRCLE: { return sdf_circle(p); }
        case SHAPE_SQUARE: { return sdf_square(p); }
        case SHAPE_DIAMOND: { return sdf_diamond(p); }
        case SHAPE_TRIANGLE_UP: { return sdf_triangle_up(p); }
        case SHAPE_TRIANGLE_DOWN: { return sdf_triangle_down(p); }
        case SHAPE_CROSS: { return sdf_cross(p); }
        case SHAPE_PLUS: { return sdf_plus(p); }
        default: { return sdf_circle(p); }
    }
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
    out.shape = point.shape;
    
    return out;
}

@fragment
fn fs_marker(in: VertexOutput) -> @location(0) vec4<f32> {
    // Skip rendering if shape is NONE
    if in.shape == SHAPE_NONE {
        discard;
    }
    
    // Evaluate signed distance field for this shape
    let sdf = evaluate_sdf(in.local_pos, in.shape);
    
    // Discard pixels outside the shape
    if sdf > 0.1 {
        discard;
    }
    
    // Anti-aliasing: smooth edge
    let alpha = 1.0 - smoothstep(-0.1, 0.1, sdf);
    
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

// Line vertex input (pre-computed screen positions)
struct LineVertexInput {
    @location(0) position: vec2<f32>,  // Already in screen coordinates
    @location(1) color: vec4<f32>,
    @location(2) edge_distance: f32,   // Signed normalised distance from line centre
}

struct LineVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) edge_distance: f32,
}

@vertex
fn vs_line(vertex: LineVertexInput) -> LineVertexOutput {
    var out: LineVertexOutput;
    
    // Convert screen position to NDC
    let ndc_x = (vertex.position.x / uniforms.viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (vertex.position.y / uniforms.viewport_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = vertex.color;
    out.edge_distance = vertex.edge_distance;
    
    return out;
}

@fragment
fn fs_line(in: LineVertexOutput) -> @location(0) vec4<f32> {
    // Anti-aliased edges: abs(edge_distance) is 0 at centre, 1.0 at original
    // line edge, >1.0 in the AA extension fringe.
    let d = abs(in.edge_distance);
    let alpha = 1.0 - smoothstep(0.8, 1.0, d);
    if alpha < 0.001 {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}


