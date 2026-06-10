using System.Reflection;
using System.Runtime.CompilerServices;
using System.Text.Json;
using System.Text.Json.Serialization;
using Wasmtime;

[assembly: InternalsVisibleTo("PolyhookTests")]

namespace Polyhook;

// ---------------------------------------------------------------------------
// IWasmInvoker — seam for testing without a real WASM binary
// ---------------------------------------------------------------------------

/// <summary>
/// Abstraction over the WASM call used by <see cref="Polyhook"/> so that
/// unit tests can inject a mock without needing a real <c>polyhook.wasm</c>.
/// </summary>
internal interface IWasmInvoker
{
    /// <summary>
    /// Executes a WASM guest function.
    /// </summary>
    /// <param name="wasmCall">
    /// Delegate that receives the instantiated <see cref="Instance"/>, the
    /// pointer to the input buffer, and the input length; returns the result
    /// pointer (length-prefixed).
    /// </param>
    /// <param name="inputBytes">Raw bytes to pass into WASM linear memory.</param>
    /// <returns>The raw payload bytes (after stripping the 4-byte length prefix).</returns>
    byte[] Invoke(Func<Instance, int, int, int> wasmCall, byte[] inputBytes);
}

/// <summary>
/// Production implementation of <see cref="IWasmInvoker"/> that delegates to
/// the private <c>InvokeWasm</c> helper on <see cref="Polyhook"/>.
/// </summary>
internal sealed class DefaultWasmInvoker : IWasmInvoker
{
    /// <inheritdoc/>
    public byte[] Invoke(Func<Instance, int, int, int> wasmCall, byte[] inputBytes) =>
        Polyhook.InvokeWasm(wasmCall, inputBytes);
}

/// <summary>
/// Entry-point for polyhook. Wraps the embedded <c>polyhook.wasm</c> via
/// Wasmtime and exposes a minimal async API for reading hook events and
/// writing hook responses.
/// </summary>
public static class Polyhook
{
    // ---------------------------------------------------------------------------
    // JSON options — shared across all calls
    // ---------------------------------------------------------------------------

    private static readonly JsonSerializerOptions s_jsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        Converters = { new JsonStringEnumConverter(JsonNamingPolicy.KebabCaseLower) },
    };

    // ---------------------------------------------------------------------------
    // Lazy WASM engine — created once per process
    // ---------------------------------------------------------------------------

    private static readonly Lazy<(Engine engine, byte[] wasmBytes)> s_engineAndBytes =
        new(LoadEngine, LazyThreadSafetyMode.ExecutionAndPublication);

    // Caller detected during the most recent ReadAsync call; used by RespondAsync.
    private static volatile string? s_lastCaller;

    // ---------------------------------------------------------------------------
    // Testability seam — replace with a mock in unit tests
    // ---------------------------------------------------------------------------

    /// <summary>
    /// The <see cref="IWasmInvoker"/> used by <see cref="ReadAsync"/> and
    /// <see cref="RespondAsync"/>. Tests may substitute a mock here; always
    /// restore the default after each test.
    /// </summary>
    internal static IWasmInvoker WasmInvoker { get; set; } = new DefaultWasmInvoker();

    // ---------------------------------------------------------------------------
    // Public API
    // ---------------------------------------------------------------------------

    /// <summary>
    /// Reads the raw stdin payload, passes it through <c>polyhook.wasm</c> for
    /// normalisation, and returns a <see cref="HookEvent"/>.
    /// </summary>
    /// <param name="inputStream">
    /// Stream to read the raw hook payload from. Defaults to
    /// <see cref="Console.OpenStandardInput()"/> when <see langword="null"/>.
    /// </param>
    /// <param name="cancellationToken">Propagates notification that operations should be cancelled.</param>
    public static async Task<HookEvent> ReadAsync(
        Stream? inputStream = null,
        CancellationToken cancellationToken = default)
    {
        // 1. Read all stdin bytes.
        var source = inputStream ?? Console.OpenStandardInput();
        using var ms = new MemoryStream();
        await source.CopyToAsync(ms, cancellationToken);
        var stdinBytes = ms.ToArray();

        // 2. Invoke WASM parse().
        var resultJson = WasmInvoker.Invoke(
            static (instance, inputPtr, inputLen) =>
            {
                var parse = instance.GetFunction<int, int, int>("parse")
                    ?? throw new InvalidOperationException("WASM export 'parse' not found.");
                return parse(inputPtr, inputLen);
            },
            stdinBytes);

        // 3. Deserialize the normalised HookEvent.
        var hookEvent = JsonSerializer.Deserialize<HookEvent>(resultJson, s_jsonOptions)
            ?? throw new InvalidOperationException("polyhook.wasm returned null for parse result.");

        // 4. Stash the caller so RespondAsync can produce the right format.
        s_lastCaller = hookEvent.Caller.ToString();

        return hookEvent;
    }

    /// <summary>
    /// Serialises <paramref name="response"/> via <c>polyhook.wasm</c> and
    /// writes the caller-specific bytes to stdout.
    /// </summary>
    /// <remarks>
    /// Must be called after <see cref="ReadAsync"/> — detection state from the
    /// previous parse call is used internally by the WASM module.
    /// </remarks>
    /// <param name="response">The hook response to serialise and emit.</param>
    /// <param name="outputStream">
    /// Stream to write the serialised response to. Defaults to
    /// <see cref="Console.OpenStandardOutput()"/> when <see langword="null"/>.
    /// </param>
    /// <param name="cancellationToken">Propagates notification that operations should be cancelled.</param>
    public static async Task RespondAsync(
        HookResponse response,
        Stream? outputStream = null,
        CancellationToken cancellationToken = default)
    {
        // 1. Serialise the HookResponse to JSON.
        var responseJson = JsonSerializer.SerializeToUtf8Bytes(response, s_jsonOptions);

        // 2. Invoke WASM serialize().
        var outBytes = WasmInvoker.Invoke(
            static (instance, inputPtr, inputLen) =>
            {
                var serialize = instance.GetFunction<int, int, int>("serialize")
                    ?? throw new InvalidOperationException("WASM export 'serialize' not found.");
                return serialize(inputPtr, inputLen);
            },
            responseJson);

        // 3. Write caller-formatted bytes to the output stream.
        var stdout = outputStream ?? Console.OpenStandardOutput();
        await stdout.WriteAsync(outBytes, cancellationToken);
        await stdout.FlushAsync(cancellationToken);
    }

    // ---------------------------------------------------------------------------
    // Factory helpers
    // ---------------------------------------------------------------------------

    /// <summary>Returns an <see cref="ApproveResponse"/> that tells the caller to proceed.</summary>
    public static HookResponse Approve() => new ApproveResponse();

    /// <summary>Returns a <see cref="BlockResponse"/> that aborts the operation.</summary>
    /// <param name="message">Human-readable reason surfaced to the user.</param>
    public static HookResponse Block(string message) =>
        new BlockResponse { Message = message };

    /// <summary>
    /// Returns a <see cref="ModifyResponse"/> that replaces the tool's input
    /// arguments with <paramref name="input"/>.
    /// </summary>
    public static HookResponse Modify(Dictionary<string, JsonElement> input) =>
        new ModifyResponse { Input = input };

    // ---------------------------------------------------------------------------
    // Internal WASM helpers
    // ---------------------------------------------------------------------------

    /// <summary>
    /// Generic helper that:
    /// <list type="number">
    ///   <item>Creates a fresh Wasmtime Store + Instance per call (stateless host).</item>
    ///   <item>alloc's a region for <paramref name="inputBytes"/>.</item>
    ///   <item>Invokes <paramref name="wasmCall"/> with the allocated pointer.</item>
    ///   <item>Reads the length-prefixed result.</item>
    ///   <item>dealloc's both regions.</item>
    ///   <item>Returns the raw payload bytes.</item>
    /// </list>
    /// </summary>
    internal static byte[] InvokeWasm(
        Func<Instance, int, int, int> wasmCall,
        byte[] inputBytes)
    {
        var (engine, wasmBytes) = s_engineAndBytes.Value;

        using var store = new Store(engine);
        using var module = Wasmtime.Module.FromBytes(engine, "polyhook", wasmBytes);
        var linker = new Linker(engine);
        var instance = linker.Instantiate(store, module);

        var memory = instance.GetMemory("memory")
            ?? throw new InvalidOperationException("WASM export 'memory' not found.");

        var alloc = instance.GetFunction<int, int>("alloc")
            ?? throw new InvalidOperationException("WASM export 'alloc' not found.");
        var dealloc = instance.GetAction<int, int>("dealloc")
            ?? throw new InvalidOperationException("WASM export 'dealloc' not found.");

        // --- write input into WASM memory ---
        int inputLen = inputBytes.Length;
        int inputPtr = alloc(inputLen);
        inputBytes.CopyTo(memory.GetSpan(inputPtr, inputLen));

        // --- call parse or serialize ---
        int resultPtr = wasmCall(instance, inputPtr, inputLen);

        // --- free input allocation ---
        dealloc(inputPtr, inputLen);

        // --- read length prefix (4 bytes LE i32) ---
        var lenBytes = memory.GetSpan(resultPtr, 4).ToArray();
        int payloadLen = BitConverter.ToInt32(lenBytes, 0);

        // --- read payload ---
        var payload = memory.GetSpan(resultPtr + 4, payloadLen).ToArray();

        // --- free result allocation ---
        dealloc(resultPtr, 4 + payloadLen);

        return payload;
    }

    private static (Engine engine, byte[] wasmBytes) LoadEngine()
    {
        var assembly = Assembly.GetExecutingAssembly();
        // Embedded resource name: <AssemblyName>.<filename>
        var resourceName = assembly.GetManifestResourceNames()
            .FirstOrDefault(n => n.EndsWith("polyhook.wasm", StringComparison.OrdinalIgnoreCase))
            ?? throw new InvalidOperationException(
                "Embedded resource 'polyhook.wasm' not found. " +
                "Ensure the file is included via <EmbeddedResource Include=\"polyhook.wasm\" /> " +
                "in the .csproj and that the build was not performed without it.");

        using var stream = assembly.GetManifestResourceStream(resourceName)!;
        using var ms = new MemoryStream((int)stream.Length);
        stream.CopyTo(ms);
        var wasmBytes = ms.ToArray();

        var config = new Config();
        var engine = new Engine(config);

        return (engine, wasmBytes);
    }
}
