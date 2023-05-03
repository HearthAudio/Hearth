# TODO:
- [ ] Add Effects
- [ ] Add YouTube support
- [ ] Add Soundcloud support
- [ ] Add env variable for setting config file and kafka ssl 
- [ ] Better Error Handling - To Improve reliability remove as much unwraps and excepts as possible and use matches with error handlers that try to make it work no matter what happens e.g. pulling cached values that may be out of date, but it might work, retrying requests, and other such methods.
- [ ] Fix Spaghetti Mess Everywhere but mostly in the Webhook Handler