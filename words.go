package main

import (
	_ "embed"
	"encoding/json"
	"math/rand"
	"os"
	"sort"
	"strings"
)

// Answers are the target pool; guesses are extra valid words.
// Valid = answers ∪ guesses, so an answer is always guessable.
// Per list, loading order: flag path, .json, .txt, embedded, fallback (answers only).

//go:embed tuidle-answers.txt
var embeddedAnswers string

//go:embed tuidle-allowed-guesses.txt
var embeddedGuesses string

var fallbackWords = []string{
	"CRANE", "SLATE", "TRACE", "CRATE", "AUDIO", "RAISE", "ARISE", "STARE",
	"ABOUT", "ABOVE", "ACTOR", "AFTER", "AGAIN", "AGENT", "AGREE", "AHEAD",
	"ALARM", "ALBUM", "ALERT", "ALIVE", "ALLOW", "ALONE", "ALONG", "ALTER",
	"AMONG", "ANGER", "ANGLE", "ANGRY", "ANKLE", "APART", "APPLE", "APPLY",
	"ARENA", "ARGUE", "ARMOR", "ARRAY", "ARROW", "ASIDE", "ATLAS", "ATONE",
	"AWAKE", "AWARE", "AWFUL", "AZURE", "BACON", "BADGE", "BADLY", "BAKER",
	"BALLS", "BANDS", "BANJO", "BANKS", "BARGE", "BARKS", "BARON", "BASIC",
	"BASIN", "BATCH", "BATON", "BEACH", "BEARD", "BEAST", "BEGIN", "BEING",
	"BELLY", "BELOW", "BENCH", "BERRY", "BIRTH", "BLACK", "BLADE", "BLAME",
	"BLANK", "BLAST", "BLAZE", "BLEND", "BLESS", "BLIND", "BLOCK", "BLOOD",
}

const (
	answersJSONName = "tuidle-answers.json"
	answersTXTName  = "tuidle-answers.txt"
	guessesJSONName = "tuidle-allowed-guesses.json"
	guessesTXTName  = "tuidle-allowed-guesses.txt"
)

type dictionary struct {
	valid   map[string]struct{}
	answers []string
	all     []string
	source  string
}

func normaliseWords(raw []string) []string {
	seen := make(map[string]struct{}, len(raw))
	out := make([]string, 0, len(raw))
	for _, w := range raw {
		w = strings.ToUpper(strings.TrimSpace(w))
		if !isFiveLetters(w) {
			continue
		}
		if _, dup := seen[w]; dup {
			continue
		}
		seen[w] = struct{}{}
		out = append(out, w)
	}
	return out
}

func buildDictionary(answersRaw, guessesRaw []string, source string) dictionary {
	answers := normaliseWords(answersRaw)
	guesses := normaliseWords(guessesRaw)

	valid := make(map[string]struct{}, len(answers)+len(guesses))
	all := make([]string, 0, len(answers)+len(guesses))
	for _, w := range answers {
		if _, dup := valid[w]; dup {
			continue
		}
		valid[w] = struct{}{}
		all = append(all, w)
	}
	for _, w := range guesses {
		if _, dup := valid[w]; dup {
			continue
		}
		valid[w] = struct{}{}
		all = append(all, w)
	}
	return dictionary{valid: valid, answers: append([]string(nil), answers...), all: all, source: source}
}

func parseWordsJSON(data []byte) ([]string, error) {
	trimmed := []byte(strings.TrimSpace(string(data)))
	if len(trimmed) == 0 {
		return nil, errEmpty
	}
	if trimmed[0] == '{' {
		var obj struct {
			Words []string `json:"words"`
		}
		if err := json.Unmarshal(trimmed, &obj); err != nil {
			return nil, err
		}
		return obj.Words, nil
	}
	var arr []string
	if err := json.Unmarshal(trimmed, &arr); err != nil {
		return nil, err
	}
	return arr, nil
}

func parseWordsTXT(data []byte) []string {
	return strings.FieldsFunc(string(data), func(r rune) bool {
		return r == '\n' || r == '\r' || r == ',' || r == ' ' || r == '\t'
	})
}

type simpleError string

func (e simpleError) Error() string { return string(e) }

const errEmpty = simpleError("empty word list")

func loadWordFile(path string) ([]string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	trimmed := strings.TrimSpace(string(data))
	if len(trimmed) > 0 && (trimmed[0] == '{' || trimmed[0] == '[') {
		return parseWordsJSON(data)
	}
	words := parseWordsTXT(data)
	if len(words) == 0 {
		return nil, errEmpty
	}
	return words, nil
}

func loadOneList(explicit, jsonName, txtName, embedded string) ([]string, string, bool) {
	if explicit != "" {
		if words, err := loadWordFile(explicit); err == nil {
			if len(normaliseWords(words)) > 0 {
				return words, explicit, true
			}
		}
	}
	for _, name := range []string{jsonName, txtName} {
		if _, err := os.Stat(name); err != nil {
			continue
		}
		if words, err := loadWordFile(name); err == nil {
			if len(normaliseWords(words)) > 0 {
				return words, name, true
			}
		}
	}
	if strings.TrimSpace(embedded) != "" {
		words := parseWordsTXT([]byte(embedded))
		if len(normaliseWords(words)) > 0 {
			return words, "embedded " + txtName, true
		}
	}
	return nil, "", false
}

func loadDictionary(answersPath, guessesPath string) dictionary {
	answersRaw, answersSource, okA := loadOneList(answersPath, answersJSONName, answersTXTName, embeddedAnswers)
	guessesRaw, guessesSource, _ := loadOneList(guessesPath, guessesJSONName, guessesTXTName, embeddedGuesses)

	if !okA {
		answersRaw = fallbackWords
		answersSource = "built-in fallback"
	}

	sources := make([]string, 0, 2)
	if answersSource != "" {
		sources = append(sources, answersSource)
	}
	if guessesSource != "" {
		sources = append(sources, guessesSource)
	}
	return buildDictionary(answersRaw, guessesRaw, strings.Join(sources, " + "))
}

func pickTarget(d dictionary, rng *rand.Rand, forced string) string {
	if forced != "" {
		return strings.ToUpper(forced)
	}
	pool := d.answers
	if len(pool) == 0 {
		pool = d.all
	}
	if len(pool) == 0 {
		pool = normaliseWords(fallbackWords)
	}
	if rng == nil {
		rng = rand.New(rand.NewSource(1))
	}
	return pool[rng.Intn(len(pool))]
}

func sortedKeys(m map[string]struct{}) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}
