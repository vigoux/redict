![redict](https://i.imgur.com/UIl1mRO.jpg)
# redict
A TUI and library to work with DICT servers.

# Using

## Starting

To start `redict` simply do :

```
redict {server}:{port}
```

## Searching

To search, just type the word you want to find the definition of !
You also have two useful tools in the search bar :

  - Prefix anything by `@` and it will select a database
  - Prefix anything by `:` and it will select a matching strategy

Thus :
```
word @database :strategy
```

Means `search "word" in "database" wiht algorithm "strategy"`.

## Navigating

We can split `redict` screen in 3 parts :

- The `Search` bar (top bar)
- The `Status` bar (just below `Search` bar)
- The "mode" space (everything appart the two aforementionned),
  delimited by the mode indicator.

There are generic key bindings :

| Key | Action |
|-----|--------|
| `Esc` | Exit `redict` |
| `PageUp` | Scroll up in the tab page |
| `PageDown` | Scroll down in the tab page |
| `Tab` | Go to next mode |
| `BackTab` | Go to previous mode |
| `Up` | Go up in `Search` bar history |
| `Down` | Go down in `Search` bar history |
| `CTRL-u` | Empty search bar |
| `Enter` | Refreshes current mode, possibly using the currently searched term |
| `Left` | Move search bar cursor to the left |
| `Right` | Move search bar cursor to the right |
| `Home` | Move cursor to the start of the search bar |
| `End` | Move cursor to the end of the search bar |

Moreover, you can use your keyboard to edit the search bar direclty,
that is anything typed will be added to the search bar.

## Modes

Each mode does a different thing and allows you to view different
informations about the server.

### Define

Shows the definition on the currently searched term.

You can you `CTRL-h` and `CTRL-l` to navigate definitions.

Definitions are listed in the `Sources` bar, at the very bottom of the
screen.

### Match

Shows words matching the currently searched term.

This may be useful if your search show `No definition` in `Define`
mode.

### Strategies

Shows the available matching strategies for this server.

### Databases

Shows the available databases for this server.

# TODO

## Features

I would like to:

- [x] Basically working binary and library
- [ ] Have a better search bar
  - [x] Database filtering in `Define` and `Match` mode using `@db`
  - [x] Strategy picking in `Match` mode using `:strategy`
  - [ ] Completions
- [ ] Enhanced `Info` mode, that englobes `Databases` and `Strategies`, and other informations
- [ ] `Command` mode, to send raw commands
- [ ] Multiple servers, but always in 0-config mode, that is only specified from the command line


## Compliance

This tool is (for now) partly [RFC 2229](https://tools.ietf.org/html/rfc2229)
compliant :

- [x] `DEFINE`
- [x] `MATCH`
- [x] `SHOW`
  - [x] `DATABASES`
  - [x] `STRATEGIES`
  - [ ] `INFO`
  - [ ] `SERVER`
- [x] `CLIENT`
- [ ] `STATUS`
- [ ] `QUIT`
- [ ] `OPTION MIME` (and it will never be)
- [ ] `AUTH`
- [ ] SASL

For now, this tool does not read `dict` URLs, either, but will in the future.
It also does not pipelines the requests, as this is not needed in interactive use.
