#!/usr/bin/env python3
"""greet — a tiny CLI that says hello to someone by name.

This is the scratch project for the AIDA onboarding tour. It is
deliberately small: the tour is about AIDA, not about this code.
"""

import argparse


def greet(name):
    """Build the greeting line for `name`."""
    return f"Hello, {name}!"


def main():
    parser = argparse.ArgumentParser(description="Say hello to someone.")
    parser.add_argument("name", help="who to greet")
    args = parser.parse_args()
    print(greet(args.name))


if __name__ == "__main__":
    main()
