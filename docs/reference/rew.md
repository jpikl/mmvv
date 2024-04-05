# rew

The Swiss Army Knife of line-oriented text processing.
Transform text by composing parallel shell pipelines!

## Usage

```
rew [OPTIONS] <COMMAND>
```

## Commands

<dl>
<dt><a href="rew-help.html"><code>help</code></a></dt>
<dd>Print this message or the help of the given subcommand(s)</dd>
</dl>

## Transform commands

Transform input text to output. May output a different number of lines than was on input.

<dl>
<dt><a href="rew-join.html"><code>join</code></a></dt>
<dd>Join input lines using a separator</dd>
<dt><a href="rew-loop.html"><code>loop</code></a></dt>
<dd>Repeatedly output all captured input</dd>
<dt><a href="rew-split.html"><code>split</code></a></dt>
<dd>Split input into lines using a separator</dd>
<dt><a href="rew-x.html"><code>x</code></a></dt>
<dd>Compose parallel shell pipelines using a pattern</dd>
</dl>

## Mapper commands

Transform each input line to output. Should output the same number of lines as was on input.

<dl>
<dt><a href="rew-ascii.html"><code>ascii</code></a></dt>
<dd>Convert characters to ASCII</dd>
<dt><a href="rew-cat.html"><code>cat</code></a></dt>
<dd>Copy all input to output</dd>
<dt><a href="rew-lower.html"><code>lower</code></a></dt>
<dd>Convert characters to lowercase</dd>
<dt><a href="rew-trim.html"><code>trim</code></a></dt>
<dd>Trim whitespaces from each line</dd>
<dt><a href="rew-upper.html"><code>upper</code></a></dt>
<dd>Convert characters to uppercase</dd>
</dl>

## Filter commands

Output only certain input lines based on some criteria.

<dl>
<dt><a href="rew-first.html"><code>first</code></a></dt>
<dd>Output first N input lines</dd>
<dt><a href="rew-last.html"><code>last</code></a></dt>
<dd>Output last N input lines</dd>
<dt><a href="rew-skip.html"><code>skip</code></a></dt>
<dd>Skip first N input lines, output the rest</dd>
</dl>

## Generator commands

Generate lines, ignore standard input.

<dl>
<dt><a href="rew-seq.html"><code>seq</code></a></dt>
<dd>Print sequence of numbers as lines</dd>
<dt><a href="rew-stream.html"><code>stream</code></a></dt>
<dd>Print arguments as lines</dd>
</dl>

## Options

<dl>

<dt><code>-h, --help</code></dt>
<dd>

Print help (see a summary with '-h')
</dd>

<dt><code>-V, --version</code></dt>
<dd>

Print version
</dd>
</dl>

## Global options

<dl>

<dt><code>-0, --null</code></dt>
<dd>

Line delimiter is NUL, not newline

Can be also set using `REW_NULL` environment variable.
</dd>

<dt><code>--buf-mode &lt;MODE&gt;</code></dt>
<dd>

Output buffering mode

Possible values:

 - `line` - Writes to stdout after a line was processed or when the output buffer is full. Enabled by default when stdout is TTY (for interactive usage)
 - `full` - Writes to stdout only when the output buffer is full. Enabled by default when stdout is not TTY (for maximal throughput)

Can be also set using `REW_BUF_MODE` environment variable.
</dd>

<dt><code>--buf-size &lt;BYTES&gt;</code></dt>
<dd>

Size of a buffer used for IO operations.

Smaller values will reduce memory consumption but could negatively affect througput.

Larger values will increase memory consumption but may improve troughput in some cases.

Certain commands (which can only operate with whole lines) won't be able to fetch a line bigger than this limit and will abort their execution instead.

Default value: `32768`

Can be also set using `REW_BUF_SIZE` environment variable.
</dd>
</dl>
