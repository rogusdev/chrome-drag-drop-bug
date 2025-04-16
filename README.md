# wasm-drag-drop-test
Test Rust WASM web_sys drag &amp; drop DragEvent for quotes in filenames

## Testing
Generally: run `./r` to start the server (must have rust + trunk installed). Drag & drop files into the outlined box. See the output below the drop zone.

Specifically: to see the error, drag & drop the `"Quoted".txt` (or the single quote version -- both single + double quotes induce this) with or without some of the other (non quotes) files. Some nontrivial percent of the time (50%? of the time on first try, less on subsequent tries) if a file with quotes in the filename is dropped, either none of the files are registered ("Items: 0" is output), or only the files without quotes get recognized ("Items: X" where X is less than the number that was actually dropped). It might be the case that this only fails in Chrome.

Verified in Linux (Ubuntu) Firefox (seems to consistently work) + Chrome (seems to fail often, but not always, usually on the first try) -- Windows does not allow double quote chars in filenames to test fully there, but Firefox + Chrome both work fine for single quotes filesnames, it seems (and non quotes filenames, as expected).

See [quoted_drop_failures.png] for a screenshot of this behavior in action. (Repeated drag & drops of the single and double quoted versions, individually.)

A non-wasm (raw javascript) implementation is also provided to demonstrate that it does not experience the same issue.

