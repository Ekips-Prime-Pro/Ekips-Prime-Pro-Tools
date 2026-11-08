# The `ekips` command-line tool

`ekips` checks your Ekips Prime Pro programs for mistakes. Later it will
also run them in the simulator and upload them to the hub.

*Deutsche Version: [../de/cli.md](../de/cli.md)*

## Checking programs: `ekips check`

```sh
ekips check mission.ekips
ekips check mission1.ekips mission2.ekips
ekips check .              # every .ekips file in this folder and its subfolders
```

If everything is fine you see:

```text
mission.ekips: OK, no mistakes found.
```

If there is a mistake, `ekips` shows where it is and how to fix it:

```text
error: I don't understand the word `cosnt` here.
 --> mission.ekips:1:1
  |
1 | cosnt arm = new Motor(Port.C);
  | ^^^^^
  = help: Did you mean `const`?

mission.ekips: 1 error
```

- The line `--> mission.ekips:1:1` means: file `mission.ekips`, line 1, column 1.
- The `^^^^^` marks the exact place.
- The `help:` line suggests a fix.

When you check more than one file, a summary is printed at the end.

### Options of `check`

| Option | Meaning |
|---|---|
| `--ast` | Also print the syntax tree, i.e. how the compiler understood the program (for experts) |

## Options for all commands

| Option | Meaning |
|---|---|
| `--lang en` / `--lang de` | Language of all messages. Without this option the language of your computer is used (German if it is set to German, otherwise English). |
| `--color auto\|always\|never` | Colors in the output. `auto` (default) uses colors only in a terminal. The `NO_COLOR` environment variable also switches colors off. |
| `-h`, `--help` | Show help (in the chosen language) |
| `-V`, `--version` | Show the version |

The environment variable `EKIPS_LANG` (`en` or `de`) sets the language as
well; `--lang` wins over it.

## Planned commands

These commands already exist, but only print a message explaining that they
are not available yet. Nothing is changed on your computer or hub.

| Command | Will do |
|---|---|
| `ekips run --sim <file>` | Run the program in the simulator |
| `ekips run <file>` | Run the program on the hub |
| `ekips upload <file>` | Upload the program to the hub |
| `ekips monitor` | Show what the hub prints (`console.log`) |
| `ekips flash-firmware` | Install the Ekips Prime Pro firmware on the hub |
| `ekips backup-firmware [file]` | Save the hub's current firmware so it can be restored |

## Exit codes

Useful for scripts and editors:

| Code | Meaning |
|---|---|
| `0` | Everything is fine |
| `1` | At least one program has an error |
| `2` | Wrong usage, or a file could not be read |
| `3` | The command is not available yet |

Help texts of `ekips` are translated. Error messages about *wrong usage*
(for example an unknown option) come from the `clap` library and are only
in English; in German mode a German hint follows them.
