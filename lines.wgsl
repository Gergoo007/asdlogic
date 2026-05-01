struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) color: vec4<f32>,
	@location(1) dist: vec2<f32>,
	@location(2) thickness: vec2<f32>,
}

struct LineInput {
	@location(0) s: vec2<f32>,
	@location(1) e: vec2<f32>,
	@location(2) c: u32,
}

struct Immediates {
	pan: vec2<f32>,
	wsize: vec2<f32>,
	zoom: f32,
};
var<immediate> imm: Immediates;

const GS: f32 = 16.0;

fn transform(in: vec2<f32>) -> vec2<f32> {
	return (((in + vec2<f32>(0.0, 0.5)) * GS + imm.pan) / imm.wsize * 2.0 - 1.0) * imm.zoom;
}

@vertex
fn vs_main(
	@builtin(vertex_index) vidx: u32,
	in: LineInput,
) -> VertexOutput {
	var out: VertexOutput;

	let s = transform(in.s + vec2<f32>(40.0, 22.0)); // ?????????????????????????????????????????????????????????????????????????????????
	let e = transform(in.e + vec2<f32>(40.0, 22.0)); // ?????????????????????????????????????????????????????????????????????????????????

	let v = e - s;
	let n = normalize(vec2<f32>(v.y, -v.x));

	let px = 2.0 / imm.wsize;
	// let thickness = 2.0 * px * imm.zoom;
	let thickness = max(px * 1.0, px * 1.5 * imm.zoom);

	let dists = array<f32, 6>(1.0, -1.0, 1.0, 1.0, -1.0, -1.0);
	let d_sign = dists[vidx];

	let vertices = array<vec2<f32>, 6>(
		s + n * thickness,
		s - n * thickness,
		e + n * thickness,
		e + n * thickness,
		s - n * thickness,
		e - n * thickness,
	);

	let vert = vec2<f32>(vertices[vidx].x, -vertices[vidx].y);

	out.clip_position = vec4<f32>(vert, 1.0, 1.0);
	out.dist = d_sign * thickness;
	out.thickness = thickness;

	out.color = vec4<f32>(
		pow(vec3<f32>(
			f32((in.c >> 0) & 0xff) / 255.0,
			f32((in.c >> 8) & 0xff) / 255.0,
			f32((in.c >> 16) & 0xff) / 255.0,
		), vec3<f32>(2.2)),
		f32((in.c >> 24) & 0xff) / 255.0,
	);

	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let pixel_dist = abs(in.dist) / (2.0 / imm.wsize);
	let thickness = in.thickness / (2.0 / imm.wsize);

	let edge0 = thickness;
	let edge1 = thickness - 1.0;
	let alpha = clamp(length(edge0 - pixel_dist), 0.0, 1.0);

	return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
