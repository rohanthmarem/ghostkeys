#!/usr/bin/env python3
"""Call the VM's existing private MCP over the owner's SSH connection."""
import json
from pathlib import Path
import shlex
import subprocess
import sys
import base64

args = json.loads(Path(sys.argv[2][1:]).read_text() if sys.argv[2].startswith('@') else sys.argv[2]) if len(sys.argv) > 2 else {}
request = {"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": sys.argv[1], "arguments": args}}
script = "import fs from 'node:fs'; const body=fs.readFileSync(0,'utf8'); const r=await fetch('http://127.0.0.1:8790/mcp',{method:'POST',headers:{'X-Exedev-Email':'rohanth.marem@gmail.com','Content-Type':'application/json','Accept':'application/json, text/event-stream'},body}); process.stdout.write(await r.text());"
command = ['/usr/bin/ssh', '-o', 'BatchMode=yes', '-i', '/Users/rohanth/.ssh/id_ed25519_exe_personal', '-o', 'IdentitiesOnly=yes', 'vm+ghostkeys@vm.exe.xyz', '/home/exedev/.local/node/bin/node --input-type=module -e ' + shlex.quote(script)]
result = subprocess.run(command, input=json.dumps(request), capture_output=True, text=True, timeout=70, check=True)
response = json.loads(result.stdout)
if response.get('error'):
    raise RuntimeError(response['error'])
for item in response['result']['content']:
    if item['type'] == 'text':
        print(item['text'])
    elif item['type'] == 'image':
        target = Path('verification/word-current.png')
        target.parent.mkdir(exist_ok=True)
        target.write_bytes(base64.b64decode(item['data']))
        print(f'Screenshot: {target.resolve()}')
if response['result'].get('isError'):
    sys.exit(1)
