import { read, respond, block, approve } from "@polyhook/sdk";

const event = await read();

if (
  event.tool === "bash" &&
  /rm\s+-rf\s+\//.test((event.input?.command as string) ?? "")
) {
  await respond(block("Refusing to delete from root"));
} else {
  await respond(approve());
}
