package main

import (
	"fmt"
	"os"
	"strings"

	polyhook "github.com/polyhook/polyhook-go"
)

func main() {
	event, err := polyhook.Read()
	if err != nil {
		fmt.Fprintf(os.Stderr, "polyhook: %v\n", err)
		os.Exit(1)
	}

	if event.Tool != nil && *event.Tool == "bash" {
		if cmd, ok := event.Input["command"].(string); ok && strings.Contains(cmd, "rm -rf /") {
			polyhook.Respond(polyhook.Block("Refusing to delete from root"))
			return
		}
	}

	polyhook.Respond(polyhook.Approve())
}
