# TODO/Tech Debt Before Launch (Feature Freeze):
- [ ] Add all other non-implemented Commands see COMMANDS.md for list
- [ ] Scheduler fail over with dead marking if fail rate is high
- [ ] Move a bunch of stuff into Config
- [ ] Fix random intermittent failed to fill full buffer errors
- [ ] Unit Tests
- [ ] Fix weird playback speed bug
- [ ] Fix file type parser if there are `.`s in the path other than file extension
# TODO General (Post-Launch)
- [ ] Chunk Youtube downloads
- [ ] Dashboard
- [ ] Worker Interface for dynamic joins and disconnects
- [ ] Find better way to handle message struct than a thousand options.
- [ ] Add SoundCloud support. See `soundcloud` crate
- [ ] Swap `youtube-dl` with the `rustube` crate (May require forking `rustube` and modifying) (This has a few issues chunk throttling - age restricted downloads etc... see Issues on repo)
- [ ] Support age restricted videos in `rustube`
- [ ] Fuzzing
- [ ] Exposed Track Event System
- [ ] Add Effects (Timescale,Rotation,Vibrato, and Distortion)
- [ ] Possible performance improvement by using single separate channel to listen DWC communications and store in HashMap instead of broadcasting. Use tokioselect! in loop to await standard ipc and seperate channel
# DONE
- [x] Add YouTube support
- [x] Do not crash thread on queue processor if error occurs
- [x] Replace full memory download with chunking for url player and other players
- [x] Add warning if youtube-dl is not installed that youtube is not supported
- [x] Worker Ping-Pong
- [x] Return Errors to client instead of just reporting internally
- [x] Add License warning if used on over 1000 servers
- [x] Remove ID from generic kafka definition