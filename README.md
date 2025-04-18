# chrome-drag-drop-bug
Demo Chrome drag &amp; drop event bug for double quotes in filenames. WASM implementation provided as well, but only because that was where it was first encountered. The problem is in the Chrome browser and thus exhibits clearly in raw Javascript without WASM involvement.

The problem is that in Chrome, dropped files with double quotes in their filenames will not show up in child handles for a directory from `getAsFileSystemHandle`. Since Windows does not allow double quotes in filenames, this is not an issue there, but does affect Mac and Linux.

There is a secondary problem that in Linux Chromium (at least in a VirtualBox VM, which might be affecting behavior), sometimes the drop lists 0 items, regardless of filenames. (But still skips filenames with double quotes if other files are processed successfully.)

A demo (raw javascript) implementation [demo.html](demo.html) is provided to confirm. Open that file in Chrome, drag & drop files from `/files` folder, or the folder itself, to test the described reproduction steps. See the output below the drop zone for confirmation.

There are [screenshots](screens/) of the various outcomes in each OS and browser.
