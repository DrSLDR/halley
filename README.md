# Halley

An offsite backup manager, using [restic](https://restic.net/)

## ToDo

First plan of action:

* [ ] Wrapper around Restic
  * [x] Presence - Ensuring restic is installed
  * [x] Init - Creating a repository
  * [x] Backup - Creating a snapshot and putting it in a repo
  * [ ] Forget - Filtering snapshots that are too old
  * [ ] Prune - Actually binning unused data
  * [ ] Stats - Getting repository statistics, for logging
* [ ] Scaffolding to talk to S3

Later on:
* [ ] A CLI
* [ ] Configuration file handling
* [ ] Scheduling (or whatever we'll call that)

## License

This project is licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Is it any good?

[yes.](https://news.ycombinator.com/item?id=3067434)
