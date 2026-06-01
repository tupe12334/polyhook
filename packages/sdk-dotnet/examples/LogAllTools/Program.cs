using Polyhook.Sdk;

var evt = await Polyhook.ReadAsync();

if (evt.Tool is not null)
{
    Console.Error.WriteLine($"[hook] caller={evt.Caller} event={evt.Event} tool={evt.Tool}");
}

await Polyhook.RespondAsync(Polyhook.Approve());
