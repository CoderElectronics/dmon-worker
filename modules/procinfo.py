#!/usr/bin/env python3

import json
import psutil
import pwd
import datetime
import os
import platform
from subprocess import check_output, PIPE, CalledProcessError

def get_user_process_info():
    info = {
        "timestamp": datetime.datetime.now().isoformat(),
        "hostname": platform.node(),
        "platform": {
            "system": platform.system(),
            "release": platform.release(),
            "version": platform.version(),
            "machine": platform.machine()
        },
        "logged_in_users": [],
        "processes": {
            "total_count": 0,
            "by_user": {},
            "top_memory": [],
            "top_cpu": [],
            "system_resources": {},
            "process_list": []
        }
    }

    # Get logged-in users
    try:
        if platform.system() == 'Linux':
            who_output = check_output(['who'], universal_newlines=True).split('\n')
            for line in who_output:
                if line:
                    parts = line.split()
                    user_session = {
                        "username": parts[0],
                        "terminal": parts[1],
                        "login_time": ' '.join(parts[2:4]),
                        "host": parts[4] if len(parts) > 4 else "local"
                    }
                    info["logged_in_users"].append(user_session)
    except Exception as e:
        info["logged_in_users"] = [{"error": f"Failed to get logged-in users: {str(e)}"}]

    # System resources
    try:
        info["processes"]["system_resources"] = {
            "cpu_percent": psutil.cpu_percent(interval=1),
            "memory": {
                "total": psutil.virtual_memory().total,
                "available": psutil.virtual_memory().available,
                "percent": psutil.virtual_memory().percent
            },
            "swap": {
                "total": psutil.swap_memory().total,
                "used": psutil.swap_memory().used,
                "percent": psutil.swap_memory().percent
            }
        }
    except Exception as e:
        info["processes"]["system_resources"] = {"error": f"Failed to get system resources: {str(e)}"}

    # Process information
    try:
        processes = []
        user_process_count = {}

        for proc in psutil.process_iter(['pid', 'name', 'username', 'cpu_percent',
                                       'memory_percent', 'create_time', 'status']):
            try:
                pinfo = proc.info
                username = pinfo['username']

                # Count processes by user
                user_process_count[username] = user_process_count.get(username, 0) + 1

                # Get detailed process info
                process_detail = {
                    "pid": pinfo['pid'],
                    "name": pinfo['name'],
                    "username": username,
                    "cpu_percent": pinfo['cpu_percent'],
                    "memory_percent": pinfo['memory_percent'],
                    "status": pinfo['status'],
                    "create_time": datetime.datetime.fromtimestamp(pinfo['create_time']).isoformat(),
                }

                # Try to get additional details
                try:
                    process = psutil.Process(pinfo['pid'])
                    process_detail.update({
                        "command_line": process.cmdline(),
                        "num_threads": process.num_threads(),
                        "memory_info": {
                            "rss": process.memory_info().rss,
                            "vms": process.memory_info().vms
                        }
                    })
                except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                    pass

                processes.append(process_detail)

            except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                continue

        # Sort processes by CPU and memory usage
        processes_sorted_cpu = sorted(processes, key=lambda x: x['cpu_percent'] or 0, reverse=True)
        processes_sorted_memory = sorted(processes, key=lambda x: x['memory_percent'] or 0, reverse=True)

        info["processes"].update({
            "total_count": len(processes),
            "by_user": user_process_count,
            "top_cpu": processes_sorted_cpu[:10],
            "top_memory": processes_sorted_memory[:10],
            "process_list": processes
        })

    except Exception as e:
        info["processes"]["error"] = f"Failed to get process information: {str(e)}"

    return info

if __name__ == "__main__":
    try:
        info = get_user_process_info()
        print(json.dumps(info, indent=2))
    except Exception as e:
        error_response = {
            "error": str(e),
            "timestamp": datetime.datetime.now().isoformat()
        }
        print(json.dumps(error_response, indent=2))
