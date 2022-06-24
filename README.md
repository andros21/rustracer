<!-- PROJECT LOGO -->
<br>
<div align="center">
  <a href="https://github.com/andros21/rustracer">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://user-images.githubusercontent.com/58751603/176992080-d96e1e43-5309-45cd-968e-76c4ea132dde.png">
      <img src="https://user-images.githubusercontent.com/58751603/176992080-d96e1e43-5309-45cd-968e-76c4ea132dde.png" alt="Logo" width="470">
    </picture>
  </a>
  <h3 style="border-bottom: 0px;">cli photorealistic image generator</h3>
  <a href="https://github.com/andros21/rustracer/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/workflow/status/andros21/rustracer/CI?style=flat-square&label=ci&logo=github" alt="CI">
  </a>
  <a href="https://github.com/andros21/rustracer/actions/workflows/ci.yml">
    <img src="https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/andros21/0e20cd331d0800e3299298a3868aab7a/raw/rustracer__master.json" alt="Coverage">
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

## Prerequisites

### Platform requirements

* `x86_64-unknown-linux-gnu` <a href="#note1"><sup>(1)</sup></a>
* `x86_64-unknown-linux-musl`

<p id="note1"><sub><strong><sup>(1)</sup> note:</strong> glibc version >= 2.27</sub></p>

### Build requirements

* for **users** install [`cargo`](https://github.com/rust-lang/cargo/) stable latest build system

* for **devels** it's advisable to install the entire (stable latest) toolchain using [`rustup`](https://www.rust-lang.org/tools/install)

   For unit tests coverage `llvm-tools-preview` is required as additional component coupled with\
   [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) for easily use LLVM source-based code coverage

  There is an handy [`makefile`](https://github.com/andros21/rustracer/blob/master/makefile) useful to:
    + preview documentation built with `rustdoc`
    + preview html code coverage analysys created with `cargo-llvm-cov`
    + create demo animations

## Installation

### From binary

Install from binary:

#### `curl -sSf https://andros21.github.io/rustracer/install.sh | bash` <a href="#note2"><sup>(2)</sup></a>

<details>
<summary>click to show other installation options</summary>

```bash
## Install the latest version `gnu` variant in `~/.rustracer/bin`
export PREFIX='~/.rustracer/'
curl -sSf https://andros21.github.io/rustracer/install.sh | bash -s -- gnu

## Install the `0.4.0` version `musl` variant in `~/.rustracer/bin`
export PREFIX='~/.rustracer/'
curl -sSf https://andros21.github.io/rustracer/install.sh | bash -s -- musl 0.4.0
```
</details>

<p id="note2"><sub><strong><sup>(2)</sup> note:</strong> will install latest musl release in <code>~/.local/bin</code></sub></p>

### From source

Install from source code, a template could be:

#### `cargo install rustracer` <a href="#note3"><sup>(3)</sup></a>

<details>
<summary>click to show other installation options</summary>

```bash
## Install the latest version using `Cargo.lock` in `~/.rustracer/bin`
export PREFIX='~/.rustracer/'
cargo install --locked --root $PREFIX rustracer

## Install the `0.4.0` version in `~/.rustracer/bin`
export VER='0.4.0'
export PREFIX='~/.rustracer/'
cargo install --root $PREFIX --version $VER rustracer
```
</details>

<p id="note3"><sub><strong><sup>(3)</sup> note:</strong> will install latest release in <code>~/.cargo/bin</code></sub></p>

## Usage

### rustracer

Available subcommands:

* [**rustracer-convert**](#rustracer-convert) - convert an hdr image into ldr image
* [**rustracer-demo**](#rustracer-demo) - render a simple demo scene (example purpose)
* [**rustracer-render**](#rustracer-render) - render a scene from file (yaml formatted)
* [**rustracer-completion**](#rustracer-completion) - generate shell completion script (hidden)

<details>
<summary>click to show <strong>rustracer</strong> help</summary>

```console
rustracer 0.4.0
cli photorealistic image generator

USAGE:
    rustracer <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    convert    Convert HDR (pfm) image to LDR (ff|png) image
    demo       Render a demo scene (hard-coded in main)
    render     Render a scene from file
```
</details>

<div align="center"> <hr width="30%"> </div>

### rustracer-convert

An example could be:

#### `rustracer convert image.pfm image.png`

<details>
<summary>click to show <strong>rustracer-convert</strong> help</summary>

```console
rustracer-convert 0.4.0
Convert HDR (pfm) image to LDR (ff|png) image

USAGE:
    rustracer convert [OPTIONS] <HDR> <LDR>

ARGS:
    <HDR>    Input pfm image
    <LDR>    Output image [possible formats: ff, png]

OPTIONS:
    -f, --factor <FACTOR>    Normalization factor [default: 0.2]
    -g, --gamma <GAMMA>      Gamma parameter [default: 1.0]
    -h, --help               Print help information
    -v, --verbose            Print stdout information
    -V, --version            Print version information
```
</details>

<div align="center"> <hr width="30%"> </div>

### rustracer-demo

An example could be:

#### `rustracer demo --anti-aliasing 3 -f 1 demo.png` <a href="#note4"><sup>(4)</sup></a>

360 degree view of demo scene (see [`makefile`](https://github.com/andros21/rustracer/blob/master/makefile)):

#### `make demo.gif`

<details>
<summary>click to show <strong>rustracer-demo</strong> help</summary>

```console
rustracer-demo 0.4.0
Render a demo scene (hard-coded in main)

USAGE:
    rustracer demo [OPTIONS] <OUTPUT>

ARGS:
    <OUTPUT>    Output image [possible formats: ff, png]

OPTIONS:
    -a, --algorithm <ALGORITHM>            Rendering algorithm [default: pathtracer] [possible values: onoff, flat,
                                           pathtracer]
        --angle-deg <ANGLE_DEG>            View angle (in degrees) [default: 0.0]
        --anti-aliasing <ANTI_ALIASING>    Anti-aliasing level [default: 1]
    -f, --factor <FACTOR>                  Normalization factor [default: 0.2]
    -g, --gamma <GAMMA>                    Gamma parameter [default: 1.0]
    -h, --help                             Print help information
        --height <HEIGHT>                  Image height [default: 480]
        --init-seq <INIT_SEQ>              Identifier of the random sequence (positive number) [default: 45]
        --init-state <INIT_STATE>          Initial random seed (positive number) [default: 45]
    -m, --max-depth <MAX_DEPTH>            Maximum depth [default: 3]
    -n, --num-of-rays <NUM_OF_RAYS>        Number of rays [default: 10]
        --orthogonal                       Use orthogonal camera instead of perspective camera
        --output-pfm                       Output also hdr image
    -v, --verbose                          Print stdout information
    -V, --version                          Print version information
        --width <WIDTH>                    Image width [default: 640]

```
</details>

<p id="note4"><sub><strong><sup>(4)</sup> note:</strong> all available threads are used, set <code>RAYON_NUM_THREADS</code> to override</sub></p>

<div align="center"> <hr width="30%"> </div>

### rustracer-render

An example could be rendering demo scene from scene [`examples/demo.yml`](https://github.com/andros21/rustracer/blob/master/examples/demo.yml):

#### `rustracer render --anti-aliasing 3 -f 1 examples/demo.yml` <a href="#note5"><sup>(5)</sup></a>

you can use this example scene to learn how to write your custom scene, ready to be rendered!

But let's unleash the power of a scene encoded in data-serialization language such as yaml\
Well repetitive scenes could be nightmare to be written, but for these (and more) there is [`cue`](https://github.com/cue-lang/cue)

Let's try to render a 3D fractal, a [sphere-flake](https://en.wikipedia.org/wiki/Koch_snowflake), but without manually write a yaml scene file\
we can automatic genererate it from [`examples/flake.cue`](https://github.com/andros21/rustracer/blob/master/examples/flake.cue)

```bash
cue eval flake.cue -e "flake" -f flake.cue.yml # generate yml from cue
cat flake.cue.yml | sed "s/'//g" > flake.yml   # little tweaks
wc -l flake.cue flake.yml                      # compare lines number
   92 flake.cue                                # .
32666 flake.yml                                # .
```
so with this trick we've been able to condense a scene info from 32666 to 92 lines, x350 shrink! :sunglasses:\
and the generated `flake.yml` can be simple parsed

#### `rustracer render --anti-aliasing 3 -f 1 flake.yml flake.png` <a href="#note5"><sup>(5)</sup></a>

<details>
<summary>click to show <strong>rustracer-render</strong> help</summary>

```console
rustracer-render 0.4.0
Render a scene from file

USAGE:
    rustracer render [OPTIONS] <INPUT> <OUTPUT>

ARGS:
    <INPUT>     Input scene file
    <OUTPUT>    Output image [possible formats: ff, png]

OPTIONS:
    -a, --algorithm <ALGORITHM>            Rendering algorithm [default: pathtracer] [possible values: onoff, flat,
                                           pathtracer]
        --angle-deg <ANGLE_DEG>            View angle (in degrees) [default: 0.0]
        --anti-aliasing <ANTI_ALIASING>    Anti-aliasing level [default: 1]
    -f, --factor <FACTOR>                  Normalization factor [default: 0.2]
    -g, --gamma <GAMMA>                    Gamma parameter [default: 1.0]
    -h, --help                             Print help information
        --height <HEIGHT>                  Image height [default: 480]
        --init-seq <INIT_SEQ>              Identifier of the random sequence (positive number) [default: 45]
        --init-state <INIT_STATE>          Initial random seed (positive number) [default: 45]
    -m, --max-depth <MAX_DEPTH>            Maximum depth [default: 3]
    -n, --num-of-rays <NUM_OF_RAYS>        Number of rays [default: 10]
        --output-pfm                       Output also hdr image
    -v, --verbose                          Print stdout information
    -V, --version                          Print version information
        --width <WIDTH>                    Image width [default: 640]

```
</details>

<p id="note5"><sub><strong><sup>(5)</sup> note:</strong> all available threads are used, set <code>RAYON_NUM_THREADS</code> to override</sub></p>

<div align="center"> <hr width="30%"> </div>

### rustracer-completion

Simple generate completion script for `bash` shell:

#### `rustracer completion bash` <a href="#note6"><sup>(6)</sup></a>

close-open your shell, and here we go, tab completions now available!

<details>
<summary>click to show <strong>rustracer-completion</strong> help</summary>

```console
rustracer-completion 0.4.0
Generate shell completion script

USAGE:
    rustracer completion [OPTIONS] <SHELL>

ARGS:
    <SHELL>    Shell to generate script for [possible values: bash, fish, zsh]

OPTIONS:
    -h, --help               Print help information
    -o, --output <OUTPUT>    Specify output script file
    -V, --version            Print version information

```
</details>

<p id="note6"><sub><strong><sup>(6)</sup> note:</strong> <code>bash>4.1</code> and <code>bash-complete>2.9</code></sub></p>

<div align="center"> <hr width="30%"> </div>

## Acknowledgements

* [pytracer](https://github.com/ziotom78/pytracer) - a simple raytracer in pure Python
