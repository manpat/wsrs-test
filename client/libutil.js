

mergeInto(LibraryManager.library, {
	save_canvas_raw: function(target, targetLen) {
		var id = Pointer_stringify(target, targetLen);
		
		var tmp = document.getElementById(id);
		var data = tmp.toDataURL('image/png');

		var dlbutton = document.createElementNS("http://www.w3.org/1999/xhtml", "a");

		if("download" in dlbutton) {
			dlbutton.download = "key.png";
			dlbutton.href = data;

			var event = document.createEvent("MouseEvents");
			event.initMouseEvent(
				"click", true, false, window, 0, 0, 0, 0, 0,
				false, false, false, false, 0, null
			);
			dlbutton.dispatchEvent(event);
		} else {
			window.location.href = data.replace('image/png', 'application/octet-stream');
		}
	},
});
