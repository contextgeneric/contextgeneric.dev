#!/usr/bin/env python3
"""Tell search engines which of the site's pages changed, through IndexNow.

IndexNow shares one submission with Bing and the other engines that use it; Google does not. The
site proves it owns the key through the file `static/<key>.txt`, which must stay deployed.

Run it from the repository's root after a deploy, naming the changed pages:

    python3 scripts/indexnow/submit.py https://contextgeneric.dev/docs/quickstart/
    python3 scripts/indexnow/submit.py --all --dry-run

`--all` submits every URL in the live sitemap and every redirect stub under `static/`, which suits
only a relaunch or migration; after one, submit only the pages that changed. `--dry-run` prints the
list and whether the live key file matches, without sending anything. `--from` reads the sitemap
from another origin, such as `npm run serve`, and `--endpoint` sends to another IndexNow endpoint.

It refuses a URL on another host, and refuses to send while the key file on the live site is
missing or differs from the one in `static/`, since the engines check the live site. It prints
the endpoint's answer, with any explanation in its body, and exits 0 when the endpoint answers 200
or 202, and 1 otherwise.
"""
import argparse
import json
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path
from urllib.parse import urlsplit

SITE = "https://contextgeneric.dev"
HOST = "contextgeneric.dev"
ENDPOINT = "https://api.indexnow.org/indexnow"
MAX_URLS = 10_000
ROOT = Path(__file__).resolve().parents[2]
STATIC = ROOT / "static"

MEANINGS = {
    200: "received",
    202: "received; the engines have not verified the key yet",
    400: "bad request: the request is malformed",
    403: "forbidden: the key file is missing or does not hold the key",
    422: "unprocessable: a URL is not on the host, or the key does not match",
    429: "too many requests: wait before submitting again",
}


def find_key():
    """Return the key whose file `static/<key>.txt` holds exactly the key."""
    keys = [
        path.stem
        for path in STATIC.glob("*.txt")
        if re.fullmatch(r"[A-Za-z0-9-]{8,128}", path.stem)
        and path.read_text().strip() == path.stem
    ]
    if len(keys) != 1:
        sys.exit(f"expected one key file in static/, found {len(keys)}")
    return keys[0]


def fetch(url):
    """Return the body of `url`, or None when it does not answer 200."""
    try:
        with urllib.request.urlopen(url, timeout=30) as response:
            return response.read().decode()
    except (OSError, UnicodeDecodeError):
        return None


def live_key_matches(key):
    """Return whether the live site serves the key file the engines will check."""
    live = fetch(f"{SITE}/{key}.txt")
    return live is not None and live.strip() == key


def stub_urls():
    """Return the site URL of each redirect stub under `static/`."""
    urls = []
    for page in sorted(STATIC.rglob("index.html")):
        if 'http-equiv="refresh"' in page.read_text():
            path = page.parent.relative_to(STATIC).as_posix()
            urls.append(f"{SITE}/{path}/")
    return urls


def all_urls(origin):
    """Return every URL in the sitemap at `origin`, then every redirect stub."""
    sitemap = fetch(f"{origin}/sitemap.xml")
    if sitemap is None:
        sys.exit(f"could not read {origin}/sitemap.xml")
    urls = re.findall(r"<loc>([^<]+)</loc>", sitemap)
    return list(dict.fromkeys(urls + stub_urls()))


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("urls", nargs="*", help="the changed pages' URLs")
    parser.add_argument("--all", action="store_true", help="every sitemap URL and redirect stub")
    parser.add_argument("--dry-run", action="store_true", help="print the list without sending")
    parser.add_argument("--from", dest="origin", default=SITE, help="where to read the sitemap")
    parser.add_argument("--endpoint", default=ENDPOINT, help="the IndexNow endpoint")
    args = parser.parse_args()

    if args.all == bool(args.urls):
        parser.error("name the changed URLs, or pass --all")
    key = find_key()
    urls = all_urls(args.origin.rstrip("/")) if args.all else args.urls

    foreign = [u for u in urls if urlsplit(u).scheme != "https" or urlsplit(u).netloc != HOST]
    if foreign:
        sys.exit("not on https://" + HOST + ":\n  " + "\n  ".join(foreign))
    if len(urls) > MAX_URLS:
        sys.exit(f"{len(urls)} URLs exceed IndexNow's limit of {MAX_URLS} per request")

    key_ok = live_key_matches(key)
    key_status = "matches" if key_ok else "is missing or differs from static/"
    count = f"{len(urls)} URL" if len(urls) == 1 else f"{len(urls)} URLs"

    if args.dry_run:
        print("\n".join(urls))
        print(f"{count}; the live key file {key_status}")
        return 0
    if not key_ok:
        sys.exit(f"not sent: the live key file {key_status}; deploy it first")

    body = json.dumps(
        {"host": HOST, "key": key, "keyLocation": f"{SITE}/{key}.txt", "urlList": urls}
    ).encode()
    request = urllib.request.Request(
        args.endpoint,
        data=body,
        headers={"Content-Type": "application/json; charset=utf-8"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            status, detail = response.status, response.read()
    except urllib.error.HTTPError as error:
        status, detail = error.code, error.read()
    except OSError as error:
        sys.exit(f"not sent: {getattr(error, 'reason', error)}")
    print(f"{count} sent to {args.endpoint}: {status} {MEANINGS.get(status, 'unexpected')}")
    detail = detail.decode(errors="replace").strip()
    if detail:
        print(f"the endpoint said: {detail[:1000]}")
    return 0 if status in (200, 202) else 1


if __name__ == "__main__":
    sys.exit(main())
