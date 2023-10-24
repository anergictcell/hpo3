#!/usr/bin/env python3

"""
Checks that the tag-version matches the Python package version in pyproject.toml
and that the version does not yet exist on PyPI
"""


import os
from urllib import request
from urllib.error import HTTPError
import re
import sys
import tomllib
from typing import Optional


def version_from_tag() -> Optional[str]:
    version = os.getenv('GITHUB_REF')
    if version:
        version = re.sub('^refs/tags/v*', '', version.lower())
        return version
    else:
        print("No version tag found")
        return None


def version_from_toml() -> Optional[str]:
    with open("Cargo.toml", "rb") as f:
        data = tomllib.load(f)
        try:
            return data['package']['version']
        except KeyError:
            print("No version defined in pyproject.toml")
            return None


def present_in_pypi(version: str) -> bool:
    url = f"https://pypi.org/pypi/hpo3/{version}/json"
    print(f"Checking {url}")
    try:
        with request.urlopen(url) as response:
            if response.status == 200:
                print(f"Version {version} is already present in PyPi")
                return True
            else:
                print(f"PyPI did not respond correctly for version {version}")
                print(f"Received response status {response.status}")
                return True
    except HTTPError:
        # In theory, the request could fail for many reasons, but most likely
        # it means that the version is not yet present in PyPI
        return False


def main():
    git_tag = version_from_tag()
    if git_tag is None:
        return 1
    toml_version = version_from_toml()
    if toml_version is None:
        return 1
    if git_tag == toml_version:
        if present_in_pypi(git_tag):
            return 1
        else:
            return 0
    else:
        print(f"Different versions in tag ({git_tag}) and toml ({toml_version})")
        return 1


if __name__ == "__main__":
    sys.exit(main())
