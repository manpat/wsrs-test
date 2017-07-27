precision mediump float;

varying vec4 vcolor;

void main() {
	vec4 col = vcolor;
	col.rgb = pow(col.rgb, vec3(1.0/2.2));

	gl_FragColor = col;
}
