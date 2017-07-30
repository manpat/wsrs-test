precision mediump float;

varying vec3 vnormal;

uniform mat4 normal_xform;
uniform vec3 color;

vec3 gamma_correct(vec3 c) {
	return pow(c, vec3(1.0/2.2));
}

void main() {
	vec3 col = color;

	// TODO: Move to post effect
	vec3 primcol = vec3(1.0, 0.975, 0.794);
	vec3 seccol = vec3(0.303, 0.222, 0.147);

	vec3 primary = normalize(mat3(normal_xform) * vec3(0.7, 0.7, -1.0));
	vec3 secondary = normalize(mat3(normal_xform) * vec3(1.0, 0.2, -0.4));

	// vec3 primary = normalize(vec3(2.0, -0.5, -1.0));
	// vec3 primary = normalize(mat3(normal_xform) * vec3(0.7, 0.0, -1.0));
	// vec3 secondary = normalize(mat3(normal_xform) * vec3(1.0, 0.0, -0.4));
	// vec3 primary = normalize(vec3(4.60729, -3.3688,-2.6491));
	// vec3 secondary = normalize(vec3(1.25866, 2.1217,-1.25883));

	float ndotl = clamp(dot(vnormal, primary), 0.0, 1.0) * 1.22;
	float ndotl2 = clamp(dot(vnormal, secondary), 0.0, 1.0) * 0.0;

	gl_FragData[0] = vec4(gamma_correct(col * (ndotl * primcol + ndotl2 * seccol)), 1.0);
	// gl_FragData[0] = vec4(vnormal*0.5 + 0.5, 1.0);
}
