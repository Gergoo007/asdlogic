struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
};

struct Immediates {
	start: vec2<f32>,
	step: vec2<f32>,
	num_cols: u32,
	zoom: f32,
};

var<immediate> imm: Immediates;

@vertex
fn vs_main(
	@builtin(instance_index) iidx: u32,
	@builtin(vertex_index) vidx: u32,
) -> VertexOutput {
	var out: VertexOutput;

	const POINT_SIZE: f32 = 0.008;
	let points = array(
		vec2f(-POINT_SIZE, -POINT_SIZE),
		vec2f( POINT_SIZE, -POINT_SIZE),
		vec2f(-POINT_SIZE,  POINT_SIZE),
		vec2f(-POINT_SIZE,  POINT_SIZE),
		vec2f( POINT_SIZE, -POINT_SIZE),
		vec2f( POINT_SIZE,  POINT_SIZE),
	);

	var asd = imm.start + points[vidx] * imm.step * 10;
	asd.x += imm.step.x * f32(iidx % imm.num_cols);
	asd.y -= imm.step.y * f32(iidx / imm.num_cols);
	out.clip_position = vec4<f32>(asd, 1.0, 1.0);

	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(0.1, 0.1, 0.1, 1.0);
}
