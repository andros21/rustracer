<!-- PROJECT LOGO -->
<br>
<div align="center">
  <a href="https://github.com/andros21/rustracer">
    <img src="https://user-images.githubusercontent.com/58751603/160428859-381f9846-b460-4d9e-bb25-4b111f99fb77.png" alt="Logo" width="70%">
  </a>
  <h3 style="border-bottom: 0px;">cli photorealistic image generator</h3>
  <a href="https://github.com/andros21/rustracer/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/workflow/status/andros21/rustracer/CI?style=flat-square&label=ci&logo=github" alt="CI">
  </a>
  <a href="https://codecov.io/gh/andros21/rustracer">
    <img src="https://img.shields.io/codecov/c/gh/andros21/rustracer?logo=codecov&style=flat-square" alt="Coverage">
  </a>
  <a href="https://github.com/andros21/rustracer/releases">
    <img src="https://img.shields.io/github/v/release/andros21/rustracer?color=orange&&sort=semver&style=flat-square" alt="Version">
  </a>
  <a href="https://github.com/andros21/rustracer/blob/master/LICENSE">
    <img src="https://img.shields.io/github/license/andros21/rustracer?color=blue&style=flat-square" alt="License">
  </a>
  <div align="center">
    <a href="#prerequisites">Prerequisites</a>
    ·
    <a href="#installation">Installation</a>
    ·
    <a href="#usage">Usage</a>
  </div>
</div>

## Installation

### Prerequisites

**Platform requirements**

* `x86_64-unknown-linux-gnu` (with `glibc>=2.27`)
* `x86_64-unknown-linux-musl`

**Build requirements**

Install [`cargo`](https://github.com/rust-lang/cargo/) stable latest build system, \
for **devel** it's advisable to install the entire (stable latest) toolchain using [`rustup`](https://www.rust-lang.org/tools/install)

> **devel**: `llvm-tools-preview` additional component is required for unit tests coverage and \
> it's advisable to install [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) for easily use LLVM source-based code coverage

### From binary

Install from binary (you can ignore **Build requirements**):

```bash
$ rustracer="rustracer-$version-$platform"
$ curl -sJSOL "https://github.com/andros21/rustracer/releases/download/$version/$rustracer.tgz"
$ tar -xvf "$rustracer.tgz"
$ install -Dm755 "$rustracer/bin/rustracer" "$PREFIX/bin/rustracer"
```

### From source

Install from source code:

```bash
$ cargo install --locked \
                --root "$PREFIX" \
                --tag "$version" \
                --git "https://github.com/andros21/rustracer" rustracer
```

## Usage

Run `rustracer -h` for short help or `rustracer --help` for long help

Example of `rustracer convert` subcommand:

```bash
$ rustracer convert image.pfm image.ff  # convert to farbfeld
$ rustracer convert image.pfm image.png # convert to png
```
