<!-- PROJECT LOGO -->
<br>
<div align="center">
  <a href="https://github.com/andros21/rustracer">
    <img src="https://user-images.githubusercontent.com/58751603/160428859-381f9846-b460-4d9e-bb25-4b111f99fb77.png" alt="Logo" width="70%">
  </a>
  <br>
  <h3>cli photorealistic image generator</h3>
  <div align="center">
    <a href="#installation">Installation</a>
    ·
    <a href="#usage">Usage</a>
    ·
    <a href="#license">License</a>
  </div>
</div>

## Installation

### Prerequisites

**Platform requirements**

`Linux x86_64`

**Compiler requirements**

Install [`cargo`](https://github.com/rust-lang/cargo/) stable latest build system, for devel it's advisable to install the entire (stable latest) toolchain using [`rustup`](https://www.rust-lang.org/tools/install)

### From binary

Install from binary (you can ignore [prerequisites](#prerequisites)):

```bash
$ rustracer="rustrace-$version-x86_64-unknown-linux-gnu"
$ curl -sJSOL "https://github.com/andros21/rustracer/releases/download/$version/$rustracer.tgz"
$ tar -xvf "$rustracer.tgz"
$ install -Dm755 "$rustracer/bin/rustracer" "$PREFIX/bin/rustracer"
```

### From source

Install from source code:

```bash
$ cargo install --locked \
                --root "$PREFIX" \
                --version "$version" \
                --git "https://github.com/andros21/rustracer" rustracer
```

## Usage

Run `rustracer -h` for short help or `rustracer --help` for long help

Example of `rustracer convert` subcommand:

```bash
# convert to png
rustracer convert image.pfm image.png

# convert to farbfeld
rustracer convert image.pfm image.ff
```
## License

`rustracer` is released as open source software under the [GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html) license, see the [LICENSE](https://github.com/andros21/rustracer/blob/master/LICENSE) file in the project root for the full license text
