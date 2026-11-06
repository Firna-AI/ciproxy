#!/usr/bin/env python3
"""Publish static exports and HTML route aliases without deleting old assets."""

import argparse
import concurrent.futures
import json
import mimetypes
from pathlib import Path
import subprocess
import urllib.parse
import urllib.request
import uuid


def objects(directory):
    """Keep original objects and add both Expo and Astro route spellings."""
    original = {
        str(path.relative_to(directory)): path
        for path in sorted(directory.rglob("*"))
        if path.is_file()
    }
    published = dict(original)
    for name, path in original.items():
        if not name.endswith(".html") or name == "index.html":
            continue
        route = name[:-len("/index.html")] if name.endswith("/index.html") else name[:-5]
        published.setdefault(route, path)
        published.setdefault(route + "/", path)
    # Assets arrive before HTML, so a newly published page can load its chunks.
    return sorted(published.items(), key=lambda item: (item[1].suffix == ".html", item[0]))


def metadata(name, path):
    html = path.suffix == ".html"
    return {
        "name": name,
        "contentType": "text/html; charset=utf-8" if html else (
            mimetypes.guess_type(path.name)[0] or "application/octet-stream"
        ),
        "cacheControl": "no-store" if html else "public,max-age=86400,must-revalidate",
    }


def upload(bucket, name, path, token):
    boundary = "ciproxy-" + uuid.uuid4().hex
    fields = metadata(name, path)
    # Cloud Storage refuses a media part whose type differs from the metadata.
    data = (
        f"--{boundary}\r\nContent-Type: application/json; charset=utf-8\r\n\r\n".encode()
        + json.dumps(fields).encode()
        + f"\r\n--{boundary}\r\nContent-Type: {fields['contentType']}\r\n\r\n".encode()
        + path.read_bytes()
        + f"\r\n--{boundary}--\r\n".encode()
    )
    query = urllib.parse.urlencode({"uploadType": "multipart"})
    request = urllib.request.Request(
        f"https://storage.googleapis.com/upload/storage/v1/b/{bucket}/o?{query}",
        data=data,
        headers={
            "Authorization": "Bearer " + token,
            "Content-Type": "multipart/related; boundary=" + boundary,
        },
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=120) as response:
        response.read()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--bucket", required=True)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if not (args.directory / "index.html").is_file():
        parser.error("the export must contain index.html")
    published = objects(args.directory)
    if args.dry_run:
        print(json.dumps([metadata(name, path) for name, path in published], indent=2))
        return
    token = subprocess.check_output(
        ["gcloud", "auth", "print-access-token"], text=True
    ).strip()
    # Wait for every asset upload before any HTML upload. Existing files stay
    # available for old browser tabs. CDN invalidation follows edge deployment.
    for html in [False, True]:
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
            tasks = [executor.submit(upload, args.bucket, name, path, token)
                     for name, path in published if (path.suffix == ".html") == html]
            for task in tasks:
                task.result()
    print(f"Published {len(published)} objects to gs://{args.bucket}.")


if __name__ == "__main__":
    main()
