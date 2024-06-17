# rew cat

Copy all input to output

Mostly useful for benchmarking raw IO throughput of rew.

## Usage

```
rew cat [OPTIONS]
```

## Options

<dl>

<dt><code>-l, --lines</code></dt>
<dd>

Process data as lines.

Will normalize newlines to LF as a side effect.
</dd>

<dt><code>-c, --chars</code></dt>
<dd>

Process data as character chunks
</dd>

<dt><code>-b, --bytes</code></dt>
<dd>

Process data as byte chunks
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

Copy input to output.

```sh
rew cat
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
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
</div>
