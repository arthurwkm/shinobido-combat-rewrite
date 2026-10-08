"""Run every generator: extract the motion files from the disc, then write src/sim/extracted.rs.

    .venv/bin/python tools/setup.py
"""

import subprocess
import sys
from pathlib import Path

T = Path(__file__).parent
for cmd in (["volume.py", "extract", "motion\\"], ["volume.py", "extract", "model\\KAGETORA_BODY.rws"], ["extract.py"], ["pose.py"]):
    subprocess.run([sys.executable, str(T / cmd[0]), *cmd[1:]], check=True)
