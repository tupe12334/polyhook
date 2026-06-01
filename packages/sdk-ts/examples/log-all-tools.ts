import { read, respond, approve } from "@polyhook/sdk";

const event = await read();

if (event.tool) {
  process.stderr.write(
    `[hook] caller=${event.caller} event=${event.event} tool=${event.tool}\n`
  );
}

await respond(approve());
