attribute vec3 position;
attribute vec4 color;

uniform mat4 view;

varying vec4 vcolor;

void main() {
	vec4 pos = view * vec4(position, 1.0);
	gl_Position = vec4(pos.xyz, 1.0);
	vcolor = color;
}
