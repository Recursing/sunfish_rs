# sunfish_rs
Rust "port" of the sunfish simple chess engine

# Credits:
- [The original sunfish](https://github.com/thomasahle/sunfish)
- [yuri91](https://github.com/yuri91) for making a first rust port that inspired this

# How to play:
Challenge it on [lichess](https://lichess.org/@/sunfish_rs) and tell me what you think!

### TODO:
- Improve time managment, maybe rewriting search to be iterative instead of recursive
- Support endgame values (ideally tapered eval), might be tricky to do with incremental updates
- Add more tests, need to test many more positions and add more unit tests
- Benchmarking, maybe build a micro benchmarking framework? See https://github.com/bheisler/criterion.rs/issues/306
- [x] Make Square enums more compact, currently they use twice as much memory as the python chars :/
