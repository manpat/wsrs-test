precision mediump float;

varying vec4 vcolor;

uniform mat4 normal_xform;

void main() {
	vec4 col = vcolor;

	// TODO: Move to post effect
	vec3 primcol = vec3(1.0, 0.975, 0.794);
	vec3 seccol = vec3(0.303, 0.222, 0.147);

	vec3 primary = normalize(mat3(normal_xform) * vec3(0.7, 0.7, -1.0));
	vec3 secondary = normalize(mat3(normal_xform) * vec3(1.0, 0.2, -0.4));

	float ndotl = clamp(dot(vec3(0.0, 1.0, 0.0), primary), 0.0, 1.0) * 1.22;
	float ndotl2 = clamp(dot(vec3(0.0, 1.0, 0.0), secondary), 0.0, 1.0);

	col.rgb = col.rgb * (ndotl * primcol + ndotl2 * seccol);

	col.rgb = pow(col.rgb, vec3(1.0/2.2));

	gl_FragColor = col;
}
