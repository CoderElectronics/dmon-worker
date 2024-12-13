import psutil
import json
import datetime
import os
import platform

def get_disk_info():
    try:
        disk_info = {
            "timestamp": datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            "system_info": {
                "system": platform.system(),
                "node": platform.node(),
                "release": platform.release()
            },
            "disks": []
        }

        # Get all disk partitions
        partitions = psutil.disk_partitions()

        for partition in partitions:
            try:
                # Skip CD-ROM drives in Windows if no disk is inserted
                if 'cdrom' in partition.opts or partition.fstype == '':
                    continue

                usage = psutil.disk_usage(partition.mountpoint)

                # Convert bytes to GB
                total_gb = usage.total / (1024 ** 3)
                used_gb = usage.used / (1024 ** 3)
                free_gb = usage.free / (1024 ** 3)

                disk_data = {
                    "device": partition.device,
                    "mountpoint": partition.mountpoint,
                    "filesystem_type": partition.fstype,
                    "status": {
                        "total_gb": round(total_gb, 2),
                        "used_gb": round(used_gb, 2),
                        "free_gb": round(free_gb, 2),
                        "usage_percent": usage.percent,
                        "is_critical": usage.percent > 90  # Flag if usage is above 90%
                    },
                    "opts": partition.opts
                }

                # Add warning levels
                if usage.percent >= 90:
                    disk_data["warning_level"] = "CRITICAL"
                elif usage.percent >= 80:
                    disk_data["warning_level"] = "WARNING"
                elif usage.percent >= 70:
                    disk_data["warning_level"] = "CAUTION"
                else:
                    disk_data["warning_level"] = "NORMAL"

                disk_info["disks"].append(disk_data)

            except (PermissionError, FileNotFoundError):
                continue

        # Add summary section
        disk_info["summary"] = {
            "total_disks": len(disk_info["disks"]),
            "critical_disks": sum(1 for disk in disk_info["disks"]
                                if disk["warning_level"] == "CRITICAL"),
            "warning_disks": sum(1 for disk in disk_info["disks"]
                               if disk["warning_level"] == "WARNING"),
            "total_storage_gb": round(sum(disk["status"]["total_gb"]
                                        for disk in disk_info["disks"]), 2),
            "total_used_gb": round(sum(disk["status"]["used_gb"]
                                     for disk in disk_info["disks"]), 2),
            "total_free_gb": round(sum(disk["status"]["free_gb"]
                                     for disk in disk_info["disks"]), 2)
        }

        return json.dumps(disk_info)

    except Exception as e:
        error_response = {
            "error": str(e),
            "timestamp": datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        }
        return json.dumps(error_response, indent=4)

def main():
    # Get disk information
    disk_info = get_disk_info()

    # Print to console
    print(disk_info)

if __name__ == "__main__":
    main()
