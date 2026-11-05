#!/usr/bin/env python3
"""Check a job's original runtime token and print only safe claim fields."""

import argparse
import base64
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
from urllib.request import urlopen

sys.dont_write_bytecode = True

ISSUER = "https://token.actions.githubusercontent.com"
JWKS = ISSUER + "/.well-known/jwks"
IDENTITIES = ("repository_id", "run_id", "job_id", "runner_id")


class SafeParser(argparse.ArgumentParser):
    def error(self, _message):
        self.exit(2, "Invalid arguments. Supply no token on the command line.\n")


def decode(value):
    return base64.urlsafe_b64decode(value + "=" * (-len(value) % 4))


def fetch_keys():
    with urlopen(ISSUER + "/.well-known/openid-configuration", timeout=10) as response:
        discovery = json.load(response)
    if discovery.get("issuer") != ISSUER or discovery.get("jwks_uri") != JWKS:
        raise ValueError("unrecognized discovery")
    with urlopen(JWKS, timeout=10) as response:
        return json.load(response)


def openssl(*arguments, data=None):
    result = subprocess.run(["openssl", *arguments], input=data,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if result.returncode:
        raise ValueError("signature operation failed")
    return result.stdout


def check(token, keys, now):
    """Verify with OpenSSL before returning the permitted diagnostic fields."""
    header_part, payload_part, signature_part = token.split(".")
    header = json.loads(decode(header_part))
    if header.get("alg") != "RS256" or not header.get("kid") or not signature_part:
        raise ValueError("unsupported token")
    key = next(key for key in keys["keys"] if key.get("kid") == header["kid"])
    if key.get("kty") != "RSA":
        raise ValueError("unsupported key")
    context = Path.cwd() / ".context/ciproxy-mvp/signed-job-ref/probe"
    context.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=context) as directory:
        directory = Path(directory)
        config = directory / "public-key.cnf"
        config.write_text("asn1=SEQUENCE:rsa\n[rsa]\nn=INTEGER:0x" + decode(key["n"]).hex()
                          + "\ne=INTEGER:0x" + decode(key["e"]).hex() + "\n")
        der = directory / "public.der"
        pem = directory / "public.pem"
        signature = directory / "signature.bin"
        signature.write_bytes(decode(signature_part))
        openssl("asn1parse", "-genconf", str(config), "-out", str(der), "-noout")
        openssl("rsa", "-RSAPublicKey_in", "-inform", "DER", "-in", str(der),
                "-pubout", "-out", str(pem))
        openssl("dgst", "-sha256", "-verify", str(pem), "-signature", str(signature),
                data=(header_part + "." + payload_part).encode("ascii"))
    claims = json.loads(decode(payload_part))
    nbf, exp = claims["nbf"], claims["exp"]
    if (claims.get("iss") != ISSUER or type(nbf) is not int or type(exp) is not int
            or nbf > exp or nbf > now + 30 or exp + 30 < now):
        raise ValueError("invalid validity or issuer")
    access = json.loads(claims["ac"])
    if not isinstance(access, list):
        raise ValueError("invalid access control")
    safe_access = []
    for entry in access:
        if not isinstance(entry["Scope"], str) or type(entry["Permission"]) is not int:
            raise ValueError("invalid access control")
        safe_access.append({"Scope": entry["Scope"], "Permission": entry["Permission"]})
    result = {"claim_names": sorted(claims), "ac": safe_access}
    for name in IDENTITIES:
        value = claims.get(name)
        if not isinstance(value, (str, int)) or isinstance(value, bool):
            raise ValueError("missing identity")
        result[name] = value
    return result


def main():
    parser = SafeParser(description=__doc__)
    parser.parse_args()
    try:
        result = check(os.environ["ACTIONS_RUNTIME_TOKEN"], fetch_keys(), int(time.time()))
    except Exception:
        print("Runtime token check failed. No token details were printed.", file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
