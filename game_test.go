package main

import (
	"math/rand"
	"os"
	"path/filepath"
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
)

func TestEvaluate(t *testing.T) {
	cases := []struct {
		name   string
		target string
		guess  string
		want   [wordLen]LetterState
	}{
		{
			name:   "all correct",
			target: "CRANE",
			guess:  "CRANE",
			want:   [wordLen]LetterState{StateCorrect, StateCorrect, StateCorrect, StateCorrect, StateCorrect},
		},
		{
			name:   "ROBOT vs SPEED all absent",
			target: "ROBOT",
			guess:  "SPEED",
			want:   [wordLen]LetterState{StateAbsent, StateAbsent, StateAbsent, StateAbsent, StateAbsent},
		},
		{
			name:   "ROBOT vs BOOTS",
			target: "ROBOT",
			guess:  "BOOTS",
			want:   [wordLen]LetterState{StatePresent, StateCorrect, StatePresent, StatePresent, StateAbsent},
		},
		{
			name:   "APPLE vs POPPY excess duplicate is gray",
			target: "APPLE",
			guess:  "POPPY",
			want:   [wordLen]LetterState{StatePresent, StateAbsent, StateCorrect, StateAbsent, StateAbsent},
		},
		{
			name:   "SPEED vs EERIE",
			target: "SPEED",
			guess:  "EERIE",
			want:   [wordLen]LetterState{StatePresent, StatePresent, StateAbsent, StateAbsent, StateAbsent},
		},
		{
			name:   "CRANE vs SLATE",
			target: "CRANE",
			guess:  "SLATE",
			want:   [wordLen]LetterState{StateAbsent, StateAbsent, StateCorrect, StateAbsent, StateCorrect},
		},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if got := evaluate(tc.target, tc.guess); got != tc.want {
				t.Fatalf("evaluate(%q, %q) = %v, want %v", tc.target, tc.guess, got, tc.want)
			}
		})
	}
}

func TestUpdateKeyboardNeverDowngrades(t *testing.T) {
	var kb [26]LetterState
	updateKeyboard(&kb, "EERIE", evaluate("SPEED", "EERIE"))
	if kb['E'-'A'] != StatePresent {
		t.Fatalf("E = %v, want Present", kb['E'-'A'])
	}
	updateKeyboard(&kb, "EXCEL", [wordLen]LetterState{StateAbsent, StateAbsent, StateAbsent, StateAbsent, StateAbsent})
	if kb['E'-'A'] != StatePresent {
		t.Fatalf("E after absent guess = %v, want still Present", kb['E'-'A'])
	}
	updateKeyboard(&kb, "EERIE", evaluate("EERIE", "EERIE"))
	if kb['E'-'A'] != StateCorrect {
		t.Fatalf("E after correct guess = %v, want Correct", kb['E'-'A'])
	}
}

func TestParseWordsJSON(t *testing.T) {
	objs := []string{
		`{"words":["crane","slate","crane","toolong","ab"]}`,
		`["crane","slate"]`,
	}
	for _, s := range objs {
		words, err := parseWordsJSON([]byte(s))
		if err != nil {
			t.Fatalf("parseWordsJSON(%q): %v", s, err)
		}
		got := normaliseWords(words)
		if len(got) == 0 || got[0] != "CRANE" {
			t.Fatalf("parseWordsJSON(%q) normalised = %v", s, got)
		}
	}
	got := normaliseWords(mustParseJSON(t, `{"words":["crane","slate","crane","toolong","ab"]}`))
	if len(got) != 2 || got[0] != "CRANE" || got[1] != "SLATE" {
		t.Fatalf("dedupe/filter = %v, want [CRANE SLATE]", got)
	}
}

func mustParseJSON(t *testing.T, s string) []string {
	t.Helper()
	words, err := parseWordsJSON([]byte(s))
	if err != nil {
		t.Fatalf("parseWordsJSON: %v", err)
	}
	return words
}

func TestLoadDictionaryFromTempFiles(t *testing.T) {
	dir := t.TempDir()

	jsonPath := filepath.Join(dir, "w.json")
	if err := os.WriteFile(jsonPath, []byte(`{"words":["apple","robot","speed"]}`), 0o644); err != nil {
		t.Fatal(err)
	}
	if words, err := loadWordFile(jsonPath); err != nil || len(words) != 3 {
		t.Fatalf("loadWordFile(json) = %v, %v", words, err)
	}

	txtPath := filepath.Join(dir, "w.txt")
	if err := os.WriteFile(txtPath, []byte("crane\nslate\ntrace\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	d := buildDictionary(parseWordsTXT(mustRead(t, txtPath)), nil, "test.txt")
	if len(d.valid) != 3 || d.source != "test.txt" {
		t.Fatalf("dictionary = %+v", d)
	}
	if _, ok := d.valid["CRANE"]; !ok {
		t.Fatalf("CRANE not in valid set: %v", sortedKeys(d.valid))
	}
}

func TestDictionaryAnswersVsGuessesSplit(t *testing.T) {
	answers := []string{"CRANE", "SLATE"}
	guesses := []string{"AAHED", "ABACA"}
	d := buildDictionary(answers, guesses, "split")

	if len(d.answers) != 2 || d.answers[0] != "CRANE" || d.answers[1] != "SLATE" {
		t.Fatalf("answers pool = %v, want [CRANE SLATE]", d.answers)
	}
	for _, w := range []string{"CRANE", "SLATE", "AAHED", "ABACA"} {
		if _, ok := d.valid[w]; !ok {
			t.Fatalf("%s should be valid (union of answers + guesses): %v", w, sortedKeys(d.valid))
		}
	}
	if len(d.all) != 4 {
		t.Fatalf("union size = %d, want 4: %v", len(d.all), d.all)
	}
	if _, ok := d.valid["CRANE"]; !ok {
		t.Fatal("CRANE (answer not in guesses) should still be guessable")
	}
	rng := rand.New(rand.NewSource(42))
	for i := 0; i < 200; i++ {
		got := pickTarget(d, rng, "")
		if got != "CRANE" && got != "SLATE" {
			t.Fatalf("pickTarget returned guesses-only word %q", got)
		}
	}
}

func TestPickTargetUsesAnswersOnly(t *testing.T) {
	answers := []string{"CRANE", "SLATE", "TRACE"}
	guesses := []string{"AAHED", "ABACA", "ABACI"}
	d := buildDictionary(answers, guesses, "split")
	if got := pickTarget(d, rand.New(rand.NewSource(1)), "robot"); got != "ROBOT" {
		t.Fatalf("forced target = %q, want ROBOT", got)
	}
	rng := rand.New(rand.NewSource(7))
	seen := map[string]bool{}
	for i := 0; i < 500; i++ {
		got := pickTarget(d, rng, "")
		seen[got] = true
		for _, a := range answers {
			if got == a {
				goto ok
			}
		}
		t.Fatalf("pickTarget returned %q, not in answers %v", got, answers)
	ok:
	}
	if len(seen) < 2 {
		t.Fatalf("pickTarget only ever returned %v, want variety from answers", seen)
	}
}

func mustRead(t *testing.T, path string) []byte {
	t.Helper()
	b, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	return b
}

func TestEmbeddedAndFallbackListsAreUsable(t *testing.T) {
	d := buildDictionary(fallbackWords, nil, "fallback")
	if len(d.all) != len(fallbackWords) {
		t.Fatalf("fallback: got %d normalised words from %d entries (some entries are not 5 letters)", len(d.all), len(fallbackWords))
	}
	if len(d.answers) != len(fallbackWords) {
		t.Fatalf("fallback answers = %d, want %d", len(d.answers), len(fallbackWords))
	}
	ed := buildDictionary(parseWordsTXT([]byte(embeddedAnswers)), parseWordsTXT([]byte(embeddedGuesses)), "embedded")
	if len(ed.answers) < 1000 {
		t.Fatalf("embedded answers parsed to only %d words", len(ed.answers))
	}
	if len(ed.valid) < 5000 {
		t.Fatalf("embedded union (answers + guesses) parsed to only %d words", len(ed.valid))
	}
	if _, ok := ed.valid["CRANE"]; !ok {
		t.Fatal("CRANE missing from embedded union")
	}
	if _, ok := ed.valid["AAHED"]; !ok {
		t.Fatal("AAHED missing from embedded union")
	}
	foundCrane := false
	for _, w := range ed.answers {
		if w == "CRANE" {
			foundCrane = true
		}
	}
	if !foundCrane {
		t.Fatal("CRANE missing from embedded answers pool")
	}
}

func TestLoadDictionarySplitFromTempFiles(t *testing.T) {
	dir := t.TempDir()
	answersPath := filepath.Join(dir, "tuidle-answers.json")
	if err := os.WriteFile(answersPath, []byte(`{"words":["crane","slate"]}`), 0o644); err != nil {
		t.Fatal(err)
	}
	guessesPath := filepath.Join(dir, "tuidle-allowed-guesses.txt")
	if err := os.WriteFile(guessesPath, []byte("aahed\nabaca\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	answersRaw, aSrc, okA := loadOneList(answersPath, answersJSONName, answersTXTName, "")
	guessesRaw, gSrc, okG := loadOneList(guessesPath, guessesJSONName, guessesTXTName, "")
	if !okA || !okG || aSrc != answersPath || gSrc != guessesPath {
		t.Fatalf("loadOneList explicit: (%q,%v) (%q,%v)", aSrc, okA, gSrc, okG)
	}
	d := buildDictionary(answersRaw, guessesRaw, aSrc+" + "+gSrc)
	if len(d.answers) != 2 {
		t.Fatalf("answers = %v, want 2", d.answers)
	}
	for _, w := range []string{"CRANE", "SLATE", "AAHED", "ABACA"} {
		if _, ok := d.valid[w]; !ok {
			t.Fatalf("%s should be valid", w)
		}
	}
}

func testModel(target string) model {
	answers := []string{"CRANE", "SLATE", "ROBOT", "SPEED", "APPLE", "POPPY", "EERIE", "BOOTS", "TRACE", "CRATE"}
	dict := buildDictionary(answers, answers, "test")
	return newModel(dict, rand.New(rand.NewSource(1)), target)
}

func keyMsg(s string) tea.KeyMsg {
	if s == "enter" {
		return tea.KeyMsg{Type: tea.KeyEnter}
	}
	if s == "backspace" {
		return tea.KeyMsg{Type: tea.KeyBackspace}
	}
	if s == "ctrl+c" {
		return tea.KeyMsg{Type: tea.KeyCtrlC}
	}
	if s == "esc" {
		return tea.KeyMsg{Type: tea.KeyEsc}
	}
	if len(s) == 1 {
		return tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune{rune(s[0])}}
	}
	switch s {
	case "up":
		return tea.KeyMsg{Type: tea.KeyUp}
	case "down":
		return tea.KeyMsg{Type: tea.KeyDown}
	case "left":
		return tea.KeyMsg{Type: tea.KeyLeft}
	case "right":
		return tea.KeyMsg{Type: tea.KeyRight}
	}
	return tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune(s)}
}

func typeWord(t *testing.T, m model, word string) model {
	t.Helper()
	for _, r := range word {
		next, _ := m.Update(keyMsg(string(r)))
		m = next.(model)
	}
	next, _ := m.Update(keyMsg("enter"))
	return next.(model)
}

func TestModelWinAndRestart(t *testing.T) {
	m := testModel("CRANE")
	m = typeWord(t, m, "CRANE")
	if !m.won || len(m.guesses) != 1 {
		t.Fatalf("after correct guess: won=%v guesses=%v status=%q", m.won, m.guesses, m.status.text)
	}
	if !strings.Contains(m.status.text, "won") {
		t.Fatalf("win status = %q", m.status.text)
	}
	next, _ := m.Update(keyMsg("r"))
	m = next.(model)
	if m.won || len(m.guesses) != 0 || m.target != "CRANE" {
		t.Fatalf("after restart: won=%v guesses=%v target=%q", m.won, m.guesses, m.target)
	}
	m = typeWord(t, m, "CRANE")
	next, _ = m.Update(keyMsg("enter"))
	if next.(model).won {
		t.Fatal("Enter after win should restart")
	}
}

func TestModelRejectsInvalidWordAndShortGuess(t *testing.T) {
	m := testModel("CRANE")
	m = typeWord(t, m, "ZZZZZ")
	if len(m.guesses) != 0 {
		t.Fatalf("invalid guess was submitted: %v", m.guesses)
	}
	if m.status.text != "Not in word list" {
		t.Fatalf("status = %q, want 'Not in word list'", m.status.text)
	}
	if m.shakeLeft == 0 {
		t.Fatal("expected board shake after invalid word")
	}

	m = testModel("CRANE")
	for _, r := range "CRA" {
		next, _ := m.Update(keyMsg(string(r)))
		m = next.(model)
	}
	next, _ := m.Update(keyMsg("enter"))
	m = next.(model)
	if len(m.guesses) != 0 || m.status.text != "Not enough letters" {
		t.Fatalf("short guess: guesses=%v status=%q", m.guesses, m.status.text)
	}
}

func TestModelBackspaceArrowKeysAndLetterLimit(t *testing.T) {
	m := testModel("CRANE")
	for _, r := range "CRANEX" { // 6th letter must be ignored
		next, _ := m.Update(keyMsg(string(r)))
		m = next.(model)
	}
	if m.current != "CRANE" {
		t.Fatalf("current = %q, want CRANE (5-letter limit)", m.current)
	}
	for _, k := range []string{"up", "down", "left", "right"} {
		next, _ := m.Update(keyMsg(k))
		m = next.(model)
	}
	if m.current != "CRANE" {
		t.Fatalf("arrow keys changed current to %q", m.current)
	}
	next, _ := m.Update(keyMsg("backspace"))
	m = next.(model)
	if m.current != "CRAN" {
		t.Fatalf("after backspace current = %q", m.current)
	}
}

func TestModelRejectsDuplicateGuess(t *testing.T) {
	m := testModel("CRANE")
	m = typeWord(t, m, "SLATE")
	if len(m.guesses) != 1 {
		t.Fatalf("first guess: guesses=%v", m.guesses)
	}
	m = typeWord(t, m, "SLATE")
	if len(m.guesses) != 1 {
		t.Fatalf("duplicate guess was submitted: %v", m.guesses)
	}
	if m.status.text != "Already guessed" {
		t.Fatalf("status = %q, want 'Already guessed'", m.status.text)
	}
	if m.shakeLeft == 0 {
		t.Fatal("expected board shake after duplicate guess")
	}
	if m.won || m.lost {
		t.Fatalf("duplicate guess must not end the game: won=%v lost=%v", m.won, m.lost)
	}
}

func TestModelLosesAfterSix(t *testing.T) {
	m := testModel("CRANE")
	for _, w := range []string{"SLATE", "TRACE", "CRATE", "ROBOT", "SPEED", "APPLE"} {
		m = typeWord(t, m, w)
	}
	if !m.lost || len(m.guesses) != 6 {
		t.Fatalf("lost=%v guesses=%d", m.lost, len(m.guesses))
	}
	if !strings.Contains(m.status.text, "CRANE") {
		t.Fatalf("loss status should reveal target: %q", m.status.text)
	}
}

func TestModelQuit(t *testing.T) {
	m := testModel("CRANE")
	for _, k := range []string{"ctrl+c", "esc"} {
		_, cmd := m.Update(keyMsg(k))
		if cmd == nil {
			t.Fatalf("%s did not return a quit command", k)
		}
		if msg := cmd(); msg != (tea.QuitMsg{}) {
			t.Fatalf("%s cmd() = %#v, want tea.QuitMsg", k, msg)
		}
	}
}

func TestViewRendersBoardKeyboardAndStatus(t *testing.T) {
	m := testModel("CRANE")
	m.current = "CRA"
	view := stripANSI(m.View())
	for _, want := range []string{"TUIDLE", "C", "R", "A", "[Q]", "[W]", "[P]", "[A]", "[L]", "[Z]", "[M]", "Type a guess"} {
		if !strings.Contains(view, want) {
			t.Fatalf("View() missing %q:\n%s", want, view)
		}
	}
	if strings.Contains(view, "|") {
		t.Fatalf("View() must not contain '|':\n%s", view)
	}
	if strings.Count(view, "[") < 6*5 {
		t.Fatalf("expected at least %d bracketed tiles:\n%s", 6*5, view)
	}
}

func stripANSI(s string) string {
	var b strings.Builder
	inEsc := false
	for _, r := range s {
		switch {
		case r == '\x1b':
			inEsc = true
		case inEsc && ((r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z')):
			inEsc = false
		case !inEsc:
			b.WriteRune(r)
		}
	}
	return b.String()
}
