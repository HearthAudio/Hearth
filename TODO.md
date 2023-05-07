# TODO/Tech Debt Before Launch:
- [ ] Add all other non-implemented Commands see COMMANDS.md for list
- [ ] Add env variable for setting config file and kafka ssl 
- [ ] Better Error Handling - To Improve reliability remove as much unwraps and excepts as possible and use matches with error handlers that try to make it work no matter what happens e.g. pulling cached values that may be out of date, but it might work, retrying requests, and other such methods. Also failover for the scheduler.
- [ ] Worker Ping-Pong and Worker Interface for dynamic joins and disconnects
- [ ] Add License warning if used on over 1000 servers
- [ ] Move a bunch of stuff into Config
- [ ] Reduce Clones https://thenewwazoo.github.io/clone.html
- [ ] Fix random intermittent failed to fill full buffer errors
- [ ] Return Errors to client instead of just reporting internally
# TODO General (Post-Launch)
- [ ] Dashboard
- [ ] Add SoundCloud support. See `soundcloud` crate
- [ ] Swap `youtube-dl` with the `rustube` crate (May require forking `rustube` and modifying)
- [ ] Support age restricted videos in `rustube`
- [ ] Fuzzing
- [ ] Store Buffers in Opus Compressed format for increased concurrent stream support
- [ ] Add Effects (Timescale,Rotation,Vibrato, and Distortion)
- [ ] Possible performance improvement by using single separate channel to listen DWC communications and store in HashMap instead of broadcasting. Use tokioselect! in loop to await standard ipc and seperate channel
# DONE
- [x] Add YouTube support
- [x] Replace full memory download with chunking for url player and other players
- [x] Add warning if youtube-dl is not installed that youtube is not supported