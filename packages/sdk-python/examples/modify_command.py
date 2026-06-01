import os
import polyhook

event = polyhook.read()

if event.tool == "bash" and os.environ.get("DRY_RUN"):
    cmd = (event.input or {}).get("command", "")
    polyhook.respond(polyhook.modify({"command": f'echo "would run: {cmd}"'}))
else:
    polyhook.respond(polyhook.approve())
