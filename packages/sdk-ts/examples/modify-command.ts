import { read, respond, modify, approve } from "@polyhook/sdk";

const event = await read();

// Prefix all bash commands with "echo would run: " in dry-run mode
if (event.tool === "bash" && process.env.DRY_RUN) {
  const cmd = (event.input?.command as string) ?? "";
  await respond(modify({ command: `echo "would run: ${cmd}"` }));
} else {
  await respond(approve());
}
