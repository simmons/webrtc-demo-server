#!/usr/bin/env python3

import sys

def chomp(x):
    if x.endswith("\r\n"): return x[:-2]
    if x.endswith("\n") or x.endswith("\r"): return x[:-1]
    return x

print("static WORDS: &[&str] = &[")
for word in sys.stdin.readlines():
    word = chomp(word)
    print("    \""+word+"\",")
print("];")
