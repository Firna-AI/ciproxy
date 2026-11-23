// SPDX-License-Identifier: Apache-2.0
// A .NET startup hook for overlay-load.py. The runner's own Worker host loads
// it before Main. It loads each raised package from the runner folder, runs
// code in each, and exits, so Main never runs.
using System;
using System.Diagnostics;
using System.IO;
using System.Reflection;

internal class StartupHook
{
    public static void Initialize()
    {
        var expected = Environment.GetEnvironmentVariable("CIPROXY_OVERLAY_PACKAGES");
        if (expected == null)
        {
            return;
        }
        var bin = Path.TrimEndingDirectorySeparator(AppContext.BaseDirectory);
        var failed = false;
        foreach (var line in expected.Split(';', StringSplitOptions.RemoveEmptyEntries))
        {
            var parts = line.Split(' ');
            var assembly = Assembly.Load(new AssemblyName(parts[0]));
            var product = FileVersionInfo.GetVersionInfo(assembly.Location).ProductVersion ?? "";
            var ok = Path.GetDirectoryName(assembly.Location) == bin && product.Split('+')[0] == parts[1];
            Console.WriteLine($"{(ok ? "PASS" : "FAIL")}: {parts[0]} {product} from {assembly.Location}");
            failed |= !ok;
        }
        var query = Type.GetType("Microsoft.AspNetCore.WebUtilities.QueryHelpers, Microsoft.AspNetCore.WebUtilities", true)!
            .GetMethod("ParseQuery", new[] { typeof(string) })!
            .Invoke(null, new object?[] { "?a=1&b=2" })!;
        var count = (int)query.GetType().GetProperty("Count")!.GetValue(query)!;
        var date = (string)Type.GetType("Microsoft.Net.Http.Headers.HeaderUtilities, Microsoft.Net.Http.Headers", true)!
            .GetMethod("FormatDate", new[] { typeof(DateTimeOffset) })!
            .Invoke(null, new object[] { DateTimeOffset.UnixEpoch })!;
        var tunnels = Assembly.Load(new AssemblyName("Microsoft.DevTunnels.Connections"));
        var ran = count == 2 && date == "Thu, 01 Jan 1970 00:00:00 GMT";
        Console.WriteLine($"{(ran ? "PASS" : "FAIL")}: package code ran; {tunnels.GetName().Name} loaded");
        Environment.Exit(failed || !ran ? 1 : 0);
    }
}
