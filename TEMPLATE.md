<!-- PROJECT LOGO -->
<br>
<div align="center">
  <a href="https://github.com/andros21/rustracer">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://user-images.githubusercontent.com/58751603/176992080-d96e1e43-5309-45cd-968e-76c4ea132dde.png">
      <img src="https://user-images.githubusercontent.com/58751603/176992080-d96e1e43-5309-45cd-968e-76c4ea132dde.png" alt="Logo" width="470">
    </picture>
  </a>
  <h3 style="border-bottom: 0px;">a multi-threaded raytracer in pure rust</h3>
  <a href="https://github.com/andros21/rustracer/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/andros21/rustracer/ci.yml?branch=master&style=flat-square&label=ci&logo=github" alt="CI"></a>
  <a href="https://github.com/andros21/rustracer/actions/workflows/ci.yml">
    <img src="https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/andros21/0e20cd331d0800e3299298a3868aab7a/raw/rustracer__master.json" alt="Coverage"></a>
  <a href="https://github.com/andros21/rustracer/actions/workflows/cd.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/andros21/rustracer/cd.yml?style=flat-square&label=cd&logo=github" alt="CD"></a>
  <a href="https://github.com/andros21/rustracer/actions/workflows/e2e.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/andros21/rustracer/e2e.yml?style=flat-square&label=e2e&logo=github" alt="E2E"></a>
  <br>
  <a href="https://github.com/andros21/rustracer/releases">
    <img src="https://img.shields.io/github/v/release/andros21/rustracer?color=orange&&sort=semver&style=flat-square&logo=github" alt="Version"></a>
  <a href="https://slsa.dev">
    <img src="https://slsa.dev/images/gh-badge-level3.svg" alt="slsa3"></a>
  <a href="https://crates.io/crates/rustracer">
    <img src="https://img.shields.io/crates/v/rustracer?color=orange&logo=rust&style=flat-square" alt="Cratesio Version"></a>
  <br>
  <a href="https://github.com/andros21/rustracer/blob/master/LICENSE">
    <img src="https://img.shields.io/github/license/andros21/rustracer?color=blue&style=flat-square&logo=gnu" alt="License">
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

- `x86_64-unknown-linux-gnu` <a href="#note1"><sup>(1)</sup></a>
- `x86_64-unknown-linux-musl`

<p id="note1"><sub><strong><sup>(1)</sup> note:</strong> glibc version >= 2.35</sub></p>

### Build requirements

- for **users** install [`cargo`](https://github.com/rust-lang/cargo/) stable latest build system (see [`rust-toolchain.toml`](https://github.com/andros21/rustracer/blob/master/rust-toolchain.toml) for stable version)

- for **devels** install [`rustup`](https://www.rust-lang.org/tools/install) that will automatically provision the correct toolchain

  For unit tests coverage [`cargo-tarpaulin`](https://crates.io/crates/cargo-tarpaulin) is required as additional component

  There is an handy [`makefile`](https://github.com/andros21/rustracer/blob/master/makefile) useful to:

  - preview documentation built with `rustdoc`
  - preview html code coverage analysis created with `cargo-tarpaulin`
  - create demo animations

## Installation

### From binary

Install from binary:

<h4>
<code>curl -sSf https://andros21.github.io/rustracer/install.sh | bash</code>&nbsp;&nbsp;<a href="#note2"><sup>(2)</sup></a>
</h4>

<br>
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

<h4>
   <code> cargo install rustracer</code>&nbsp;&nbsp;<a href="#note3"><sup>(3)</sup></a>
</h4>

<br>
<details>
<summary>click to show other installation options</summary>

```bash
## Install the latest version using `Cargo.lock` in `~/.rustracer/bin`
export PREFIX='~/.rustracer/'
cargo install --locked --root $$PREFIX rustracer

## Install the `0.4.0` version in `~/.rustracer/bin`
export VER='0.4.0'
export PREFIX='~/.rustracer/'
cargo install --root $$PREFIX --version $$VER rustracer
```

</details>

<p id="note3"><sub><strong><sup>(3)</sup> note:</strong> will install latest release in <code>~/.cargo/bin</code></sub></p>

## Usage

### rustracer

| **subcommands**                                   | **description**                              |
| :------------------------------------------------ | :------------------------------------------- |
| [**rustracer-convert**](#rustracer-convert)       | convert an hdr image into ldr image          |
| [**rustracer-demo**](#rustracer-demo)             | render a simple demo scene (example purpose) |
| [**rustracer-render**](#rustracer-render)         | render a scene from file (yaml formatted)    |
| [**rustracer-completion**](#rustracer-completion) | generate shell completion script (hidden)    |

<br>
<details>
<summary>click to show <strong>rustracer -h </strong></summary>

```console
$rustracer
```

</details>

<div align="center"> <hr width="30%"> </div>

### rustracer-convert

Convert a pfm file to png:

<h5>
   <code>rustracer convert image.pfm image.png</code>
</h5>

<br>
<details>
<summary>click to show <strong>rustracer-convert -h </strong></summary>

```console
$rustracer_convert
```

</details>

<div align="center"> <hr width="30%"> </div>

### rustracer-demo

Rendering demo scene:

<div align="center">
   <h5>
      <code>rustracer demo --width 1920 --height 1080 --anti-aliasing 3 demo.png</code>&nbsp;&nbsp;<a href="#note4"><sup>(4)</sup></a>
   </h5>
   <img src="https://github.com/andros21/rustracer/raw/master/examples/demo.png" width="500" alt="rustracer-demo-png"/>
   <p><sub><strong>demo.png:</strong> cpu Intel(R) Xeon(R) CPU E5520 @ 2.27GHz | threads 8 | time ~35s</sub></p>
</div>

\
demo scene 360 degree (see [`makefile`](https://github.com/andros21/rustracer/blob/master/makefile)):

<div align="center">
  <h5>
      <code>make demo.gif</code>&nbsp;&nbsp;<a href="#note4"><sup>(4)</sup></a>
  </h5>
  <img src="https://github.com/andros21/rustracer/raw/master/examples/demo.gif" width="500" alt="rustracer-demo-gif"/>
  <p><sub><strong>demo.gif:</strong> cpu Intel(R) Xeon(R) CPU E5520 @ 2.27GHz | threads 8 | time ~15m</sub></p>
</div>

<br>
<details>
<summary>click to show <strong>rustracer-demo -h </strong></summary>

```console
$rustracer_demo
```

</details>

<p id="note4"><sub><strong><sup>(4)</sup> note:</strong> all available threads are used, set <code>RAYON_NUM_THREADS</code> to override</sub></p>

<div align="center"> <hr width="30%"> </div>

### rustracer-render

Rendering demo scene from scene file [`examples/demo.yml`](https://github.com/andros21/rustracer/blob/master/examples/demo.yml):

<h5>
   <code>rustracer render --anti-aliasing 3 examples/demo.yml demo.png</code>&nbsp;&nbsp;<a href="#note5"><sup>(5)</sup></a>
</h5>

you can use this example scene to learn how to write your custom scene, ready to be rendered!

But let's unleash the power of a scene encoded in data-serialization language such as yaml\
Well repetitive scenes could be nightmare to be written, but for these (and more) there is [`cue`](https://github.com/cue-lang/cue)

Let's try to render a 3D fractal, a [sphere-flake](https://en.wikipedia.org/wiki/Koch_snowflake), but without manually write a yaml scene file\
we can automatic generate it from [`examples/flake.cue`](https://github.com/andros21/rustracer/blob/master/examples/flake.cue)

```bash
cue eval flake.cue -e "flake" -f flake.cue.yml   # generate yml from cue
cat flake.cue.yml | sed "s/'//g" > flake.yml     # little tweaks
wc -l flake.cue flake.yml                        # compare lines number
   92 flake.cue                                  # .
 2750 flake.yml                                  # .
```

so with this trick we've been able to condense a scene info from 2750 to 92 lines, x30 shrink! 😎\
and the generated `flake.yml` can be simple parsed

<div align="center">
   <h5>
   <code>rustracer render --width 1280 --height 720 --anti-aliasing 3 flake.yml flake.png</code>&nbsp;&nbsp;<a href="#note5"><sup>(5)</sup></a>
   </h5>
  <img src="https://github.com/andros21/rustracer/raw/master/examples/flake.png" width="500" alt="rustracer-flake"/>
  <p><sub><strong>flake.png:</strong> cpu Intel(R) Xeon(R) CPU E5520 @ 2.27GHz | threads 8 | time ~7h</sub></p>
</div>

<br>
<details>
<summary>click to show <strong>rustracer-render -h </strong></summary>

```console
$rustracer_render
```

</details>

<p id="note5"><sub><strong><sup>(5)</sup> note:</strong> all available threads are used, set <code>RAYON_NUM_THREADS</code> to override</sub></p>

<div align="center"> <hr width="30%"> </div>

### rustracer-completion

Simple generate completion script for `bash` shell (same for `fish` and `zsh`):

<div align="center">
   <h5>
      <code>rustracer completion bash</code> <a href="#note6"><sup>(6)</sup></a>
   </h5>
   <a href="https://asciinema.org/a/1lqL4683WLvXPfOo5W608je6V?autoplay=1&speed=2" target="_blank"><img src="https://asciinema.org/a/1lqL4683WLvXPfOo5W608je6V.svg" width="500" /></a>
   <p><sub><strong>note:</strong> close-open your shell, and here we go, tab completions now available!</sub></p>
</div>

<br>
<details>
<summary>click to show <strong>rustracer-completion -h </strong></summary>

```console
$rustracer_completion
```

</details>

<p id="note6"><sub><strong><sup>(6)</sup> note:</strong> <code>bash>4.1</code> and <code>bash-complete>2.9</code></sub></p>

<div align="center"> <hr width="30%"> </div>

## Acknowledgements

- [pytracer](https://github.com/ziotom78/pytracer) - a simple raytracer in pure Python
