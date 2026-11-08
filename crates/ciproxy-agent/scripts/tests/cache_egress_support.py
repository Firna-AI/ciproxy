"""Host boundaries for the cache firewall proof's network regression tests."""

from contextlib import contextmanager
import copy
import ipaddress
import json
from types import SimpleNamespace
from urllib.error import URLError
from urllib.parse import urlsplit

from cache_egress_host import Failure


@contextmanager
def host_listener(family):
    yield 18080 if family == "4" else 18081


@contextmanager
def peer_listener(name, family):
    yield 18082 if family == "4" else 18083


class ProofHost:
    def __init__(self, provider=False, metadata_status=200):
        self.routes = [{"dst": "default", "gateway": "192.0.2.1", "dev": "eth0"}]
        if provider:
            self.routes.append({"dst": "169.254.169.254", "dev": "eth0",
                                "gateway": "192.0.2.1", "metric": 100})
        self.original = copy.deepcopy(self.routes)
        self.provider = provider
        self.metadata_status = metadata_status
        self.commands = []
        self.controls = []
        self.blocked_urls = []
        self.provider_routes_during_proof = []
        self.hits = {}
        self.admin_paths = []
        self.forwarded = []
        self.dns_requests = 0
        self.ingress_drops = 0

    def command(self, *args, **kwargs):
        self.commands.append(args)
        status, output = 0, ""
        if args[:5] == ("ip", "-j", "-4", "route", "show"):
            output = json.dumps(self.routes)
        elif args[:5] == ("ip", "-j", "-6", "route", "show"):
            output = "[]"
        elif args[:3] == ("ip", "route", "add"):
            self.routes.append({"dst": args[3], "gateway": args[5],
                                "dev": args[7], "metric": 0})
        elif args[:3] == ("ip", "route", "delete"):
            self.routes = [route for route in self.routes
                           if not (route["dst"] == args[3] and route["dev"] == args[7])]
        elif args[:3] == ("ip", "link", "show"):
            status = 1
        elif args[:3] == ("ip", "netns", "exec"):
            self.ingress_drops += 1
            status = 1
        return SimpleNamespace(returncode=status, stdout=output)

    def request(self, url, method="GET", **kwargs):
        target = urlsplit(url)
        self.controls.append((url, method))
        if target.hostname == "169.254.169.254" and target.port is None:
            if not self.provider or self.metadata_status is None:
                raise URLError("No metadata HTTP response")
            return self.metadata_status, b""
        if target.path == "/count":
            return 200, str(self.hits.get(target.netloc, 0)).encode()
        self.hits[target.netloc] = self.hits.get(target.netloc, 0) + 1
        return 200, b"public probe"

    def blocked(self, url):
        self.blocked_urls.append(url)
        self.provider_routes_during_proof.append([
            copy.deepcopy(route) for route in self.routes
            if route["dst"] != "default" and ipaddress.ip_address("169.254.169.254")
            in ipaddress.ip_network(route["dst"], strict=False)])
        if not any(control == url for control, _ in self.controls):
            raise Failure("The cache probe has no reachable control")

    def drops(self, chain="service"):
        return self.ingress_drops

    def forward(self, url, expected):
        self.forwarded.append((url, expected))

    def dns(self):
        self.dns_requests += 1
        return 2

    def admin(self, path):
        self.admin_paths.append(path)
