// SPDX-License-Identifier: Apache-2.0
// Exercise the actual pinned Worker hook with local host-decision fixtures.
using System.Net.Sockets;
using System.Reflection;
using System.Runtime.ExceptionServices;
using System.Text;
using GitHub.DistributedTask.Pipelines;
using GitHub.DistributedTask.WebApi;
using GitHub.Runner.Common;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

static void Require(bool value, string label)
{
    if (!value) throw new Exception("ASSERT: " + label);
}

var scenario = args.Single();
var jobId = "7092f54e-439d-4884-94e2-84685b20d905";
var message = JsonConvert.DeserializeObject<AgentJobRequestMessage>(new JObject {
    ["JobId"] = jobId, ["Steps"] = new JArray(),
    ["Resources"] = new JObject { ["Endpoints"] = new JArray { new JObject {
        ["Name"] = "SystemVssConnection", ["Url"] = "https://runtime.example/",
        ["Authorization"] = new JObject { ["Parameters"] = new JObject { ["AccessToken"] = "fixture-original-token" } },
        ["Data"] = new JObject { ["CacheServerUrl"] = "https://cache.example/", ["ResultsServiceUrl"] = "https://results.example/" }
    } } },
    ["Variables"] = new JObject { ["actions_cache_mode"] = new JObject { ["Value"] = "read" } }
}.ToString())!;
var hook = typeof(GitHub.Runner.Worker.JobRunner).Assembly
    .GetType("GitHub.Runner.Worker.CiproxyAssignment")!
    .GetMethod("PrepareAsync", BindingFlags.Static | BindingFlags.NonPublic)!;
var socketDirectory = Environment.GetEnvironmentVariable("CIPROXY_PROOF_SOCKET_DIR")!;
var path = Path.Combine(socketDirectory, "s-" + Guid.NewGuid().ToString("N")[..8]);
Environment.SetEnvironmentVariable("CIPROXY_ASSIGNMENT_SOCKET", path);
using var listener = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
using var cancellation = new CancellationTokenSource();
var firstError = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
void FailedConnection(object? sender, FirstChanceExceptionEventArgs value)
{
    if (value.Exception is SocketException) firstError.TrySetResult();
}
if (scenario == "connect_error") AppDomain.CurrentDomain.FirstChanceException += FailedConnection;
else { listener.Bind(new UnixDomainSocketEndPoint(path)); listener.Listen(4); }
try
{
    var arguments = hook.GetParameters().Length == 1
        ? new object[] { message }
        : new object[] { message, cancellation.Token };
    var prepare = (Task)hook.Invoke(null, arguments)!;
    var stepsStarted = false;
    async Task Steps() { await prepare; stepsStarted = true; }
    var steps = Steps();
    if (scenario == "connect_error")
    {
        await firstError.Task;
        listener.Bind(new UnixDomainSocketEndPoint(path)); listener.Listen(4);
    }
    else
    {
        using var first = await listener.AcceptAsync();
        using var stream = new NetworkStream(first, ownsSocket: false);
        using var reader = new StreamReader(stream, Encoding.UTF8, false, 4096, leaveOpen: true);
        var original = JObject.Parse((await reader.ReadLineAsync())!);
        Require(original.Value<string>("job_id") == jobId, "original assignment reaches the socket");
        if (scenario == "github_cancellation")
        {
            cancellation.Cancel();
            var cancelled = false;
            try { await steps; }
            catch (OperationCanceledException) { cancelled = true; }
            Require(cancelled && !stepsStarted, "GitHub cancellation must exit without releasing steps");
            using var context = new HostContext("Worker", Path.Combine(socketDirectory, "cancellation.log"));
            var runner = new GitHub.Runner.Worker.JobRunner();
            runner.Initialize(context);
            var result = await runner.RunAsync(message, cancellation.Token);
            Require(result == TaskResult.Canceled, "the real JobRunner must report GitHub cancellation as canceled");
            Console.WriteLine("PASS: GitHub cancellation stops the actual waiting hook without starting steps");
            return;
        }
    }
    var reconnect = listener.AcceptAsync();
    var firstResult = await Task.WhenAny(steps, reconnect);
    Require(firstResult == reconnect && !stepsStarted,
        "job steps started after early socket failure without a host decision");
    using var second = await reconnect;
    using var secondStream = new NetworkStream(second, ownsSocket: false);
    using var secondReader = new StreamReader(secondStream, Encoding.UTF8, false, 4096, leaveOpen: true);
    var replay = JObject.Parse((await secondReader.ReadLineAsync())!);
    Require(replay["runtime"]!.Value<string>("github_token") == "fixture-original-token", "retry retains the original token");
    Require(replay["runtime"]!["permissions"]!.Value<bool>("write") == false, "retry retains the original rights");
    var decision = scenario == "connect_error" ? "github_only" : "cache_ready";
    var reply = new JObject {
        ["handoff_protocol_version"] = 2, ["job_id"] = jobId, ["decision"] = decision,
        ["proxy_url"] = "http://172.17.0.1:8230/"
    };
    await secondStream.WriteAsync(Encoding.UTF8.GetBytes(reply.ToString(Formatting.None) + "\n"));
    await steps;
    Require(stepsStarted, "a matching host decision must release steps");
    Require(message.Resources.Endpoints[0].Data["ResultsServiceUrl"] ==
        (decision == "github_only" ? "https://results.example/" : "http://172.17.0.1:8230/"),
        "the released assignment must use the host-selected route");
    Console.WriteLine($"PASS: {scenario} holds steps until the actual hook receives {decision}");
}
finally
{
    AppDomain.CurrentDomain.FirstChanceException -= FailedConnection;
    File.Delete(path);
}
