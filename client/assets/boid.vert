precision mediump float;

attribute vec3 position;

uniform mat4 view;
uniform float pointScale;

void main() {
	vec4 pos = view * vec4(position, 1.0);
	gl_Position = vec4(pos.xyz, 1.0);
	gl_PointSize = pointScale;
}
