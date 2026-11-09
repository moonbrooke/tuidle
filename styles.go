package main

import "github.com/charmbracelet/lipgloss"

var (
	correctBg  = lipgloss.Color("#538d4e")
	presentBg  = lipgloss.Color("#b59f3b")
	absentBg   = lipgloss.Color("#787c7e")
	brightFg   = lipgloss.Color("#f8f8f2")
	dimFg      = lipgloss.Color("#565758")
	activeFg   = lipgloss.Color("#d7dadc")
	currentBg  = lipgloss.Color("#252526")
	titleFg    = lipgloss.Color("#ffffff")
	subtitleFg = lipgloss.Color("#9a9a9a")
	warnFg     = lipgloss.Color("#f2b8b5")
	winFg      = lipgloss.Color("#a6d8a8")
	loseFg     = lipgloss.Color("#f2b8b5")
	footerFg   = lipgloss.Color("#8a8a8a")
)

var (
	titleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(titleFg).
			Align(lipgloss.Center)

	subtitleStyle = lipgloss.NewStyle().
			Foreground(subtitleFg).
			Align(lipgloss.Center)

	winStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(winFg).
			Align(lipgloss.Center)

	loseStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(loseFg).
			Align(lipgloss.Center)

	warnStyle = lipgloss.NewStyle().
			Foreground(warnFg).
			Align(lipgloss.Center)

	infoStyle = lipgloss.NewStyle().
			Foreground(subtitleFg).
			Align(lipgloss.Center)

	footerStyle = lipgloss.NewStyle().
			Foreground(footerFg).
			Align(lipgloss.Center)
)

const tileWidth = 3

func tileStyle(state LetterState) lipgloss.Style {
	base := lipgloss.NewStyle().
		Bold(true).
		Align(lipgloss.Center)

	switch state {
	case StateCorrect:
		return base.Foreground(brightFg).Background(correctBg)
	case StatePresent:
		return base.Foreground(brightFg).Background(presentBg)
	case StateAbsent:
		return base.Foreground(brightFg).Background(absentBg)
	case StateCurrent:
		return base.Foreground(brightFg).Background(currentBg)
	case StateFuture:
		return base.Foreground(dimFg)
	case StateUnused:
		return base.Foreground(activeFg)
	default:
		return base.Foreground(activeFg)
	}
}

// Decided tiles are " X " (solid bg). Pending tiles are "[X]"/"[ ]" (fg only).
func renderTile(letter byte, state LetterState) string {
	ch := " "
	if letter >= 'A' && letter <= 'Z' {
		ch = string(letter)
	}
	switch state {
	case StateCorrect, StatePresent, StateAbsent, StateCurrent:
		return tileStyle(state).Render(" " + ch + " ")
	default:
		return tileStyle(state).Render("[" + ch + "]")
	}
}
