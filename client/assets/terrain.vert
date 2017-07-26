attribute vec3 position;
attribute float health;

uniform mat4 view;
uniform sampler2D health_lut;

varying vec4 vcolor;

void main() {
	vec4 pos = view * vec4(position, 1.0);
	gl_Position = vec4(pos.xyz, 1.0);
	vcolor = texture2D(health_lut, vec2(health, 0.5));
}
