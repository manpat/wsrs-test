precision mediump float;

attribute vec3 position;
attribute vec3 normal;

uniform mat4 view;
uniform mat4 normal_xform;

varying vec3 vnormal;

void main() {
	vec4 pos = view * vec4(position, 1.0);
	gl_Position = vec4(pos.xyz, 1.0);
	vnormal = mat3(normal_xform) * normal;
}
