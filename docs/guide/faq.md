# FAQ

## What does the "rew" mean?

**rew** is a short for _"rewrite"_.

## How do I pronounce "rew"?

`/ˌriː/` (like _"rewrite"_ but without the _"-write"_)

However, I personally do not care and often just pronounce it simply _"rev"_.

## Why not have a separate binary for each command?

There are several reasons:

1. A single binary is much easier to distribute.
2. Binaries produced by Rust can be quite large due to being statically linked.
   Although there are [ways how to optimize this](https://github.com/johnthagen/min-sized-rust), having just a single binary results in a huge size reduction overall.
3. It is quite difficult to come up with reasonable names for new commands which do not already exist (`seq`, `last`).
   Having subcommands means I can name them whatever I want.

If you find this inconvenient, feel free to add aliases for your shell:

```shell
alias trim='rew trim'
alias x='rew x'
...
```
