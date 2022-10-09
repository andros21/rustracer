#!/usr/bin/env python

# md.py
# =========
# update `README.md` console help output from `TEMPLATE.md`

import os
import subprocess

from jinja2 import Template

if __name__ == "__main__":
    os.environ["COLUMNS"] = "150"
    rustracer = {
        "rustracer": subprocess.run(
            "target/debug/rustracer -h".split(), capture_output=True, encoding="utf8"
        ).stdout,
        "rustracer_convert": subprocess.run(
            "target/debug/rustracer convert -h".split(),
            capture_output=True,
            encoding="utf8",
        ).stdout,
        "rustracer_demo": subprocess.run(
            "target/debug/rustracer demo -h".split(),
            capture_output=True,
            encoding="utf8",
        ).stdout,
        "rustracer_render": subprocess.run(
            "target/debug/rustracer render -h".split(),
            capture_output=True,
            encoding="utf8",
        ).stdout,
        "rustracer_completion": subprocess.run(
            "target/debug/rustracer completion -h".split(),
            capture_output=True,
            encoding="utf8",
        ).stdout,
    }

    with open("TEMPLATE.md") as ifs:
        template = Template(ifs.read())
    with open("README.md", "w") as ofs:
        ofs.write(template.render(rustracer) + "\n")
