// SPDX-License-Identifier: Apache-2.0
// Runs before action setup, with GitHub's original assignment in memory.
using System;
using System.IO;
using System.Linq;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using GitHub.DistributedTask.Pipelines;
using GitHub.DistributedTask.WebApi;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace GitHub.Runner.Worker
{
    internal static class CiproxyAssignment
    {
        internal static async Task PrepareAsync(AgentJobRequestMessage message)
        {
            var path = Environment.GetEnvironmentVariable("CIPROXY_ASSIGNMENT_SOCKET");
            if (string.IsNullOrEmpty(path)) return;
            var endpoint = message.Resources.Endpoints.Single(x =>
                string.Equals(x.Name, WellKnownServiceEndpointNames.SystemVssConnection,
                    StringComparison.OrdinalIgnoreCase));
            endpoint.Authorization.Parameters.TryGetValue("AccessToken", out var token);
            endpoint.Data.TryGetValue("CacheServerUrl", out var cacheUrl);
            endpoint.Data.TryGetValue("ResultsServiceUrl", out var resultsUrl);
            message.Variables.TryGetValue("actions_cache_mode", out VariableValue mode);
            var effectiveMode = mode?.Value;
            var payload = new {
                job_id = message.JobId.ToString(),
                runtime = new {
                    github_token = token ?? "",
                    cache_url = cacheUrl ?? "",
                    results_url = resultsUrl ?? "",
                    permissions = new {
                        read = effectiveMode == "read" || effectiveMode == "write",
                        write = effectiveMode == "write" || effectiveMode == "write-only"
                    }
                }
            };
            try
            {
                using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
                using var socket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
                await socket.ConnectAsync(new UnixDomainSocketEndPoint(path), deadline.Token);
                using var stream = new NetworkStream(socket, ownsSocket: false);
                var bytes = Encoding.UTF8.GetBytes(JsonConvert.SerializeObject(payload) + "\n");
                await stream.WriteAsync(bytes, deadline.Token);
                using var reader = new StreamReader(stream, Encoding.UTF8, false, 4096, leaveOpen: true);
                var line = await reader.ReadLineAsync(deadline.Token);
                var reply = JObject.Parse(line ?? "{}");
                if (reply.Value<bool?>("preparing") == true) {
                    await stream.WriteAsync(Encoding.UTF8.GetBytes("{\"acknowledged\":true}\n"), deadline.Token);
                    line = await reader.ReadLineAsync();
                    reply = JObject.Parse(line ?? "{}");
                }
                var proxy = reply.Value<string>("proxy_url");
                if (!string.IsNullOrEmpty(proxy)) {
                    endpoint.Data["CacheServerUrl"] = proxy;
                    endpoint.Data["ResultsServiceUrl"] = proxy;
                }
            }
            catch (Exception error) when (error is IOException || error is SocketException ||
                error is OperationCanceledException || error is JsonException)
            {
                // Cache failure must leave the original GitHub endpoints available.
            }
        }
    }
}
