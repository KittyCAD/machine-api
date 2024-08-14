# Bambu X1 Carbon RTSP streaming

Authenticates with a Bambu X1 Carbon RTSP stream and shows the frames in an SDL2 window. The printer
IP and access code are currently hard coded. SDL2 can be a bit of a pain to get working on macOS but
[this](https://github.com/embedded-graphics/simulator?tab=readme-ov-file#macos-brew) works well for
me at least.

```bash
cargo run --release
```

`RUST_LOG=debug` can be useful for debugging RTSP auth issues.

Note that exiting the program doesn't work very well, even though to my knowledge the SDL event loop
isn't blocked. Try using `killall bambu-steam` on Linux.
