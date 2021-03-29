# 💬 Output

By default, results are printed as lines to standard output.
`LF` character is used as a line terminator.

- Use `-T, --print` option to print results terminated by a specific string.
- Use `-Z, --print-nul` flag to print results terminated by `NUL` character.
- Use `-R, --print-raw` flag to print results without a terminator.
- Use `-L, --no-print-end` flag to disable printing terminator for the last result.

The following table shows how values would be printed for valid combinations of flags/options:

| Values        | Flags    | Output      |
| ------------- | -------- | ----------- |
| `a`, `b`, `c` | *(none)* | `a\nb\nc\n` |
| `a`, `b`, `c` | `-L`     | `a\nb\nc`   |
| `a`, `b`, `c` | `-Z`     | `a\0b\0c\0` |
| `a`, `b`, `c` | `-LZ`    | `a\0b\0c`   |
| `a`, `b`, `c` | `-T:`    | `a:b:c:`    |
| `a`, `b`, `c` | `-LT:`   | `a:b:c`     |
| `a`, `b`, `c` | `-R`     | `abc`       |

Apart from this (standard) mode, there are also two other output modes.

## 🤖 Diff mode

- Enabled using `-b, --diff` flag.
- Respects `--print*` flags/options.
- Ignores `--no-print-end` flag.
- Prints machine-readable transformations as results:

```text
<input_value_1
>output_value_1
<input_value_2
>output_value_2
...
<input_value_N
>output_value_N
```

Such output can be processed by accompanying `mvb` and `cpb` utilities to perform bulk move/copy.

```bash
find -name '*.jpeg' | rew -d '{B}.jpg' | mvb # Rename all *.jpeg files to *.jpg
find -name '*.txt'  | rew -d '{}.bak'  | cpb # Make backup copy of each *.txt file
```

## 🌹 Pretty mode

- Enabled using `-p, --pretty` flag.
- Ignores `--print*` flags/options.
- Ignores `--no-print-end` flag.
- Prints human-readable transformations as results:

```text
input_value_1 -> output_value_1
input_value_2 -> output_value_2
...
input_value_N -> output_value_N
```
