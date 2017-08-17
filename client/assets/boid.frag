precision mediump float;

uniform vec3 color;

vec3 gamma_correct(vec3 c) {
	return pow(c, vec3(1.0/2.2));
}

void main() {
	vec2 uv = abs(vec2(0.5) - gl_PointCoord);

	float manhattan = 2.0 * (uv.x + uv.y);

	if(manhattan < 1.0)
		gl_FragColor = vec4(gamma_correct(color), 1.0);
	else
		discard;
}
