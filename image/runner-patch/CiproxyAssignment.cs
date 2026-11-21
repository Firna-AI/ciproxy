// SPDX-License-Identifier: Apache-2.0
// Hold action setup until the host authorizes this original assignment.
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
        internal const int HandoffProtocolVersion = 2;

        internal static async Task PrepareAsync(AgentJobRequestMessage message, CancellationToken cancellationToken)
        {
            var path = Environment.GetEnvironmentVariable("CIPROXY_ASSIGNMENT_SOCKET");
            if (string.IsNullOrEmpty(path)) path = "/run/ciproxy/assignment.sock";
            var endpoint = message.Resources.Endpoints.Single(x =>
                string.Equals(x.Name, WellKnownServiceEndpointNames.SystemVssConnection,
                    StringComparison.OrdinalIgnoreCase));
            endpoint.Authorization.Parameters.TryGetValue("AccessToken", out var token);
            endpoint.Data.TryGetValue("CacheServerUrl", out var cacheUrl);
            endpoint.Data.TryGetValue("ResultsServiceUrl", out var resultsUrl);
            message.Variables.TryGetValue("actions_cache_mode", out VariableValue mode);
            var effectiveMode = mode?.Value;
            var jobId = message.JobId.ToString();
            var payload = new {
                handoff_protocol_version = HandoffProtocolVersion,
                job_id = jobId,
                runtime = new {
                    docker_data_bytes = 0,
                    proxy_url = "",
                    github_token = token ?? "",
                    cache_url = cacheUrl ?? "",
                    results_url = resultsUrl ?? "",
                    permissions = new {
                        read = effectiveMode == "read" || effectiveMode == "write",
                        write = effectiveMode == "write" || effectiveMode == "write-only"
                    }
                }
            };
            var bytes = Encoding.UTF8.GetBytes(JsonConvert.SerializeObject(payload) + "\n");
            while (true)
            {
                cancellationToken.ThrowIfCancellationRequested();
                try
                {
                    using var socket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
                    await socket.ConnectAsync(new UnixDomainSocketEndPoint(path), cancellationToken);
                    using var stream = new NetworkStream(socket, ownsSocket: false);
                    await stream.WriteAsync(bytes, cancellationToken);
                    using var reader = new StreamReader(stream, Encoding.UTF8, false, 4096, leaveOpen: true);
                    while (true)
                    {
                        var line = await reader.ReadLineAsync(cancellationToken);
                        if (line == null) throw new EndOfStreamException();
                        var reply = JObject.Parse(line);
                        if (reply["handoff_protocol_version"]?.Type != JTokenType.Integer ||
                            reply["handoff_protocol_version"]!.ToString() != HandoffProtocolVersion.ToString() ||
                            reply["job_id"]?.Type != JTokenType.String || reply.Value<string>("job_id") != jobId)
                            throw new InvalidDataException("Host decision does not match the assignment");
                        if (reply["preparing"]?.Type == JTokenType.Boolean && reply.Value<bool>("preparing"))
                        {
                            var acknowledged = new { handoff_protocol_version = HandoffProtocolVersion, job_id = jobId, acknowledged = true };
                            await stream.WriteAsync(Encoding.UTF8.GetBytes(JsonConvert.SerializeObject(acknowledged) + "\n"), cancellationToken);
                            continue;
                        }
                        var decision = reply["decision"]?.Type == JTokenType.String ? reply.Value<string>("decision") : null;
                        if (decision == "github_only") return;
                        var proxy = reply["proxy_url"]?.Type == JTokenType.String ? reply.Value<string>("proxy_url") : null;
                        if (decision != "cache_ready" || string.IsNullOrEmpty(proxy))
                            throw new InvalidDataException("Host decision is unavailable");
                        endpoint.Data["CacheServerUrl"] = proxy;
                        endpoint.Data["ResultsServiceUrl"] = proxy;
                        return;
                    }
                }
                catch (Exception error) when (error is IOException || error is SocketException || error is JsonException)
                {
                    // A transport error does not authorize action setup.
                }
                await Task.Delay(TimeSpan.FromSeconds(1), cancellationToken);
            }
        }
    }
}
