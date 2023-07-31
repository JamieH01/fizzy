# Fizzy
Fizzy is a live fuzzy finder/explorer for directories. Press enter to exit and save a `cd` command corresponding to the last entry to your clipboard to quickly jump to any directory.
Use the arrow keys to select the path you want. The full command is printed to stdout if you want to use fizzy with a shell script.

### Installation
Fizzy can be installed from the crates.io registry:
```
cargo install fizzy-rs
```
or you can build from soure:
```
git clone https://github.com/JamieH01/fizzy ./fizzy && cargo install --path ./fizzy
```
