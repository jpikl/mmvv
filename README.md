# rew

> _The Swiss Army Knife of line-oriented text processing._

[![Build](https://img.shields.io/github/actions/workflow/status/jpikl/rew/ci.yml?branch=master)](https://github.com/jpikl/rew/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/codecov/c/github/jpikl/rew/master?token=9K88E1ZCBU)](https://codecov.io/gh/jpikl/rew)
[![Version](https://img.shields.io/crates/v/rew.svg)](https://crates.io/crates/rew)
[![Dependencies](https://deps.rs/repo/github/jpikl/rew/status.svg)](https://deps.rs/repo/github/jpikl/rew)
[![Downloads](https://img.shields.io/crates/d/rew)](https://crates.io/crates/rew)
[![License](https://img.shields.io/crates/l/rew.svg)](https://github.com/jpikl/rew/blob/master/LICENSE.md)

<!-- ANCHOR: description -->

**rew** is a collection of command-line tools for line-oriented text processing.

It includes tools for a wide range of tasks, such as:

- Text filtering, transformation and generation.
- File system paths manipulation.
- Parallel process pipeline composition.
- Shell metaprogramming (code generation).

All tools are distributed as a single binary (similar to [BusyBox](https://www.busybox.net)).

<!-- ANCHOR_END: description -->

## Documentation

Visit [rew website](https://jpikl.github.io/rew) for installation, usage, examples and more.

<!-- ANCHOR: showcase -->

## Showcase

Let's start with output of the standard Unix `find` command:

```sh
$ find -type f

./README.TXT
./image_1.JPG
./image_2.JPEG
```

Use `rew` subcommands to query components of each path:

```sh
$ find -type f | rew base

README
image_1
image_2
```

```sh
$ find -type f | rew ext

TXT
JPG
JPEG
```

Combine multiple `rew` subcommands to get normalized results:

```sh
$ find -type f | rew ext | rew lower

txt
jpg
jpeg
```

```sh
$ find -type f | rew ext | rew lower | rew replace eg g

txt
jpg
jpg
```

Compose multiple pipelines using `x` subcommand:

```sh
$ find -type f | rew x 'out/{base}.{ext | lower | replace eg g}'

out/README.txt
out/image_1.jpg
out/image_2.jpg
```

Update the pattern to generate shell code:

```sh
$ find -type f | rew x 'mv {} out/{base}.{ext | lower | replace eg g}'

mv ./README.TXT out/README.txt
mv ./image_1.JPG out/image_1.jpg
mv ./image_2.JPEG out/image_2.jpg
```

And pipe it into a shell for execution:

```sh
$ find -type f | rew x 'mv {} out/{base}.{ext | lower | replace eg g}' | sh
```

Or into a tool like [GNU parallel](https://www.gnu.org/software/parallel/parallel.html) for even faster execution:

```sh
$ find -type f | rew x 'mv {} out/{base}.{ext | lower | replace eg g}' | parallel
```

You are not limited only to `rew` subcommands. Call whatever tool you like.
For example, let's use `sed` instead of `rew replace`:

```sh
$ find -type f | rew x 'mv {} out/{base}.{ext | lower | sed s/eg/g/}' | sh
```

More examples can be found in [rew x command reference](https://jpikl.github.io/rew/reference/rew-x.html#examples) or by calling `rew x --examples`.

Have fun using rew!

<!-- ANCHOR_END: showcase -->

## Development

The project uses [just](https://github.com/casey/just) to run development tasks.

Run `just` without arguments to show available recipes.

## License

**rew** source and documentation are released under the [MIT License](LICENSE.md).
