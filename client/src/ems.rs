use context::MainContext;
use std::mem::transmute;
use std::ptr;

#[repr(C)]
struct EmscriptenMouseEvent {
	ts: f64,
	_screen: [i32; 2],
	x: i32, y: i32,
	// ... I don't care about the rest of these fields
}

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
}


pub fn start(ctx_ptr: *mut MainContext) {
	unsafe {
		emscripten_set_click_callback(ptr::null(), ctx_ptr as *mut u8, 0, on_click);
		emscripten_set_main_loop_arg(on_update, ctx_ptr as *mut u8, 0, 1);

		// emscripten_exit_with_live_runtime();
	}
}

extern fn on_update(ud: *mut u8) {
	let mut ctx: &mut MainContext = unsafe{ transmute(ud) };
	ctx.on_update();
}

extern fn on_click(_: i32, ev: *const EmscriptenMouseEvent, ud: *mut u8) -> i32 {
	let mut ctx: &mut MainContext = unsafe{ transmute(ud) };

	// let (ww, wh) = unsafe { get_canvas_size() };
	let (x, y) = unsafe {
		let ev = &*ev;
		(ev.x, ev.y)
	};

	ctx.on_click(x, y);

	1
}