# rew x

Compose parallel shell pipelines using a pattern

## Usage

```
rew x [OPTIONS] [PATTERN]...
```

## Arguments

<dl>
<dt><code>[PATTERN]...</code></dt>
<dd>

Output pattern(s).

A pattern describes how to transform each input line into output. Multiple patterns are joined together, using space `' '` as a delimiter.

See examples (`--examples` option) for more details.
</dd>
</dl>

## Options

<dl>

<dt><code>-e, --escape &lt;CHAR&gt;</code></dt>
<dd>

Escape character for the pattern

Default value: `\`
</dd>

<dt><code>-s, --shell &lt;SHELL&gt;</code></dt>
<dd>

Shell used to evaluate `{# ...}` expressions.

Default value: `cmd` on Windows, `sh` everywhere else.

Can be also set using `SHELL` environment variable.
</dd>

<dt><code>-q, --quote</code></dt>
<dd>

Wrap output of every pattern expression in quotes

Use the flag once for single quotes `''` or twice for double quotes `""`.
</dd>

<dt><code>--examples</code></dt>
<dd>

Print examples of the command usage
</dd>

<dt><code>-h, --help</code></dt>
<dd>

Print help (see a summary with '-h')
</dd>
</dl>

## Global options

See [rew reference](rew.md#global-options) for list of additional global options.

## Examples

Empty expression `{}` is replaced by input line.

```sh
rew x 'Hello {}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>Hello first</code></li>
<li><code>Hello second</code></li>
<li><code>Hello third</code></li>
</ul>
</div>
</div>

Expressions can call other `rew` commands to process the input.

Here, we call the `rew upper` command which converts text to uppercase.

```sh
rew x 'Hello {upper}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>Hello FIRST</code></li>
<li><code>Hello SECOND</code></li>
<li><code>Hello THIRD</code></li>
</ul>
</div>
</div>

Expressions can also call any external command.

Let's remove all `aeiou` characters from text using `tr`.

```sh
rew x 'Hello {tr -d aeiou}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>Hello frst</code></li>
<li><code>Hello scnd</code></li>
<li><code>Hello thrd</code></li>
</ul>
</div>
</div>

Multiple commands can be joined into a pipeline.

```sh
rew x 'Hello {tr -d aeiou | upper}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>Hello FRST</code></li>
<li><code>Hello SCND</code></li>
<li><code>Hello THRD</code></li>
</ul>
</div>
</div>

Multiple expressions are run in parallel and their output is combined. The excution runs until one of the expressions no longer produces any output.

```sh
rew x '{seq}. {tr -d aeiou | upper}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>1. FRST</code></li>
<li><code>2. SCND</code></li>
<li><code>3. THRD</code></li>
</ul>
</div>
</div>

Arguments containing whitepaces must be wrapped in single `''` or double quotes `""`.

Here, we replace `aeiou` characters with space `' '`.

```sh
rew x 'Hello {tr aeiou " " | upper}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>Hello F RST</code></li>
<li><code>Hello S C ND</code></li>
<li><code>Hello TH RD</code></li>
</ul>
</div>
</div>

The `!` marker denotes an external command.

Let's call the standard `seq` command instead of the built-in `rew seq`.

```sh
rew x '{!seq 1 3}. {}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>1. first</code></li>
<li><code>2. second</code></li>
<li><code>3. third</code></li>
</ul>
</div>
</div>

The `#` marker denotes "raw shell expression". Everything after it will be interpreted by the current shell.

For example, the following expression is equivalent to `{sh -c 'printf "%s\n" a b c'}`

```sh
rew x '{# printf "%s\n" a b c}. {}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>a. first</code></li>
<li><code>b. second</code></li>
<li><code>c. third</code></li>
</ul>
</div>
</div>

A specific shell for `{# ...}` can be set using the `-s, --shell` option or the `SHELL` environment variable.

```sh
rew x -s bash '{# for((i=0;i<3;i++)); do echo $i; done}. {}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>0. first</code></li>
<li><code>1. second</code></li>
<li><code>2. third</code></li>
</ul>
</div>
</div>

The `:` marker is a hint that an expression does not consume stdin. Without it, the overall execution might get stuck forever due to blocked IO calls.

Only external commands need `:` to be explicitly specified. For built-in commands, `:` is detected automatically.

```sh
rew x '{seq 1..3} {: !seq 1 3} {:# echo 1; echo 2; echo 3}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>1 1 1</code></li>
<li><code>2 2 2</code></li>
<li><code>3 3 3</code></li>
</ul>
</div>
</div>

Backslash `\` can be used to escape special characters

```sh
rew x '\{ "{}": {seq} \}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>{ "first": 1 }</code></li>
<li><code>{ "second": 2 }</code></li>
<li><code>{ "third": 3 }</code></li>
</ul>
</div>
</div>

A custom escape character can be set using the `-e, --escape` option.

```sh
rew x -e% '%{ "{}": {seq} %}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>{ "first": 1 }</code></li>
<li><code>{ "second": 2 }</code></li>
<li><code>{ "third": 3 }</code></li>
</ul>
</div>
</div>

Certain special characters like `|` must be escaped only within a specific context.

```sh
rew x '| {echo "|"} {echo \|}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>| | |</code></li>
</ul>
</div>
</div>

Escape character can be also used to produce line feed `\n`, carriage return `\r` or tab `\t`.

```sh
rew x '{seq}:\n\t{}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>1:</code></li>
<li><code>    first</code></li>
<li><code>2:</code></li>
<li><code>    second</code></li>
<li><code>3:</code></li>
<li><code>    third</code></li>
</ul>
</div>
</div>

You can enable automatic expression quoting using `-q, --quote` flag.

```sh
rew x -q 'mv {} {lower | tr " " "_"}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>IMG 1.jpg</code></li>
<li><code>IMG 2.jpg</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>mv 'IMG 1.jpg' 'img_1.jpg'</code></li>
<li><code>mv 'IMG 2.jpg' 'img_2.jpg'</code></li>
</ul>
</div>
</div>

Double the `-q, --quote` to use double quotes instead of single quotes.

```sh
rew x -qq 'mv {} {lower | tr " " "_"}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>IMG 1.jpg</code></li>
<li><code>IMG 2.jpg</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>mv "IMG 1.jpg" "img_1.jpg"</code></li>
<li><code>mv "IMG 2.jpg" "img_2.jpg"</code></li>
</ul>
</div>
</div>

All global options `-0, --null`, `--buf-size` and `--buf-mode` are propagated to rew subcommands. Do not forget configure NUL separator manually for any external commands.

```sh
rew x --null '{upper | sed --null-data "s/^.//g"}'
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>aa\0bb\0cc\0</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>A\0B\0C\0</code></li>
</ul>
</div>
</div>
