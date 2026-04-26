# LazyCSV Keymaps

Drop-in `keys.toml` files that re-bind LazyCSV's keys to match a familiar
editor's muscle memory. Pick one, copy it into place, and restart (or just
save — LazyCSV hot-reloads).

## Available Presets

| File | Description |
|------|-------------|
| `vim.toml` | The default — vim-style modal editing. Built into the binary. |
| `emacs.toml` | Readline-style: `Ctrl-f/b/n/p`, `Ctrl-a/e`, `Ctrl-w/u`, `Alt-x`, … |
| `excel.toml` | Arrow-key navigation, `F2` to edit, `Ctrl-S/Z/Y`, `Tab`/`Enter` for data entry. |

## How to install

`keys.toml` lives at `~/.config/lazycsv/keys.toml` (macOS / Linux) or
`%APPDATA%\lazycsv\keys.toml` (Windows). To use a preset, copy the file
there:

```sh
cp keymaps/emacs.toml ~/.config/lazycsv/keys.toml
```

LazyCSV picks it up on the next launch — and on every save thereafter, via
the existing config watcher.

If you only want to override a few bindings on top of the default, make a
small `keys.toml` of your own:

```toml
[meta]
inherit = "vim"   # default; layer your overrides on top of vim

[normal]
"ctrl+s" = "save"
"ctrl+f" = "search_enter"
```

To start from a blank slate (no inherited bindings), set
`[meta] inherit = "none"` and bind every key you want explicitly.

## Authoring your own preset

The full action catalog lives in
[`src/input/keymap_actions.rs`](../src/input/keymap_actions.rs). Every
variant has a stable `snake_case` ID — those are what go on the right-hand
side of a binding. Run `:keys` from inside lazycsv to see how many
bindings are active and where to edit them.

### Sequence syntax

```
"j"             single keypress
"J"             Shift-J (uppercase auto-lifts Shift)
"gg"            chord: g then g
",dd"           chord: comma, d, d
"ctrl+s"        Ctrl-s (modifiers: ctrl, shift, alt, super)
"<esc>"         reserved key — esc, enter, tab, bs, space, del,
                up, down, left, right, home, end, pgup, pgdn, f1..f12
"ctrl+<enter>"  Ctrl-Enter
"ctrl+x ctrl+s" multi-atom chord (whitespace separates atoms)
""              (empty value) unbind the key
```

### Per-mode sections

Each preset is split by mode:

| Section | Active when |
|---------|-------------|
| `[normal]` | the normal mode (default) |
| `[insert]` | the cell-edit mode (after `i`/`a`/`F2`) |
| `[visual]` | visual block / line / column |
| `[command]` | the `:` ex-command line |
| `[search]` | the `/` search line |
| `[magnifier]` | full-cell magnifier mode |
| `[file_list]` | the file menu (`<space>f`) |
| `[sql_editor]` | the SQL editor (`<space>q`) |
| `[file_operation]` | the rename/move/delete prompts |
| `[global]` | falls back when no mode-specific binding matches |

## Phase 1 vs. Phase 2

LazyCSV v0.24.1 ships **Phase 1** keymap support:

- ✅ Single-key bindings work in Normal and Insert modes via the keymap
- ✅ Multi-key chords (e.g. `gg`, `dd`, `,yy`) still work via the legacy
  vim chord path — they aren't user-rebindable yet, but they aren't broken
- ⏳ **Phase 2** will pull chord bindings, visual/command/sql/magnifier/
  file-menu modes through the keymap as well, plus add the `:keys` popup
  with the full searchable action ↔ key table

If you bind a chord in `keys.toml` today (e.g. `"gg" = "cursor_left"`), the
binding is **parsed and stored** but won't fire until Phase 2 lands. The
keymap loader will not warn about it — the binding becomes live the moment
Phase 2 ships.
