# Substack Scraper
This scrapes Substack blogs for all their post content and outputs it into raw text files. This project was intended to create neural network training data.

*NOTE*: This project currently cannot get around subscriber-only Substack articles; it will output the truncated article text along with the subscriber message.

# Usage
```shell
git clone https://github.com/ivyraine/substack_scraper
cargo run -- -w <BLOGS>
```
Example:
```sh
## Example:
cargo run -- -w "https://substack.thewebscraping.club/ https://etiennefd.substack.com/"
```
For debug messages, set envvar `RUST_LOG=debug`

# Contributing
Feel free to open an issue or PR if you have any suggestions or improvements, but I cannot guarantee that I'll get to them! The project is small and has some documentation, so I would encourage putting up a PR if you have a feature you want to add.

