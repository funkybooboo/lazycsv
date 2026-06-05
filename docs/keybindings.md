# LazyCSV Keybindings Reference

LazyCSV features a fully customizable, data-driven keybinding system. While it ships with a **Vim-First** philosophy by default, every action can be remapped.

## Customizing Keybindings

Keybindings are configured in `~/.config/lazycsv/keys.toml`.

### Key Sequence Syntax

Sequences are concatenations of *atoms*. Whitespace separates atoms in a multi-key chord.

- `j` — Single character (lowercase)
- `J` — Shift+J (uppercase letters auto-lift Shift)
- `gg` — Two-key chord
- `ctrl+s` — Modifier + key
- `<esc>` — Reserved key name
- `ctrl+shift+f` — Multiple modifiers

**Modifiers:** `ctrl`, `shift`, `alt`, `super`

**Reserved Keys:** `<esc>`, `<enter>`, `<tab>`, `<bs>`, `<del>`, `<space>`, `<up>`, `<down>`, `<left>`, `<right>`, `<home>`, `<end>`, `<pgup>`, `<pgdn>`, `<f1>`..`<f12>`

### Example `keys.toml`

```toml
[meta]
inherit = "vim" # Start with vim defaults

[normal]
# Remap save to Ctrl+S
"ctrl+s" = "save"

# Unbind a default key
"q" = ""

# Map a chord
"<space>f" = "enter_file_list"
```

## Action Catalog

Below are the named actions available for remapping, grouped by mode.

### Global Actions
*Can be bound in any mode-scoped section or the `[global]` section.*

- `quit` — Quit (checks for unsaved changes)
- `quit_force` — Quit without saving
- `save` — Save current file
- `save_quit` — Save and quit
- `reload_file` — Force reload from disk
- `toggle_help` — Show/hide help overlay

### Normal Mode (`[normal]`)

- `cursor_up`, `cursor_down`, `cursor_left`, `cursor_right`
- `cursor_word_forward`, `cursor_word_backward`, `cursor_word_end`
- `goto_first_row`, `goto_last_row`, `goto_first_column`, `goto_last_column`
- `page_up`, `page_down`, `half_page_up`, `half_page_down`
- `viewport_top`, `viewport_center`, `viewport_bottom`
- `cell_edit_at_end`, `cell_edit_at_start`, `cell_edit_at_line_end`
- `cell_replace`, `cell_replace_f2`, `cell_clear`
- `toggle_case`, `title_case`, `toggle_boolean`
- `undo`, `redo`, `repeat_last_edit`
- `row_insert_below`, `row_insert_above`, `row_delete`, `row_yank`, `row_paste_below`, `row_paste_above`
- `col_delete`, `col_yank`, `col_paste_right`, `col_paste_left`, `col_insert_right`, `col_insert_left`
- `enter_command_mode`, `enter_visual_block`, `enter_visual_line`, `enter_visual_column`
- `enter_sql_editor`, `enter_magnifier`, `enter_file_list`

### Insert Mode (`[insert]`)

- `insert_commit_down`, `insert_commit_up`, `insert_commit_tab`, `insert_commit_back_tab`
- `insert_cancel`, `insert_cursor_left`, `insert_cursor_right`, `insert_delete_backward`, `insert_delete_forward`

### Visual Mode (`[visual]`)

- `visual_exit`, `visual_cursor_up`, `visual_cursor_down`, `visual_cursor_left`, `visual_cursor_right`
- `visual_delete`, `visual_yank`, `visual_paste`, `visual_stats`

### Magnifier Mode (`[magnifier]`)

- `mag_exit`, `mag_save_and_close`, `mag_enter_insert`, `mag_enter_command`, `mag_enter_visual`
- `mag_navigate_up`, `mag_navigate_down`, `mag_navigate_left`, `mag_navigate_right`
- `mag_delete_line`, `mag_yank_line`, `mag_indent_right`, `mag_indent_left`

### SQL Editor Mode (`[sql_editor]`)

- `sql_exit`, `sql_execute`, `sql_format`, `sql_context_completion`, `sql_history_popup_open`

## Preset Profiles

LazyCSV ships with built-in presets under the `keymaps/` directory:

1. **vim.toml** (Default) — Standard vim bindings.
2. **excel.toml** — Arrow keys, `F2` to edit, `Ctrl+C/V/X` for clipboard.
3. **emacs.toml** — Readline-style `Ctrl-f/b/n/p` navigation.
4. **modeless.toml** — Minimal bindings for plain typing and arrow navigation.

To use a preset, copy it to your config location:
`cp keymaps/excel.toml ~/.config/lazycsv/keys.toml`
