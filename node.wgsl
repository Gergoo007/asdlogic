struct VertexOutput {
	@builtin(position) pos: vec4<f32>,
	@location(0) uv: vec2<f32>,
	@location(1) filled: u32,
	@location(2) color: vec3<f32>,
}

struct NodeInput {
	@location(0) k: vec2<f32>,
	@location(1) filled: u32,
	@location(2) color: u32,
}

struct Immediates {
	pan: vec2<f32>,
	wsize: vec2<f32>,
	zoom: f32,
};
var<immediate> imm: Immediates;

const GS: f32 = 16.0;

fn transform(in: vec2<f32>) -> vec2<f32> {
	return ((in * GS + imm.pan) / imm.wsize * 2.0 - 1.0) * imm.zoom;
}

@vertex
fn vs_main(
	@builtin(vertex_index) vidx: u32,
	in: NodeInput,
) -> VertexOutput {
	var out: VertexOutput;

	let rad = 0.5;

	let vertices = array<vec4<f32>, 4>(
		vec4(transform((in.k + imm.wsize / 2.0 / 16.0) + vec2(-rad, -rad)), vec2(-1.0, -1.0)),
		vec4(transform((in.k + imm.wsize / 2.0 / 16.0) + vec2( rad, -rad)), vec2( 1.0, -1.0)),
		vec4(transform((in.k + imm.wsize / 2.0 / 16.0) + vec2(-rad,  rad)), vec2(-1.0,  1.0)),
		vec4(transform((in.k + imm.wsize / 2.0 / 16.0) + vec2( rad,  rad)), vec2( 1.0,  1.0)),
	);

	let vert = vec2<f32>(vertices[vidx].x, -vertices[vidx].y);

	out.pos = vec4<f32>(vert, 1.0, 1.0);
	out.uv = vertices[vidx].zw;
	out.filled = in.filled;
	out.color = pow(
		vec3(
			f32((in.color >> 0) & 0xff) / 255.0,
			f32((in.color >> 8) & 0xff) / 255.0,
			f32((in.color >> 16) & 0xff) / 255.0,
		),
		vec3(2.2)
	);

	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	if in.filled == 0 {
		let d = length(in.uv);
		let rad = 0.65;
		let th = 0.268;

		let dist = abs(d - rad) - th / 2.0;
		let aa = fwidth(dist) / 3.0;

		return vec4(in.color, smoothstep(aa, -aa, dist));
	} else {
		let d = length(in.uv);
		let rad = 0.5;

		let dist = d - rad;
		let aa = fwidth(dist) / 3.0;

		let alpha = smoothstep(aa, -aa, dist);

		return vec4<f32>(in.color, alpha);
	}
}
