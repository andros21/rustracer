#!/usr/bin/env python

# md.py
# =========
# update `README.md` console help output from `TEMPLATE.md`

import os
import subprocess
from string import Template

if __name__ == "__main__":
    os.environ["COLUMNS"] = "150"
    rustracer = {
        "rustracer": subprocess.run(
            ["target/debug/rustracer", "-h"],
            capture_output=True,
            encoding="utf8",
            check=False,
        ).stdout,
        "rustracer_convert": subprocess.run(
            ["target/debug/rustracer", "convert", "-h"],
            capture_output=True,
            encoding="utf8",
            check=False,
        ).stdout,
        "rustracer_demo": subprocess.run(
            ["target/debug/rustracer", "demo", "-h"],
            capture_output=True,
            encoding="utf8",
            check=False,
        ).stdout,
        "rustracer_render": subprocess.run(
            ["target/debug/rustracer", "render", "-h"],
            capture_output=True,
            encoding="utf8",
            check=False,
        ).stdout,
        "rustracer_completion": subprocess.run(
            ["target/debug/rustracer", "completion", "-h"],
            capture_output=True,
            encoding="utf8",
            check=False,
        ).stdout,
    }

    with open("TEMPLATE.md") as ifs:
        template = Template(ifs.read())
    with open("README.md", "w") as ofs:
        ofs.write(template.substitute(rustracer))
