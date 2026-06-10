import sys
import re
import polyhook

event = polyhook.read()

if event.tool == "bash" and re.search(
    r"rm\s+-rf\s+/", event.input.get("command", "") if event.input else ""
):
    polyhook.respond(polyhook.block("Refusing to delete from root"))
else:
    polyhook.respond(polyhook.approve())
