"""Disposable listeners prove blocked cache requests against reachable targets."""

from contextlib import contextmanager
import http.client
import http.server
import ipaddress
import json
from pathlib import Path
import selectors
import socket
import subprocess
import sys
import threading
import urllib.error
import uuid

sys.dont_write_bytecode = True

from cache_egress_host import command, request, require


def routes(family):
    return json.loads(command("ip", "-j", "-" + family, "route", "show", "table", "all").stdout)


def unused_probe_ranges(link_local):
    ipv4 = ["10.254.253.0/30", "93.184.215.254/32"]
    if link_local:
        ipv4.append("169.254.253.2/32")
    for family, ranges in [("4", ipv4),
                           ("6", ["2001:db8:cf::/64"])]:
        for route in routes(family):
            destination = route.get("dst", "default")
            if destination == "default":
                continue
            existing = ipaddress.ip_network(destination, strict=False)
            require(not any(existing.overlaps(ipaddress.ip_network(probe)) for probe in ranges),
                    "A disposable probe range already has a host route")


def provider_metadata(cache):
    url = "http://169.254.169.254/"
    try:
        status, _ = request(url, method="HEAD", timeout=5)
    except (OSError, urllib.error.URLError, http.client.HTTPException):
        address = ipaddress.ip_address("169.254.169.254")
        for route in routes("4"):
            destination = route.get("dst", "default")
            if destination != "default":
                require(address not in ipaddress.ip_network(destination, strict=False),
                        "The control UID cannot reach the provider metadata server")
        return False
    require(100 <= status < 600, "The control UID received no metadata HTTP response")
    cache.blocked(url)
    return True


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/count":
            body = str(self.server.hits).encode()
        else:
            self.server.hits += 1
            body = b"ciproxy-public-firewall-probe"
        self.send_response(200)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


class Server(http.server.ThreadingHTTPServer):
    hits = 0


class IPv6Server(Server):
    address_family = socket.AF_INET6


def serve(family):
    server = (IPv6Server if family == "6" else Server)(("::" if family == "6" else "0.0.0.0", 0), Handler)
    print(server.server_port, flush=True)
    server.serve_forever()


@contextmanager
def host_listener(family):
    server = (IPv6Server if family == "6" else Server)(("::1" if family == "6" else "0.0.0.0", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield server.server_port
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


@contextmanager
def namespace(link_local=True):
    unused_probe_ranges(link_local)
    tag = uuid.uuid4().hex[:12]
    name, host, peer = "cache-egress-" + tag, "cpe" + tag, "cep" + tag
    created = False
    routed = False
    try:
        created = True
        command("ip", "netns", "add", name)
        command("ip", "link", "add", host, "type", "veth", "peer", "name", peer)
        command("ip", "link", "set", peer, "netns", name)
        for address in ["10.254.253.1/30", "93.184.215.254/32", "2001:db8:cf::1/64"]:
            command("ip", "addr", "add", address, "dev", host, *(["nodad"] if ":" in address else []))
        addresses = ["10.254.253.2/30", "2001:db8:cf::2/64"]
        if link_local:
            addresses.append("169.254.253.2/32")
        for address in addresses:
            command("ip", "-n", name, "addr", "add", address, "dev", peer, *(["nodad"] if ":" in address else []))
        command("ip", "link", "set", host, "up")
        command("ip", "-n", name, "link", "set", peer, "up")
        command("ip", "-n", name, "link", "set", "lo", "up")
        if link_local:
            command("ip", "route", "add", "169.254.253.2/32", "via", "10.254.253.2", "dev", host)
            routed = True
        yield name
    finally:
        if routed:
            command("ip", "route", "delete", "169.254.253.2/32", "via", "10.254.253.2", "dev", host, check=False)
        if created:
            command("ip", "link", "delete", host, check=False)
            command("ip", "netns", "delete", name, check=False)
            require(command("ip", "link", "show", "dev", host, check=False).returncode != 0,
                    "The disposable host link was not removed")
            require(not (Path("/run/netns") / name).exists(),
                    "The disposable network namespace was not removed")


@contextmanager
def peer_listener(name, family):
    child = subprocess.Popen(["ip", "netns", "exec", name, sys.executable, "-c",
                              Path(__file__).read_text(), family],
                             stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                             text=True, cwd=Path(__file__).parent)
    try:
        with selectors.DefaultSelector() as ready:
            ready.register(child.stdout, selectors.EVENT_READ)
            require(ready.select(10), "The disposable peer listener did not start")
        line = child.stdout.readline().strip()
        require(line.isdigit(), "The disposable peer listener failed")
        yield int(line)
    finally:
        child.terminate()
        try:
            child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()


def blocked_target(cache, url):
    status, _ = request(url)
    require(status == 200, "The control UID cannot reach the disposable upstream")
    _, before = request(url + "count")
    cache.blocked(url)
    _, after = request(url + "count")
    require(after == before, "A blocked cache request reached the disposable upstream")


def established_probe(cache, port):
    code = """import socket, sys
connection = socket.create_connection(('10.254.253.2', int(sys.argv[1])), 10)
connection.settimeout(5)
print('READY', flush=True)
sys.stdin.readline()
connection.sendall(b'GET / HTTP/1.0\\r\\nHost: public-probe\\r\\n\\r\\n')
try:
    connection.recv(1024)
    sys.exit(1)
except socket.timeout:
    sys.exit(0)
"""
    child = None
    try:
        command("nft", "insert", "rule", "inet", "ciproxy_cache", "service",
                "ip", "daddr", "10.254.253.2", "tcp", "dport", str(port), "return")
        child = subprocess.Popen(["setpriv", "--reuid=" + str(cache.account.pw_uid),
                                  "--regid=" + str(cache.account.pw_gid), "--clear-groups",
                                  sys.executable, "-c", code, str(port)],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                 stderr=subprocess.DEVNULL, text=True)
        with selectors.DefaultSelector() as ready:
            ready.register(child.stdout, selectors.EVENT_READ)
            require(ready.select(15), "The established private control connection did not start")
        require(child.stdout.readline().strip() == "READY", "The private control connection failed")
        command("/bin/sh", "/etc/ciproxy-cache-firewall")
        before = cache.drops()
        child.communicate("go\n", timeout=15)
        require(child.returncode == 0 and cache.drops() > before,
                "An established original-direction connection bypassed the output policy")
    finally:
        if child and child.poll() is None:
            child.kill()
            child.wait()
        command("/bin/sh", "/etc/ciproxy-cache-firewall")


def probes(cache):
    with host_listener("4") as port:
        for host in ["127.0.0.1", "[::ffff:127.0.0.1]"]:
            blocked_target(cache, f"http://{host}:{port}/")
        metadata = provider_metadata(cache)
        with namespace(link_local=not metadata) as name:
            blocked_target(cache, f"http://93.184.215.254:{port}/")
            with peer_listener(name, "4") as peer_port:
                for host in ["10.254.253.2"] + ([] if metadata else ["169.254.253.2"]):
                    blocked_target(cache, f"http://{host}:{peer_port}/")
                established_probe(cache, peer_port)
            with peer_listener(name, "6") as peer_port:
                blocked_target(cache, f"http://[2001:db8:cf::2]:{peer_port}/")
            before = cache.drops("input")
            result = command("ip", "netns", "exec", name, sys.executable, "-c",
                             "import socket; socket.create_connection(('10.254.253.1',7887),5)",
                             check=False)
            require(result.returncode != 0, "An external interface reached the cache listener")
            require(cache.drops("input") > before, "The installed ingress rule did not drop the probe")
    with host_listener("6") as port:
        blocked_target(cache, f"http://[::1]:{port}/")
    cache.forward("https://example.com", 200)
    dns = cache.dns()
    cache.admin("/report")
    return {"blocked_targets": 7, "provider_metadata": metadata,
            "established_original_blocked": True,
            "ingress_blocked": True, "public_https": True,
            "dns_requests": dns, "admin_reply": True}


if __name__ == "__main__":
    serve(sys.argv[1])
