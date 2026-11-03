#!/usr/bin/env python3
"""Checks the translation catalogs of fili-file against the English texts of the code.

    scripts/check-i18n.py          (from the source tree of fili-file)

Texts are written in English in the code: tr("...") / trf("...") in src/*.rs and Fili.tr("...") / Fili.trf("...") in
qml/*.qml. Each catalog (i18n/<language>.json) needs an entry per text with the same %1, %2... placeholders; an entry
that nothing uses any more is reported too. "@decimal" is the decimal separator of the language. Exit code 1 on any problem.
"""
import glob
import json
import os
import re
import sys

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
texts = set()
for path in glob.glob(f"{root}/src/*.rs"):
    if path.endswith("i18n.rs"):
        continue
    for m in re.finditer(r'\btrf?\(\s*"((?:[^"\\]|\\.)*)"', open(path).read()):
        texts.add(json.loads('"' + m.group(1) + '"'))
for path in glob.glob(f"{root}/qml/*.qml"):
    for m in re.finditer(r'Fili\.trf?\(\s*"((?:[^"\\]|\\.)*)"', open(path).read()):
        texts.add(json.loads('"' + m.group(1) + '"'))


def placeholders(s):
    return sorted(set(re.findall(r"%[1-9]", s)))


problems = 0
catalogs = sorted(glob.glob(f"{root}/i18n/*.json"))
for catalog in catalogs:
    lang = os.path.basename(catalog)[:-5]
    entries = json.load(open(catalog))
    for t in sorted(texts):
        if t not in entries:
            print(f"{lang}: missing: {t}")
            problems += 1
        elif placeholders(t) != placeholders(entries[t]):
            print(f"{lang}: placeholders differ: {t}")
            problems += 1
    for k in sorted(set(entries) - texts - {"@decimal"}):
        print(f"{lang}: unused entry: {k}")
        problems += 1
print(f"{len(texts)} texts, {len(catalogs)} catalog(s), {problems} problem(s)")
sys.exit(1 if problems else 0)
