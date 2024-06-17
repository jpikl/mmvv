# rew seq

Print sequence of numbers as lines

## Usage

```
rew seq [OPTIONS] [FROM..[TO]] [STEP]
```

## Arguments

<dl>
<dt><code>[FROM..[TO]]</code></dt>
<dd>

Sequence range.

Both `FROM` and `TO` are integers.

`TO` may be omitted to produce an infinite sequence.

Default value: `1..`
</dd>
<dt><code>[STEP]</code></dt>
<dd>

Increment between numbers in sequence.

Default value: `1` (for increasing sequence), `-1` (for decreasing sequence)
</dd>
</dl>

## Options

<dl>

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

Print numbers from 1 to 3.

```sh
rew seq 1..3
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>1</code></li>
<li><code>2</code></li>
<li><code>3</code></li>
</ul>
</div>
</div>

Print numbers from 1 to 5 with step 2.

```sh
rew seq 1..5 2
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>1</code></li>
<li><code>3</code></li>
<li><code>5</code></li>
</ul>
</div>
</div>

Print numbers from 1 to -1.

```sh
rew seq 1..-1
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>1</code></li>
<li><code>0</code></li>
<li><code>-1</code></li>
</ul>
</div>
</div>

Print numbers from 1 to -3 with step -2.

```sh
rew seq 1..-3 -2
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>1</code></li>
<li><code>-1</code></li>
<li><code>-3</code></li>
</ul>
</div>
</div>
