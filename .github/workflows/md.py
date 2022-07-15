#!/usr/bin/env python

# md.py
# =========
# update `README.md` console help output from `TEMPLATE.md`

import subprocess

from jinja2 import Template

if __name__ == "__main__":
    rustracer = {
        "rustracer": subprocess.run(
            "target/release/rustracer -h".split(), capture_output=True, encoding="utf8"
        ).stdout,
        "rustracer_convert": subprocess.run(
            "target/release/rustracer convert -h".split(),
            capture_output=True,
            encoding="utf8",
        ).stdout,
        "rustracer_demo": subprocess.run(
            "target/release/rustracer demo -h".split(),
            capture_output=True,
            encoding="utf8",
        ).stdout,
        "rustracer_render": subprocess.run(
            "target/release/rustracer render -h".split(),
            capture_output=True,
            encoding="utf8",
        ).stdout,
        "rustracer_completion": subprocess.run(
            "target/release/rustracer completion -h".split(),
            capture_output=True,
            encoding="utf8",
        ).stdout,
    }

    with open("TEMPLATE.md") as ifs:
        template = Template(ifs.read())
    with open("README.md", "w") as ofs:
        ofs.write(template.render(rustracer) + "\n")
