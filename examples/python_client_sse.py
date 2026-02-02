import requests
import json
import sys
import threading
import time

try:
    import sseclient
except ImportError:
    print("Please install sseclient-py: pip install sseclient-py")
    sys.exit(1)

SERVER_URL = "http://127.0.0.1:8080"
SSE_URL = f"{SERVER_URL}/sse"

def listen_sse(url, event_container):
    print(f"Listening to SSE at {url}...")
    response = requests.get(url, stream=True)
    client = sseclient.SSEClient(response)
    for msg in client.events():
        print(f"\n[SSE] Event: {msg.event}")
        if msg.data:
            print(f"[SSE] Data: {msg.data}")
        
        if msg.event == 'endpoint':
            event_container['endpoint'] = msg.data
        elif msg.event == 'message':
            # This is where responses come
            try:
                data = json.loads(msg.data)
                print(f"[SSE] Message JSON: {json.dumps(data, indent=2)}")
            except:
                pass

def run_test():
    event_container = {}
    t = threading.Thread(target=listen_sse, args=(SSE_URL, event_container))
    t.daemon = True
    t.start()
    
    print("Waiting for endpoint...")
    while 'endpoint' not in event_container:
        time.sleep(0.1)
    
    post_endpoint = event_container['endpoint']
    print(f"Got POST endpoint: {post_endpoint}")
    
    full_post_url = f"{SERVER_URL}{post_endpoint}"
    print(f"Sending requests to: {full_post_url}")
    
    # Test 1: Initialize
    init_req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "1.0"}
        }
    }
    
    print("Sending Initialize...")
    requests.post(full_post_url, json=init_req)
    
    time.sleep(1) # Wait for init response
    
    # Test 2: Call find_crates
    call_req = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "find_crates",
            "arguments": {
                "query": "serde",
                "limit": 1
            }
        }
    }
    print("Sending find_crates...")
    requests.post(full_post_url, json=call_req)
    
    time.sleep(2) # Wait for response
    
    # Test 3: Call get_crate_overview
    call_req_2 = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_crate_overview",
            "arguments": {
                "crate_name": "serde"
            }
        }
    }
    print("Sending get_crate_overview...")
    requests.post(full_post_url, json=call_req_2)
    
    time.sleep(2) # Wait for response

if __name__ == "__main__":
    run_test()
