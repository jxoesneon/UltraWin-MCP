import json
import subprocess
import sys
import os
import time

class MCPClient:
    def __init__(self, executable_path):
        self.executable_path = executable_path
        self.process = None
        self.request_id = 0

    def start(self):
        print(f"[*] Launching {self.executable_path}...")
        self.process = subprocess.Popen(
            [self.executable_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=sys.stderr, # Forward stderr to see server logs
            bufsize=0
        )
        self._handshake()

    def _send(self, message):
        content = json.dumps(message)
        header = f"Content-Length: {len(content)}\r\n\r\n"
        self.process.stdin.write(header.encode('ascii'))
        self.process.stdin.write(content.encode('utf-8'))
        self.process.stdin.flush()

    def _receive(self):
        # Header reading
        line = self.process.stdout.readline().decode('ascii')
        if not line:
            return None
        
        if line.startswith("Content-Length:"):
            length = int(line.split(":")[1].strip())
            # Read until the double newline
            while True:
                next_line = self.process.stdout.readline().decode('ascii')
                if next_line.strip() == "":
                    break
            
            content = self.process.stdout.read(length).decode('utf-8')
            return json.loads(content)
        return None

    def _handshake(self):
        self.request_id += 1
        print("[*] Sending 'initialize'...")
        self._send({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {}, "implementation": {"name": "SimClient", "version": "1.0"},
                "clientInfo": {"name": "client_sim", "version": "1.0.0"}
            }
        })
        
        resp = self._receive()
        if resp and "result" in resp:
            print("[+] Initialized successfully.")
            self._send({
                "jsonrpc": "2.0",
                "method": "initialized"
            })
        else:
            print(f"[-] Initialization failed: {resp}")

    def call_tool(self, name, arguments=None):
        self.request_id += 1
        self._send({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments or {}
            }
        })
        return self._receive()

def main():
    exe = os.path.join(os.getcwd(), "target", "release", "ultrawin-mcp.exe")
    if not os.path.exists(exe):
        exe = r"C:\Users\josee\UltraWin-MCP\target\release\ultrawin-mcp.exe"
    
    if not os.path.exists(exe):
        print(f"Error: Binary not found at {exe}")
        sys.exit(1)

    client = MCPClient(exe)
    client.start()

    print("\nUltraWin-MCP Client Simulator")
    print("Available commands: screenshot, ui_tree, click <x> <y>, exit")
    
    try:
        while True:
            try:
                cmd_input = input("\n> ").strip().replace("\ufeff", "")
            except EOFError:
                break
                
            if not cmd_input:
                continue
            
            parts = cmd_input.split()
            cmd = parts[0].lower()

            if cmd == "exit":
                break
            elif cmd == "screenshot":
                res = client.call_tool("screenshot")
                print(json.dumps(res, indent=2))
            elif cmd == "ui_tree":
                res = client.call_tool("get_ui_tree")
                print(json.dumps(res, indent=2))
            elif cmd == "click":
                if len(parts) < 3:
                    print("Usage: click <x> <y>")
                    continue
                try:
                    x, y = int(parts[1]), int(parts[2])
                    res = client.call_tool("click", {"x": x, "y": y})
                    print(json.dumps(res, indent=2))
                except ValueError:
                    print("Error: x and y must be integers.")
            else:
                # Generic tool call fallback
                arg_json = cmd_input.replace(parts[0], "", 1).strip(); args = json.loads(arg_json) if arg_json else {}; res = client.call_tool(cmd, args)
                print(json.dumps(res, indent=2))
                
    except KeyboardInterrupt:
        pass
    finally:
        if client.process:
            client.process.terminate()

if __name__ == '__main__':
    main()



