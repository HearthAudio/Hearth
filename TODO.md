# TODO:
- [ ] Add Effects (Volume,Timescale,Rotation,Vibrato,Channel Mix, and Distortion)
- [ ] Add YouTube support
- [ ] Add Soundcloud support
- [ ] Add env variable for setting config file and kafka ssl 
- [ ] Better Error Handling - To Improve reliability remove as much unwraps and excepts as possible and use matches with error handlers that try to make it work no matter what happens e.g. pulling cached values that may be out of date, but it might work, retrying requests, and other such methods.
- [ ] Possible performance improvement by using single seperate channel to listen DWC communications and store in HashMap instead of broadcasting. Use tokioselect! in loop to await standard ipc and seperate channel
- [ ] Worker Ping-Pong and Worker Interface for dynamic joins and disconnects
- [ ] Add License warning if used on over 1000 servers
- [ ] Add warning if youtube-dl is not installed that youtube is not supported