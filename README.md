# TUIDLE

Wordle clone in your terminal.

## Requirements

- Go 1.24.2 or newer (`go version`)

## Build & run

```sh
# Just run it
go run .

# Or build the binary:
go build -o tuidle .        # Linux / macOS
go build -o tuidle.exe .    # Windows

./tuidle                    # Linux / macOS
.\tuidle.exe                # Windows
```

### Tests

```sh
go test ./... -v
go vet ./...
```

### Cross-compiling

```sh
# On Linux/macOS
GOOS=linux   GOARCH=amd64 go build -o tuidle .                      # Linux target
GOOS=darwin  GOARCH=arm64 go build -o tuidle .                      # macOS target
GOOS=windows GOARCH=amd64 go build -o tuidle.exe .                  # Windows target

# On Windows
$env:GOOS="linux"; $env:GOARCH="amd64"; go build -o tuidle .        # Linux target
$env:GOOS="darwin"; $env:GOARCH="arm64"; go build -o tuidle .       # macOS target
$env:GOOS="windows"; $env:GOARCH="amd64"; go build -o tuidle.exe .  # Windows target
```

Optional flags:

```sh
go run . -answers path/to/answers.json  # custom answers/target pool
go run . -guesses path/to/guesses.txt   # custom allowed-guesses list
go run . -words   path/to/words.txt     # legacy: same path for both
go run . -target CRANE                  # force the answer (testing aid)
go run . -h                             # flag help
```

The binary must be run in a real terminal (a TTY). In a pipe or
redirected shell it exits with a hint instead of hanging.

### Code layout (Elm architecture)

| File | Contents |
|------|----------|
| `main.go` | Flags, dictionary loading, `tea.NewProgram(...).Run()` |
| `model.go` | `model` struct, `Init` / `Update` / `View`, keyboard handling, board + keyboard + status rendering, shake animation |
| `game.go` | Pure logic only: `evaluate` (repeated letters), keyboard precedence, win messages — unit-tested without a terminal |
| `words.go` | `tuidle-answers.*` (targets) + `tuidle-allowed-guesses.*` (validation) / embedded / fallback loading, random target picking |
| `styles.go` | Lipgloss colours and tile styles (green/yellow/gray/active) |
| `game_test.go` | Table-driven tests for scoring, parsing, submit/restart, quit, and view rendering |

`Update` handles `tea.WindowSizeMsg` (so a resize recentres the UI),
`tea.KeyMsg`, plus internal messages that clear transient warnings
after 1.5 s and drive the 50 ms board shake after an invalid guess.

### Cross-platform notes

- Bubbletea handles raw mode and ANSI input on Linux and Windows
  (via its Windows console support); Lipgloss/termenv automatically
  down-samples the hex tile colours to the terminal's colour profile
  (TrueColor → 256 → ANSI 16), so colours degrade gracefully instead
  of breaking.
- The UI uses only plain ASCII (solid ` X ` blocks plus `[X]` /
  `[ ]` brackets for not-yet tiles), so it renders correctly in
  legacy Windows code pages as well as modern terminals. There are
  no `|` outlines, which keeps the board from looking like a row of
  vertical stripes. Windows Terminal is still recommended for the
  best look.
- Recommended terminal size is at least 60×20; smaller terminals
  still work — the layout simply is not centred.
- No mouse support is enabled: input is keyboard-only, which keeps
  behaviour identical across terminals.

## Demo

Scored/typed tiles are solid colour blocks (` X `); not-yet
slots and unused keys are `[X]` / `[ ]` brackets. Colours are
omitted below — in a terminal the blocks are green / yellow /
gray / dark (see the Rules above).

```
                                TUIDLE
                  Guess the 5-letter word in 6 tries

                           P   L   E   A   D
                           S   H   I   F   T
                           F   A   T  [ ] [ ]
                          [ ] [ ] [ ] [ ] [ ]
                          [ ] [ ] [ ] [ ] [ ]
                          [ ] [ ] [ ] [ ] [ ]

               [Q] [W] [E] [R] [T] [Y] [U] [I] [O] [P]
                 [A] [S] [D] [F] [G] [H] [J] [K] [L]
                     [Z] [X] [C] [V] [B] [N] [M]

                    Type a guess, then press Enter
       a-z type  enter submit  backspace delete  ctrl+c / esc quit
Word list: tuidle-answers.json + tuidle-allowed-guesses.json (12972 words)
```
