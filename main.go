// Command tuidle is a terminal Wordle.
package main

import (
	"flag"
	"fmt"
	"math/rand"
	"os"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

func main() {
	wordsFlag := flag.String("words", "", "path to word list (.json or .txt), used as both answers and guesses (legacy; prefer -answers / -guesses); "+
		"default: ./tuidle-answers.json, then ./tuidle-answers.txt, then the embedded list")
	answersFlag := flag.String("answers", "", "path to answers/target list (.json or .txt); "+
		"default: ./tuidle-answers.json, then ./tuidle-answers.txt, then the embedded list")
	guessesFlag := flag.String("guesses", "", "path to allowed-guesses list (.json or .txt); "+
		"default: ./tuidle-allowed-guesses.json, then ./tuidle-allowed-guesses.txt, then the embedded list")
	targetFlag := flag.String("target", "", "force the target word (5 letters, for testing); "+
		"default: chosen at random from the answers list")
	versionFlag := flag.Bool("version", false, "print version and exit")
	vFlag := flag.Bool("v", false, "shorthand for -version")
	flag.Parse()

	if *versionFlag || *vFlag {
		fmt.Println("tuidle", version)
		return
	}

	if *targetFlag != "" && !isFiveLetters(strings.ToUpper(*targetFlag)) {
		fmt.Fprintln(os.Stderr, "error: -target must be exactly 5 letters A-Z")
		os.Exit(2)
	}
	for _, pf := range []struct{ name, path string }{
		{"-words", *wordsFlag},
		{"-answers", *answersFlag},
		{"-guesses", *guessesFlag},
	} {
		if pf.path != "" {
			if _, err := os.Stat(pf.path); err != nil {
				fmt.Fprintf(os.Stderr, "warning: %s %q not readable (%v); falling back to the default lists\n", pf.name, pf.path, err)
			}
		}
	}

	answersPath := *answersFlag
	guessesPath := *guessesFlag
	if *wordsFlag != "" {
		if answersPath == "" {
			answersPath = *wordsFlag
		}
		if guessesPath == "" {
			guessesPath = *wordsFlag
		}
	}

	dict := loadDictionary(answersPath, guessesPath)
	if len(dict.all) == 0 {
		fmt.Fprintln(os.Stderr, "error: no usable word list found")
		os.Exit(1)
	}

	rng := rand.New(rand.NewSource(time.Now().UnixNano()))
	m := newModel(dict, rng, *targetFlag)

	p := tea.NewProgram(m)
	if _, err := p.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "error: failed to start TUI: %v\n", err)
		fmt.Fprintln(os.Stderr, "hint: run in a real terminal (Windows Terminal on Windows, any terminal on Linux); it cannot run in a pipe or redirected shell")
		os.Exit(1)
	}
}
