using System.Text.Json;
using System.Text.Json.Serialization;
using Polyhook;
using Xunit;

namespace PolyhookTests;

public class HookResponseTests
{
    private static readonly JsonSerializerOptions s_opts = new()
    {
        PropertyNamingPolicy   = JsonNamingPolicy.CamelCase,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        Converters = { new JsonStringEnumConverter(JsonNamingPolicy.KebabCaseLower) },
    };

    // -----------------------------------------------------------------------
    // Factory helpers
    // -----------------------------------------------------------------------

    [Fact]
    public void Approve_ReturnsApproveResponse()
    {
        var r = Polyhook.Polyhook.Approve();
        Assert.IsType<ApproveResponse>(r);
        Assert.Equal("approve", r.Action);
    }

    [Fact]
    public void Block_ReturnsBlockResponseWithMessage()
    {
        var r = Polyhook.Polyhook.Block("not allowed");
        var block = Assert.IsType<BlockResponse>(r);
        Assert.Equal("block",       block.Action);
        Assert.Equal("not allowed", block.Message);
    }

    [Fact]
    public void Modify_ReturnsModifyResponseWithInput()
    {
        var input = new Dictionary<string, JsonElement>
        {
            ["command"] = JsonSerializer.SerializeToElement("ls"),
        };
        var r   = Polyhook.Polyhook.Modify(input);
        var mod = Assert.IsType<ModifyResponse>(r);
        Assert.Equal("modify", mod.Action);
        Assert.Equal("ls", mod.Input["command"].GetString());
    }

    // -----------------------------------------------------------------------
    // JSON serialisation round-trips
    // -----------------------------------------------------------------------

    [Fact]
    public void ApproveResponse_Serialises_WithActionField()
    {
        var json = JsonSerializer.Serialize(Polyhook.Polyhook.Approve(), s_opts);
        using var doc = JsonDocument.Parse(json);
        Assert.Equal("approve", doc.RootElement.GetProperty("action").GetString());
    }

    [Fact]
    public void BlockResponse_Serialises_WithActionAndMessage()
    {
        var json = JsonSerializer.Serialize(Polyhook.Polyhook.Block("stop"), s_opts);
        using var doc = JsonDocument.Parse(json);
        Assert.Equal("block", doc.RootElement.GetProperty("action").GetString());
        Assert.Equal("stop",  doc.RootElement.GetProperty("message").GetString());
    }

    [Fact]
    public void ModifyResponse_Serialises_WithActionAndInput()
    {
        var input = new Dictionary<string, JsonElement>
        {
            ["file"] = JsonSerializer.SerializeToElement("foo.txt"),
        };
        var json  = JsonSerializer.Serialize(Polyhook.Polyhook.Modify(input), s_opts);
        using var doc = JsonDocument.Parse(json);
        Assert.Equal("modify",   doc.RootElement.GetProperty("action").GetString());
        Assert.Equal("foo.txt",  doc.RootElement.GetProperty("input").GetProperty("file").GetString());
    }
}
