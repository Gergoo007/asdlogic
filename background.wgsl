struct VertexOutput {
	@builtin(position) clip_position: vec4f,
};

struct Immediates {
	start: vec2f,
	wsize: vec2f,
	num_cols: u32,
	zoom: f32,
	nth: u32,
};

var<immediate> imm: Immediates;

@vertex
fn vs_main(
	@builtin(instance_index) iidx: u32,
	@builtin(vertex_index) vidx: u32,
) -> VertexOutput {
	var out: VertexOutput;

	let step = 16.0 * f32(imm.nth) * imm.zoom * 2.0 / imm.wsize;

	let px_sz_x = 2.0 / imm.wsize.x;
	let px_sz_y = 2.0 / imm.wsize.y;

	// A pontok mérete soha ne legyen kisebb mint egy pixel mert goofy lesz
	let POINT_SIZE_X: f32 = max(px_sz_x * imm.zoom, px_sz_x);
	let POINT_SIZE_Y: f32 = max(px_sz_y * imm.zoom, px_sz_y);
	let points = array(
		vec2f(-POINT_SIZE_X, -POINT_SIZE_Y),
		vec2f( POINT_SIZE_X, -POINT_SIZE_Y),
		vec2f(-POINT_SIZE_X,  POINT_SIZE_Y),
		vec2f( POINT_SIZE_X,  POINT_SIZE_Y),
	);

	var asd = imm.start + points[vidx];
	asd.x += step.x * f32(iidx % imm.num_cols);
	asd.y -= step.y * f32(iidx / imm.num_cols);
	out.clip_position = vec4f(asd, 1.0, 1.0);

	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	return vec4f(0.1, 0.1, 0.1, 1.0);
}
