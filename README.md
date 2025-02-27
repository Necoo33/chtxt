# Change Text

This program is a simple tool for changing the content of files in terminal with rich synthax.

It warns and asks you to how you proceed in order to prevent you to make mistakes.

The synthax is that:

`(binary) (text you want to replace) (text you want to put) (path specifier)`

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

`chtxt "hello" "world" ./blabla/blablabla ./blabla/blablablabla ...`

It changes every "hello" chunk with "world" in files on specified paths.

`chtxt "hello" "world" ../blabla ../blablabla ...`

It changes every "hello" chunk with "world" in files on specified path, also works for parent directories.