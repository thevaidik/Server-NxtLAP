import urllib.request
import json
import ssl
import sys
import time
from datetime import datetime

# Disable SSL verification for simplicity
# context = ssl._create_unverified_context()

BASE_URL = "https://brto98doc9.execute-api.us-east-1.amazonaws.com"

endpoints = {
    # limit to one endpoint for detailed introspection to avoid massive output
    "All Races": "/races", 
}

def inspect_ttl(data):
    current_time = int(time.time())
    print(f"Current Time (Epoch): {current_time} ({datetime.fromtimestamp(current_time)})")
    
    count_2025 = 0
    expired_count = 0
    
    for event in data:
        date_str = event.get('date', '')
        year = date_str[:4] if len(date_str) >= 4 else "Unknown"
        
        if year == "2025":
            count_2025 += 1
            ttl = event.get('ttl')
            event_name = event.get('event_name', 'Unknown')
            
            if ttl:
                is_expired = ttl < current_time
                if is_expired:
                    expired_count += 1
                
                # Print details for the first few 2025 events
                if count_2025 <= 5:
                     status = "EXPIRED" if is_expired else "VALID"
                     ttl_date = datetime.fromtimestamp(ttl)
                     print(f"  [{year}] {event_name[:30]}... | TTL: {ttl} ({ttl_date}) | {status}")
            else:
                 if count_2025 <= 5:
                    print(f"  [{year}] {event_name[:30]}... | TTL: NOT SET")

    print(f"\nSummary for 2025 events:")
    print(f"  Total 2025 events: {count_2025}")
    print(f"  Expired (TTL < Now): {expired_count}")
    print(f"  Valid (TTL > Now): {count_2025 - expired_count}")

def test_endpoint(name, path):
    url = f"{BASE_URL}{path}"
    try:
        req = urllib.request.Request(url)
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            
            if isinstance(data, list):
                inspect_ttl(data)
            else:
                print(f"⚠️  {name:15} : Not a list")

    except Exception as e:
        print(f"❌ {name:15} : Failed - {str(e)}")

print(f"Inspecting TTL for 2025 data at {BASE_URL}...\n")

for name, path in endpoints.items():
    test_endpoint(name, path)
