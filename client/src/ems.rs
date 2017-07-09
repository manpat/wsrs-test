use context::MainContext;
use std::mem::transmute;
use std::ffi::CString;
use std::ptr;

#[repr(C)]
struct EmscriptenMouseEvent {
	ts: f64,
	_screen: [i32; 2],
	x: i32, y: i32,
	// ... I don't care about the rest of these fields
}

#[repr(C)]
pub struct EmscriptenWebGLContextAttributes {
	pub alpha: i32,
	pub depth: i32,
	pub stencil: i32,
	pub antialias: i32,
	pub premultiplied_alpha: i32,
	pub preserve_drawing_buffer: i32,
	pub prefer_low_power_to_high_performance: i32,
	pub fail_if_major_performance_caveat: i32,

	pub major_version: i32,
	pub minor_version: i32,

	pub enable_extensions_by_default: i32,
}

pub type EmWebGLContext = i32;
pub type EmSocketCallback = extern fn(fd: i32, ud: *mut u8);
type EmMouseCallback = extern fn(etype: i32, evt: *const EmscriptenMouseEvent, ud: *mut u8) -> i32;
type EmArgCallback = extern fn(ud: *mut u8);

#[allow(dead_code, improper_ctypes)]
extern {
	fn emscripten_set_main_loop_arg(func: extern fn(arg: *mut u8), arg: *mut u8, fps: i32, simulate_infinite_loop: i32);
	fn emscripten_exit_with_live_runtime();

	pub fn emscripten_set_socket_open_callback(ud: *mut u8, callback: EmSocketCallback);
	pub fn emscripten_set_socket_close_callback(ud: *mut u8, callback: EmSocketCallback);
	pub fn emscripten_set_socket_message_callback(ud: *mut u8, callback: EmSocketCallback);

	fn emscripten_set_click_callback(target: *const u8, ud: *mut u8, useCapture: i32, cb: EmMouseCallback);

	pub fn emscripten_async_call(callback: EmArgCallback, ud: *mut u8, millis: i32);
	pub fn emscripten_asm_const_int(s: *const u8, ...) -> i32;

	pub fn emscripten_webgl_init_context_attributes(attribs: *mut EmscriptenWebGLContextAttributes);
	pub fn emscripten_webgl_create_context(target: *const u8, attribs: *const EmscriptenWebGLContextAttributes) -> EmWebGLContext;
	pub fn emscripten_webgl_make_context_current(context: EmWebGLContext) -> i32;
	pub fn emscripten_webgl_destroy_context(context: EmWebGLContext) -> i32;
	pub fn emscripten_webgl_get_current_context() -> EmWebGLContext;
}

pub trait Interop {
	fn as_int(self, _: &mut Vec<CString>) -> i32;
}

impl Interop for i32 {
	fn as_int(self, _: &mut Vec<CString>) -> i32 {
		return self;
	}
}

impl<'a> Interop for &'a str {
	fn as_int(self, arena: &mut Vec<CString>) -> i32 {
		let c = CString::new(self).unwrap();
		let ret = c.as_ptr() as i32;
		arena.push(c);
		return ret;
	}
}

impl<'a> Interop for *const u8 {
	fn as_int(self, _: &mut Vec<CString>) -> i32 {
		return self as i32;
	}
}

#[macro_export]
macro_rules! js {
	( ($( $x:expr ),*) $y:expr ) => {
		{
			use std::ffi::CString;
			let mut arena: Vec<CString> = Vec::new();
			const LOCAL: &'static [u8] = $y;
			unsafe { ::ems::emscripten_asm_const_int(&LOCAL[0] as *const _ as *const u8, $(::ems::Interop::as_int($x, &mut arena)),*) }
		}
	};
	( $y:expr ) => {
		{
			const LOCAL: &'static [u8] = $y;
			unsafe { ::ems::emscripten_asm_const_int(&LOCAL[0] as *const _ as *const u8) }
		}
	};
}


pub fn start(ctx_ptr: *mut MainContext) {
	unsafe {
		emscripten_set_click_callback(ptr::null(), ctx_ptr as *mut u8, 0, on_click);
		emscripten_set_main_loop_arg(on_update, ctx_ptr as *mut u8, 0, 1);
	}
}

extern fn on_update(ud: *mut u8) {
	let mut ctx: &mut MainContext = unsafe{ transmute(ud) };

	ctx.on_update();
	ctx.on_render();
}

extern fn on_click(_: i32, ev: *const EmscriptenMouseEvent, ud: *mut u8) -> i32 {
	let mut ctx: &mut MainContext = unsafe{ transmute(ud) };

	let (x, y) = unsafe {
		let ev = &*ev;
		(ev.x, ev.y)
	};

	ctx.on_click(x, y);

	1
}