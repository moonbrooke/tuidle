package main

const (
	wordLen    = 5
	maxGuesses = 6
)

type LetterState int

const (
	StateUnused  LetterState = iota // not guessed yet
	StateAbsent                     // not in target
	StatePresent                    // in target, wrong position
	StateCorrect                    // in target, right position
)

// Rendering-only states for the current / future rows.
const (
	StateCurrent LetterState = iota + 100
	StateActiveEmpty
	StateFuture
)

// evaluate scores guess against target (both 5 uppercase letters).
// Exact matches consume counts first, so repeated letters score correctly.
func evaluate(target, guess string) [wordLen]LetterState {
	var result [wordLen]LetterState
	if len(target) != wordLen || len(guess) != wordLen {
		for i := range result {
			result[i] = StateAbsent
		}
		return result
	}

	var counts [26]int
	for i := 0; i < wordLen; i++ {
		if c := target[i]; c >= 'A' && c <= 'Z' {
			counts[c-'A']++
		}
	}

	for i := 0; i < wordLen; i++ {
		if guess[i] == target[i] {
			result[i] = StateCorrect
			if c := target[i]; c >= 'A' && c <= 'Z' {
				counts[c-'A']--
			}
		}
	}

	for i := 0; i < wordLen; i++ {
		if result[i] == StateCorrect {
			continue
		}
		c := guess[i]
		if c < 'A' || c > 'Z' {
			result[i] = StateAbsent
			continue
		}
		if counts[c-'A'] > 0 {
			result[i] = StatePresent
			counts[c-'A']--
		} else {
			result[i] = StateAbsent
		}
	}
	return result
}

// letterRank keeps better keyboard states from being overwritten.
func letterRank(s LetterState) int {
	switch s {
	case StateCorrect:
		return 3
	case StatePresent:
		return 2
	case StateAbsent:
		return 1
	default:
		return 0
	}
}

func updateKeyboard(kb *[26]LetterState, guess string, feedback [wordLen]LetterState) {
	for i := 0; i < wordLen && i < len(guess); i++ {
		c := guess[i]
		if c < 'A' || c > 'Z' {
			continue
		}
		idx := c - 'A'
		if letterRank(feedback[i]) > letterRank(kb[idx]) {
			kb[idx] = feedback[i]
		}
	}
}

func isFiveLetters(s string) bool {
	if len(s) != wordLen {
		return false
	}
	for i := 0; i < len(s); i++ {
		if s[i] < 'A' || s[i] > 'Z' {
			return false
		}
	}
	return true
}

func winMessage(n int) string {
	switch n {
	case 1:
		return "Genius!"
	case 2:
		return "Magnificent!"
	case 3:
		return "Impressive!"
	case 4:
		return "Splendid!"
	case 5:
		return "Great!"
	default:
		return "Phew!"
	}
}
