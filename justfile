# Patterns for files that go into the prompt
include := "*.rs,*.slint,Cargo.toml,build.rs"
exclude := "target/*,*.sqlite3"

# Show available recipes
default:
    @just --list

# Write the whole project to prompt.md
prompt:
    code2prompt . --include "{{include}}" --exclude "{{exclude}}" --output-file prompt.md

# Only the UI files, for slint questions
prompt-ui:
    code2prompt . --include "*.slint,build.rs" --exclude "{{exclude}}" --output-file prompt-ui.md

# Only the Rust sources of one directory, e.g. `just prompt-dir src`
prompt-dir dir:
    code2prompt {{dir}} --include "*.rs" --exclude "{{exclude}}" --output-file prompt-{{dir}}.md
