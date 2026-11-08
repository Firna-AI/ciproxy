"""The firewall proof preserves provider routes and checks reachable targets."""

from pathlib import Path
import copy
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from cache_egress_host import Failure
import cache_egress_network as network
from cache_egress_support import ProofHost, host_listener, peer_listener


class NetworkProofTests(unittest.TestCase):
    def run_proof(self, host):
        with patch.object(network, "command", host.command), \
                patch.object(network, "request", host.request), \
                patch.object(network, "host_listener", host_listener), \
                patch.object(network, "peer_listener", peer_listener), \
                patch.object(network, "established_probe") as established:
            result = network.probes(host)
        established.assert_called_once_with(host, 18082)
        self.assertEqual(host.routes, host.original)
        self.assertEqual(host.dns_requests, 1)
        self.assertEqual(host.forwarded, [("https://example.com", 200)])
        self.assertEqual(host.admin_paths, ["/report"])
        self.assertEqual(result["ingress_blocked"], True)
        self.assertEqual(result["established_original_blocked"], True)
        self.assertEqual(result["dns_requests"], 2)
        self.assertEqual(host.blocked_urls[:2], [
            "http://127.0.0.1:18080/", "http://[::ffff:127.0.0.1]:18080/"])
        for url in ["http://93.184.215.254:18080/", "http://10.254.253.2:18082/",
                    "http://[2001:db8:cf::2]:18083/",
                    "http://[::1]:18081/"]:
            self.assertIn(url, host.blocked_urls)
        return result

    def test_provider_metadata_stays_reachable_without_route_or_dns_changes(self):
        for destination, status in [("169.254.169.254", 200), ("169.254.0.0/16", 401),
                                    (None, 200)]:
            with self.subTest(metadata_route=destination, metadata_status=status):
                host = ProofHost(provider=True, metadata_status=status)
                if destination is None:
                    host.routes.pop()
                else:
                    host.routes[1]["dst"] = destination
                host.original = copy.deepcopy(host.routes)
                result = self.run_proof(host)
                self.assertEqual(host.provider_routes_during_proof,
                                 [host.original[1:]] * 7)
                self.assertIn("http://169.254.169.254/", host.blocked_urls)
                self.assertNotIn("http://169.254.253.2:18082/", host.blocked_urls)
                self.assertEqual(result["blocked_targets"], 7)
                self.assertEqual(result["provider_metadata"], True)
                self.assertEqual([args for args in host.commands
                                  if "169.254.169.254/32" in args], [])

    def test_host_without_provider_uses_only_disposable_link_local_listener(self):
        host = ProofHost()
        result = self.run_proof(host)
        self.assertIn("http://169.254.253.2:18082/", host.blocked_urls)
        self.assertEqual(result["blocked_targets"], 7)
        self.assertEqual(result["provider_metadata"], False)
        self.assertNotIn("http://169.254.169.254/", host.blocked_urls)
        self.assertEqual([args for args in host.commands
                          if "169.254.169.254/32" in args], [])

    def test_existing_probe_route_is_preserved_before_namespace_setup(self):
        host = ProofHost()
        host.routes.append({"dst": "169.254.253.2/32", "dev": "eth0", "metric": 100})
        with patch.object(network, "command", host.command):
            with self.assertRaisesRegex(Failure, "A disposable probe range already has a host route"):
                with network.namespace():
                    self.fail("The proof used an existing probe route")
        self.assertEqual(host.routes[-1], {
            "dst": "169.254.253.2/32", "dev": "eth0", "metric": 100})
        self.assertEqual([args for args in host.commands if "add" in args], [])

    def test_unreachable_provider_fails_without_substituting_a_listener(self):
        host = ProofHost(provider=True, metadata_status=None)
        with patch.object(network, "command", host.command), \
                patch.object(network, "request", host.request), \
                patch.object(network, "host_listener", host_listener), \
                patch.object(network, "peer_listener", peer_listener), \
                patch.object(network, "established_probe"):
            with self.assertRaisesRegex(Failure, "The control UID cannot reach the provider metadata server"):
                network.probes(host)
        self.assertEqual(host.routes, host.original)
        self.assertNotIn("http://169.254.169.254/", host.blocked_urls)
        self.assertEqual([args for args in host.commands
                          if "169.254.169.254/32" in args], [])


if __name__ == "__main__":
    unittest.main()
