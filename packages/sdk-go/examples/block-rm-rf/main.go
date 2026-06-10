package main

import (
	"fmt"
	"os"
	"strings"

	polyhook "github.com/tupe12334/polyhook/packages/sdk-go"
)

func main() {
	event, err := polyhook.Read()
	if err != nil {
		fmt.Fprintf(os.Stderr, "polyhook: %v\n", err)
		os.Exit(1)
	}

	if event.Tool != nil && *event.Tool == "bash" {
		if input, ok := event.Input.(map[string]interface{}); ok {
			if cmd, ok := input["command"].(string); ok && strings.Contains(cmd, "rm -rf /") {
				polyhook.Respond(polyhook.Block("Refusing to delete from root"))
				return
			}
		}
	}

	polyhook.Respond(polyhook.Approve())
}
