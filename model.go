package main

import (
	"fmt"
	"math/rand"
	"strings"
	"time"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type statusKind int

const (
	statusNone statusKind = iota
	statusInfo
	statusWarn
	statusWin
	statusLose
)

type statusMessage struct {
	text   string
	kind   statusKind
	id     int
	sticky bool
}

type statusClearMsg struct{ id int }
type shakeTickMsg struct{ id int }

type model struct {
	dict         dictionary
	rng          *rand.Rand
	forcedTarget string

	target   string
	guesses  []string
	feedback [maxGuesses][wordLen]LetterState

	current  string
	keyboard [26]LetterState

	won  bool
	lost bool

	status    statusMessage
	statusSeq int

	shakeID   int
	shakeLeft int

	width  int
	height int

	help help.Model
}

var (
	helpType    = key.NewBinding(key.WithKeys("a-z"), key.WithHelp("a-z", "type"))
	helpEnter   = key.NewBinding(key.WithKeys("enter"), key.WithHelp("enter", "submit"))
	helpBack    = key.NewBinding(key.WithKeys("backspace"), key.WithHelp("backspace", "delete"))
	helpQuit    = key.NewBinding(key.WithKeys("ctrl+c", "esc"), key.WithHelp("ctrl+c / esc", "quit"))
	helpRestart = key.NewBinding(key.WithKeys("r", "enter"), key.WithHelp("r / enter", "restart"))
)

var _ tea.Model = model{}

func newModel(dict dictionary, rng *rand.Rand, forcedTarget string) model {
	if rng == nil {
		rng = rand.New(rand.NewSource(time.Now().UnixNano()))
	}
	h := help.New()
	h.Styles.ShortKey = lipgloss.NewStyle().Foreground(footerFg)
	h.Styles.ShortDesc = lipgloss.NewStyle().Foreground(footerFg)
	h.Styles.ShortSeparator = lipgloss.NewStyle().Foreground(footerFg)
	h.ShortSeparator = "  "
	m := model{
		dict:         dict,
		rng:          rng,
		forcedTarget: strings.ToUpper(forcedTarget),
		help:         h,
	}
	return m.reset()
}

func (m model) reset() model {
	m.target = pickTarget(m.dict, m.rng, m.forcedTarget)
	m.guesses = nil
	m.feedback = [maxGuesses][wordLen]LetterState{}
	m.current = ""
	m.keyboard = [26]LetterState{}
	m.won = false
	m.lost = false
	m.status = statusMessage{}
	m.shakeLeft = 0
	m.shakeID++
	return m
}

func (m model) Init() tea.Cmd { return nil }

func (m model) setTransient(text string, kind statusKind, shake bool) (model, tea.Cmd) {
	m.statusSeq++
	id := m.statusSeq
	m.status = statusMessage{text: text, kind: kind, id: id}

	var cmds []tea.Cmd
	cmds = append(cmds, tea.Tick(1500*time.Millisecond, func(t time.Time) tea.Msg {
		return statusClearMsg{id: id}
	}))
	if shake {
		m.shakeID++
		sid := m.shakeID
		m.shakeLeft = 6
		cmds = append(cmds, tea.Tick(50*time.Millisecond, func(t time.Time) tea.Msg {
			return shakeTickMsg{id: sid}
		}))
	}
	return m, tea.Batch(cmds...)
}

func (m model) setSticky(text string, kind statusKind) model {
	m.statusSeq++
	m.status = statusMessage{text: text, kind: kind, id: m.statusSeq, sticky: true}
	return m
}

func (m model) hasGuessed(word string) bool {
	for _, g := range m.guesses {
		if g == word {
			return true
		}
	}
	return false
}

func (m model) submit() (model, tea.Cmd) {
	guess := m.current
	if len(guess) < wordLen {
		return m.setTransient("Not enough letters", statusWarn, true)
	}
	if _, ok := m.dict.valid[guess]; !ok {
		return m.setTransient("Not in word list", statusWarn, true)
	}
	if m.hasGuessed(guess) {
		return m.setTransient("Already guessed", statusWarn, true)
	}

	row := len(m.guesses)
	fb := evaluate(m.target, guess)
	m.feedback[row] = fb
	updateKeyboard(&m.keyboard, guess, fb)
	m.guesses = append(m.guesses, guess)
	m.current = ""
	if !m.status.sticky {
		m.status = statusMessage{}
	}

	switch {
	case guess == m.target:
		m.won = true
		m = m.setSticky(
			fmt.Sprintf("%s You won in %d %s!", winMessage(len(m.guesses)), len(m.guesses), plural(len(m.guesses), "guess", "guesses")),
			statusWin,
		)
	case len(m.guesses) >= maxGuesses:
		m.lost = true
		m = m.setSticky(fmt.Sprintf("Game over! The word was %s", m.target), statusLose)
	}
	return m, nil
}

func plural(n int, one, many string) string {
	if n == 1 {
		return one
	}
	return many
}

func (m model) gameOver() bool { return m.won || m.lost }

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		m.help.Width = msg.Width
		return m, nil

	case statusClearMsg:
		if m.status.id == msg.id && !m.status.sticky {
			m.status = statusMessage{}
		}
		return m, nil

	case shakeTickMsg:
		if msg.id != m.shakeID {
			return m, nil
		}
		if m.shakeLeft > 0 {
			m.shakeLeft--
			if m.shakeLeft > 0 {
				sid := m.shakeID
				return m, tea.Tick(50*time.Millisecond, func(t time.Time) tea.Msg {
					return shakeTickMsg{id: sid}
				})
			}
		}
		return m, nil

	case tea.KeyMsg:
		return m.handleKey(msg)
	}
	return m, nil
}

func (m model) handleKey(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	k := msg.String()

	switch k {
	case "ctrl+c", "esc":
		return m, tea.Quit
	}

	if m.gameOver() {
		if k == "r" || k == "R" || k == "enter" {
			return m.reset(), nil
		}
		return m, nil
	}

	switch k {
	case "enter":
		return m.submit()

	case "backspace", "ctrl+h":
		if len(m.current) > 0 {
			m.current = m.current[:len(m.current)-1]
		}
		return m, nil

	case "up", "down", "left", "right", "home", "end", "pgup", "pgdown", "tab", "space":
		return m, nil
	}

	if len(k) == 1 {
		c := k[0]
		if c >= 'a' && c <= 'z' {
			c = c - 'a' + 'A'
		}
		if c >= 'A' && c <= 'Z' && len(m.current) < wordLen {
			m.current += string(c)
			if !m.status.sticky {
				m.status = statusMessage{}
			}
		}
	}
	return m, nil
}

func (m model) View() string {
	alignWidth := contentWidth()
	if m.width > alignWidth {
		alignWidth = m.width
	}
	center := lipgloss.NewStyle().Width(alignWidth).Align(lipgloss.Center)

	board := center.Render(m.renderBoard())
	keyboard := m.renderKeyboard(alignWidth)

	shake := 0
	if m.shakeLeft > 0 {
		offsets := []int{0, 2, 0, 1, 0, 2, 0}
		if m.shakeLeft < len(offsets) {
			shake = offsets[m.shakeLeft]
		}
	}
	shift := lipgloss.NewStyle().MarginLeft(shake)

	var b strings.Builder
	b.WriteString(titleStyle.Width(alignWidth).Render("TUIDLE"))
	b.WriteString("\n")
	b.WriteString(subtitleStyle.Width(alignWidth).Render("Guess the 5-letter word in 6 tries"))
	b.WriteString("\n\n")
	b.WriteString(shift.Render(board))
	b.WriteString("\n\n")
	b.WriteString(shift.Render(keyboard))
	b.WriteString("\n\n")
	b.WriteString(m.renderStatus(alignWidth))
	b.WriteString("\n")
	b.WriteString(footerStyle.Width(alignWidth).Render(m.renderHelp()))
	if m.dict.source != "" {
		b.WriteString("\n")
		b.WriteString(footerStyle.Width(alignWidth).Render(
			fmt.Sprintf("Word list: %s (%d words)", m.dict.source, len(m.dict.all))))
	}

	content := b.String()
	if m.width > 0 && m.height > 0 {
		return lipgloss.Place(m.width, m.height, lipgloss.Center, lipgloss.Center, content)
	}
	return content
}

func (m model) renderHelp() string {
	bindings := []key.Binding{helpType, helpEnter, helpBack, helpQuit}
	if m.gameOver() {
		bindings = []key.Binding{helpRestart, helpQuit}
	}
	return m.help.ShortHelpView(bindings)
}

func (m model) renderStatus(alignWidth int) string {
	text := m.status.text
	if text == "" {
		if m.gameOver() {
			text = "Press R or Enter to play again"
		} else {
			text = "Type a guess, then press Enter"
		}
	}
	switch m.status.kind {
	case statusWin:
		return winStyle.Width(alignWidth).Render(text)
	case statusLose:
		return loseStyle.Width(alignWidth).Render(text)
	case statusWarn:
		return warnStyle.Width(alignWidth).Render(text)
	default:
		return infoStyle.Width(alignWidth).Render(text)
	}
}

func boardWidth() int { return wordLen*tileWidth + (wordLen - 1) }

func contentWidth() int { return 10*tileWidth + 9 }

func (m model) renderBoard() string {
	rows := make([]string, 0, maxGuesses)
	for r := 0; r < maxGuesses; r++ {
		tiles := make([]string, wordLen)
		for c := 0; c < wordLen; c++ {
			letter, state := m.cell(r, c)
			tiles[c] = renderTile(letter, state)
		}
		rows = append(rows, strings.Join(tiles, " "))
	}
	return strings.Join(rows, "\n")
}

func (m model) cell(row, col int) (byte, LetterState) {
	if row < len(m.guesses) {
		g := m.guesses[row]
		return g[col], m.feedback[row][col]
	}
	if row == len(m.guesses) && !m.gameOver() {
		if col < len(m.current) {
			return m.current[col], StateCurrent
		}
		return ' ', StateActiveEmpty
	}
	return ' ', StateFuture
}

func (m model) renderKeyboard(alignWidth int) string {
	rowsDef := []string{"QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"}
	lines := make([]string, 0, len(rowsDef))
	for _, row := range rowsDef {
		keys := make([]string, 0, len(row))
		for i := 0; i < len(row); i++ {
			ch := row[i]
			state := m.keyboard[ch-'A']
			keys = append(keys, renderTile(ch, state))
		}
		line := strings.Join(keys, " ")
		line = lipgloss.NewStyle().Width(alignWidth).Align(lipgloss.Center).Render(line)
		lines = append(lines, line)
	}
	return strings.Join(lines, "\n")
}
