"""Mint a VM-only MCP key locally without copying the SSH private key."""
import base64, json, pathlib, subprocess, time, uuid, os

private = pathlib.Path('/Users/rohanth/Documents/ChatGPT/ghostkeys/.private')
private.mkdir(mode=0o700, exist_ok=True)
client = uuid.uuid4().hex
payload = json.dumps({'exp':int(time.time())+365*86400,'cmds':[],'ctx':{'role':'ghostkeys-mcp','id':client}},separators=(',',':')).encode()
payload_file = private / 'ghostkeys.payload'
payload_file.write_bytes(payload)
subprocess.run(['ssh-keygen','-Y','sign','-f','/Users/rohanth/.ssh/id_ed25519_exe_personal','-n','v0@ghostkeys.exe.xyz',str(payload_file)],check=True,capture_output=True)
signature = ''.join(line for line in (private/'ghostkeys.payload.sig').read_text().splitlines() if not line.startswith('-----'))
encode = lambda b:base64.urlsafe_b64encode(b).decode().rstrip('=')
token = 'exe0.'+encode(payload)+'.'+encode(base64.b64decode(signature))
connection = {'url':'https://ghostkeys.exe.xyz/mcp','headers':{'X-Exedev-Authorization':'Bearer '+token}}
for name, value in [('mcp-connection.json',json.dumps(connection,indent=2)),('client-id',client)]:
    p=private/name
    p.write_text(value+'\n')
    os.chmod(p,0o600)
print('Ghostkeys MCP client key saved privately; expires in one year.')
