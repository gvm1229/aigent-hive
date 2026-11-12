"""Gate a live frozen updater on an actually published matching product."""
import json
import os
from pathlib import Path
import re
import tomllib
import urllib.request


def matching_target(product, package):
    if not isinstance(package, str) or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+-test\.[1-9][0-9]*", package):
        raise ValueError('registry test tag is not a numbered public test')
    return bool(re.fullmatch(re.escape(product) + r"-test\.[1-9][0-9]*", package))


def main():
    root = Path(__file__).resolve().parents[3]
    product = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))['workspace']['package']['version']
    with urllib.request.urlopen('https://registry.npmjs.org/-/package/aigent-hive/dist-tags', timeout=30) as response:
        package = json.load(response)['test']
    ready = matching_target(product, package)
    with open(os.environ['GITHUB_OUTPUT'], 'a', encoding='utf-8') as output:
        output.write('ready=' + str(ready).lower() + '\n')
    print(json.dumps({'product': product, 'registry_test': package, 'legacy_target_ready': ready,
                      'scope': 'actual frozen-updater execution required after matching public test publication'}, ensure_ascii=False))
    if not ready:
        print('::notice::Frozen-updater execution deferred: the public test belongs to a different product. Re-run after matching publication; stable release remains blocked by OV150-018.')


if __name__ == '__main__':
    main()
