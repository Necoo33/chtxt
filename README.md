# Change Text

This program is a simple tool for changing the content of files in terminal with fairly rich synthax.

It warns and asks you to how you proceed in order to prevent you to make mistakes.

It also supports buffering. If you don't specify the buffering threshold, it scans every file with streaming which is bigger than 8 kb.

The synthax is that:

`(binary) (text you want to replace) (text you want to put) (path specifier) (other flags and arguments)`

## Cross Platform Support

Although it's firstly meant for linux, since it's not contains a platform specific api, it also could work on other major platforms.

## Some examples:

`chtxt "hello" "world"`

It changes every "hello" chunk with "world" in every file on your current working directory.

`chtxt "hello" "world" ../`

It changes every "hello" chunk with "world" in every file on parent directory.

`chtxt "hello" "world" blabla`

It changes every "hello" chunk with "world" in "blabla" file on your current working directory.

`chtxt "hello" "world" ../blabla`

It changes every "hello" chunk with "world" in "blabla" file on your parent directory.

`chtxt "hello" --empty`

It deletes every "hello" chunk in every file in current working directory. It also works with that flags: `--delete`, `-empty`, `-delete`, `-e`, `-d`

`chtxt "hello" --empty blabla`

It deletes every "hello" chunk in "blabla" file on your current working directory. It also works with that flags: `--delete`, `-empty`, `-delete`, `-e`, `-d`

`chtxt "hello" "world" ./blabla/blablabla ./blabla/blablablabla ...`

It changes every "hello" chunk with "world" in files on specified paths.

`chtxt "hello" "world" ../blabla ../blablabla ...`

It changes every "hello" chunk with "world" in files on specified path, also works for parent directories.

## Flags

- `--opt`, `--options`: It prompts options and makes program exit after prompting options.
- `--ext`, `--extension` `--extensions`: It specifies the extensions of subject-to-change files. In other words, If you specify that, only the files with that extensions will be scanned and changes. Examples: `--ext js css html`, `--extensions .js .css .html`
- `--bt`, `--buffering-threshold`: It's default 8192 byte(8 kb.), that means if a file is bigger than 8 kb will be scanned via buffering, if it smaller than that it scanned directly. Example: `--bt 8388608`
- `--bs`, `--buffer-size`: It's default to 65536 byte(64 kb.), that means if you scanning files via streaming, buffered chunks size will be 64 kb. If you scan files directly, it has not any effect. Example: `--bs 16384`
