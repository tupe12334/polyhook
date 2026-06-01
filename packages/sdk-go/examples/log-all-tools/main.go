package main

import (
	"fmt"
	"os"

	polyhook "github.com/polyhook/polyhook-go"
)

func main() {
	event, err := polyhook.Read()
	if err != nil {
		fmt.Fprintf(os.Stderr, "polyhook: %v\n", err)
		os.Exit(1)
	}

	if event.Tool != nil {
		fmt.Fprintf(os.Stderr, "[hook] caller=%s event=%s tool=%s\n",
			event.Caller, event.Event, *event.Tool)
	}

	polyhook.Respond(polyhook.Approve())
}
