

mergeInto(LibraryManager.library, {
	dc_set_userdata: function(ptr) {
		dc_userdata = ptr;
	},

	get_canvas_width: function() {
		return cv.width;
	},
	get_canvas_height: function() {
		return cv.height;
	},


	dc_fill_color: function(r, g, b, a) {
		draw_commands.push({
			type: "fill_style",
			style: "rgba("+r+","+g+","+b+","+a+")"
		});
	},
	dc_stroke_color: function(r, g, b, a) {
		draw_commands.push({
			type: "stroke_style",
			style: "rgba("+r+","+g+","+b+","+a+")"
		});
	},

	dc_fill_rect: function(x,y,w,h) {
		draw_commands.push({
			type: "fill_rect",
			x:x, y:y, w:w, h:h
		});
	},

	dc_set_font_raw: function(text) {
		draw_commands.push({
			type: "set_font",
			font: Pointer_stringify(text)
		});
	},

	dc_fill_text_raw: function(text,x,y) {
		draw_commands.push({
			type: "fill_text",
			x:x, y:y, text: Pointer_stringify(text)
		});
	},

	dc_draw_circle: function(x,y,r) {
		draw_commands.push({
			type: "draw_circle",
			x:x, y:y, r: Math.max(r, 0)
		});
	},

	dc_fill_circle: function(x,y,r) {
		draw_commands.push({
			type: "fill_circle",
			x:x, y:y, r: Math.max(r, 0)
		});
	},
});
