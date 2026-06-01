using Polyhook.Sdk;
using System.Text.RegularExpressions;

var evt = await Polyhook.ReadAsync();

if (evt.Tool == "bash" &&
    evt.Input?.TryGetValue("command", out var cmdEl) == true &&
    Regex.IsMatch(cmdEl.ToString()!, @"rm\s+-rf\s+/"))
{
    await Polyhook.RespondAsync(Polyhook.Block("Refusing to delete from root"));
}
else
{
    await Polyhook.RespondAsync(Polyhook.Approve());
}
