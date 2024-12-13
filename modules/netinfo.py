#!/usr/bin/env python3

import json
import socket
import psutil
import netifaces
import subprocess
import platform
from datetime import datetime

def get_network_info():
    network_info = {
        "timestamp": datetime.now().isoformat(),
        "hostname": socket.gethostname(),
        "fqdn": socket.getfqdn(),
        "platform": platform.system(),
        "interfaces": {},
        "default_gateway": {},
        "dns_servers": [],
        "connections": [],
        "ping_stats": {}
    }

    # Get network interfaces information
    for interface in netifaces.interfaces():
        try:
            addrs = netifaces.ifaddresses(interface)
            interface_info = {
                "mac_address": addrs.get(netifaces.AF_LINK, [{}])[0].get('addr', ''),
                "ipv4": [],
                "ipv6": [],
                "status": "down"
            }

            # Get IPv4 addresses
            if netifaces.AF_INET in addrs:
                for addr in addrs[netifaces.AF_INET]:
                    interface_info["ipv4"].append({
                        "address": addr.get('addr', ''),
                        "netmask": addr.get('netmask', ''),
                        "broadcast": addr.get('broadcast', '')
                    })
                    interface_info["status"] = "up"

            # Get IPv6 addresses
            if netifaces.AF_INET6 in addrs:
                for addr in addrs[netifaces.AF_INET6]:
                    interface_info["ipv6"].append({
                        "address": addr.get('addr', ''),
                        "netmask": addr.get('netmask', '')
                    })

            network_info["interfaces"][interface] = interface_info

        except Exception as e:
            network_info["interfaces"][interface] = {"error": str(e)}

    # Get default gateway
    try:
        gateways = netifaces.gateways()
        default = gateways.get('default', {})
        if netifaces.AF_INET in default:
            network_info["default_gateway"]["ipv4"] = default[netifaces.AF_INET][0]
        if netifaces.AF_INET6 in default:
            network_info["default_gateway"]["ipv6"] = default[netifaces.AF_INET6][0]
    except Exception as e:
        network_info["default_gateway"]["error"] = str(e)

    # Get DNS servers
    try:
        with open('/etc/resolv.conf', 'r') as f:
            for line in f:
                if line.startswith('nameserver'):
                    network_info["dns_servers"].append(line.split()[1])
    except Exception as e:
        network_info["dns_servers"] = ["Error reading DNS servers: " + str(e)]

    # Get network connections
    try:
        connections = psutil.net_connections()
        for conn in connections:
            if conn.status == 'ESTABLISHED':
                network_info["connections"].append({
                    "local_address": f"{conn.laddr.ip}:{conn.laddr.port}",
                    "remote_address": f"{conn.raddr.ip}:{conn.raddr.port}",
                    "status": conn.status,
                    "pid": conn.pid
                })
    except Exception as e:
        network_info["connections"] = ["Error getting connections: " + str(e)]

    # Ping test to common DNS servers
    ping_targets = ['8.8.8.8', '1.1.1.1']
    for target in ping_targets:
        try:
            if platform.system().lower() == 'windows':
                ping_param = '-n'
            else:
                ping_param = '-c'

            result = subprocess.run(
                ['ping', ping_param, '1', target],
                capture_output=True,
                text=True,
                timeout=5
            )
            network_info["ping_stats"][target] = {
                "success": result.returncode == 0,
                "output": result.stdout.strip()
            }
        except Exception as e:
            network_info["ping_stats"][target] = {
                "success": False,
                "error": str(e)
            }

    return network_info

if __name__ == "__main__":
    try:
        network_info = get_network_info()
        print(json.dumps(network_info, indent=2))
    except Exception as e:
        error_response = {
            "error": str(e),
            "timestamp": datetime.now().isoformat()
        }
        print(json.dumps(error_response, indent=2))
