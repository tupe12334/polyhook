import sys
import polyhook

event = polyhook.read()

if event.tool:
    print(
        f"[hook] caller={event.caller} event={event.event} tool={event.tool}",
        file=sys.stderr,
    )

polyhook.respond(polyhook.approve())
