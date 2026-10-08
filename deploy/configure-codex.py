"""Add the separate remote server while preserving existing Codex settings."""
import json, pathlib, os, tomllib
root = pathlib.Path('/Users/rohanth/Documents/ChatGPT/ghostkeys')
connection = json.loads((root/'.private/mcp-connection.json').read_text())
path = pathlib.Path('/Users/rohanth/.codex/config.toml')
existing = path.read_text()
settings = tomllib.loads(existing)
if 'ghostkeys' in settings.get('mcp_servers',{}):
    raise SystemExit('Ghostkeys is already configured. No changes made.')
backup = root/'.private/codex-config-before-ghostkeys.toml'
with backup.open('x') as f: f.write(existing)
os.chmod(backup,0o600)
header = connection['headers']['X-Exedev-Authorization']
updated = existing + '\n[mcp_servers.ghostkeys]\nurl = "https://ghostkeys.exe.xyz/mcp"\nstartup_timeout_sec = 30\ntool_timeout_sec = 120\nhttp_headers = { "X-Exedev-Authorization" = '+json.dumps(header)+' }\n'
tomllib.loads(updated)
temporary = path.with_name('config.ghostkeys.pending.toml')
with temporary.open('x') as f: f.write(updated)
os.chmod(temporary,0o600)
temporary.replace(path)
print('Ghostkeys added to Codex alongside Waterloo. Existing server settings preserved.')
