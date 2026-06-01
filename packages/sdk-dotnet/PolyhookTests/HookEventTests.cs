using System.Text.Json;
using System.Text.Json.Serialization;
using Polyhook;
using Xunit;

namespace PolyhookTests;

public class HookEventTests
{
    private static readonly JsonSerializerOptions s_opts = new()
    {
        PropertyNamingPolicy   = JsonNamingPolicy.CamelCase,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        Converters = { new JsonStringEnumConverter(JsonNamingPolicy.KebabCaseLower) },
    };

    // Minimal JSON that a normalised parse result might look like.
    private const string MinimalJson = """
        {
          "event":     "tool:before",
          "tool":      "bash",
          "input":     { "command": "echo hi" },
          "sessionId": "sess-abc",
          "caller":    "claude-code"
        }
        """;

    [Fact]
    public void Deserialise_MinimalJson_Succeeds()
    {
        var evt = JsonSerializer.Deserialize<HookEvent>(MinimalJson, s_opts);
        Assert.NotNull(evt);
        Assert.Equal(EventKind.ToolBefore,    evt.Event);
        Assert.Equal("bash",                  evt.Tool);
        Assert.Equal("sess-abc",              evt.SessionId);
        Assert.Equal(CallerKind.ClaudeCode,   evt.Caller);
        Assert.Null(evt.AgentId);
        Assert.Null(evt.Output);
    }

    [Fact]
    public void Deserialise_Input_ContainsCommandKey()
    {
        var evt = JsonSerializer.Deserialize<HookEvent>(MinimalJson, s_opts)!;
        Assert.NotNull(evt.Input);
        Assert.True(evt.Input.ContainsKey("command"));
        Assert.Equal("echo hi", evt.Input["command"].GetString());
    }

    [Fact]
    public void CallerKind_UnknownFallback_Deserialises()
    {
        var json = """{"event":"session:start","sessionId":"s","caller":"unknown"}""";
        var evt  = JsonSerializer.Deserialize<HookEvent>(json, s_opts)!;
        Assert.Equal(CallerKind.Unknown, evt.Caller);
    }

    [Theory]
    [InlineData("claude-code", CallerKind.ClaudeCode)]
    [InlineData("cursor",      CallerKind.Cursor)]
    [InlineData("windsurf",    CallerKind.Windsurf)]
    [InlineData("cline",       CallerKind.Cline)]
    [InlineData("amp",         CallerKind.Amp)]
    [InlineData("unknown",     CallerKind.Unknown)]
    public void CallerKind_AllValues_Deserialise(string raw, CallerKind expected)
    {
        var json = $$"""{"event":"notification","sessionId":"s","caller":"{{raw}}"}""";
        var evt  = JsonSerializer.Deserialize<HookEvent>(json, s_opts)!;
        Assert.Equal(expected, evt.Caller);
    }

    [Fact]
    public void Deserialise_AllOptionalFields_Null_WhenAbsent()
    {
        var json = """{"event":"notification","sessionId":"s","caller":"cursor"}""";
        var evt  = JsonSerializer.Deserialize<HookEvent>(json, s_opts)!;
        Assert.Null(evt.Tool);
        Assert.Null(evt.Input);
        Assert.Null(evt.Output);
        Assert.Null(evt.AgentId);
    }
}
