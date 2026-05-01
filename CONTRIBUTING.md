# Contributing

Thank you for wanting to contribute to `nmrs`!

I'm fairly accepting to all PR's, only with a couple caveats: 
- Do not submit low-effort or purely LLM generated code. If you absolutely must, please disclose _how_ you used AI otherwise I will close the PR.
- Please try to (when possible) contribute to an [issue](https://github.com/cachebag/nmrs/issues). This is not a hard ask, I'll still consider your contribution if it makes sense.

## Requirements

**To run or develop nmrs you need:**
- Rust (stable) via `rustup`
- A running `NetworkManager` instance 

I also provide a `Dockerfile` you can build if you don't use Linux and use macOS instead. 

**To build the image:**
```bash
docker build -t nmrs-lib .
```

**To run tests:**
```bash
docker compose run test
```

**To run an interactive shell:**
```bash
docker compose run shell
```

If you just want quick builds/tests without the full NetworkManager environment:
```bash
docker run --rm nmrs-lib cargo test -p nmrs --lib
docker run --rm -it -v $(pwd):/app nmrs-lib   # mounts local changes
```

If you decide to run the shell, ensure you run all commands from within the nmrs directory, not root.
```bash
cargo test -p nmrs           # run library tests
cargo build -p nmrs          # build the library
cargo check                  # you get the point...
```

## When your branch falls behind `master`

If the respective branch for a PR goes out of sync, I prefer you _rebase_. 
I've exposed this setting for you to to automatically do so as a contributor on any PR you open.

## Issues and Commit Message Hygiene
When you've made changes and are ready to commit, I prefer that you follow the standards explained [here](https://www.conventionalcommits.org/en/v1.0.0/).

I additionally request that you format your commits as such: <br>
"type((some issue number)): changes made", <br> i.e.
```log
fix(#24): fixed bug where something was happening
```
Obviously, if there is no issue number to attach, no need to add anything there.

Lastly, please ensure you make [atomic commmits](https://en.wikipedia.org/wiki/Atomic_commit).


All issues are acceptable. If a situation arises where a request or concern is not valid, I will respond directly to the issue. 


## Tests
All tests must pass before a merge takes place.

### Ensure NetworkManager is running
```bash
sudo systemctl start NetworkManager
```

### Test everything (unit + integration)
```bash
cargo test --all-features
```

### Integration tests
These require WiFi hardware. Please make sure you 
run this locally before your PR to ensure everything works.
```bash
cargo test --test integration_test --all-features
```

If you do not have access to WiFi hardware (for whatever odd reason that is), you can do something like this:
```bash
sudo modprobe mac80211_hwsim radios=2
cargo test --test integration_test --all-features
sudo modprobe -r mac80211_hwsim
```
> [!NOTE]
>
> This method only works on linux

## License

All contributions fall under the [MIT License](https://github.com/cachebag/nmrs?tab=MIT-1-ov-file).
