import psutil
import json
import datetime

def get_system_metrics():
    try:
        # Get CPU usage percentage
        cpu_percent = psutil.cpu_percent(interval=1)

        # Get memory usage
        memory = psutil.virtual_memory()
        memory_total = memory.total / (1024.0 ** 3)     # Convert to GB
        memory_used = memory.used / (1024.0 ** 3)       # Convert to GB
        memory_percent = memory.percent

        # Get current timestamp
        timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")

        # Create metrics dictionary
        metrics = {
            "timestamp": timestamp,
            "cpu": {
                "usage_percent": round(cpu_percent, 2)
            },
            "memory": {
                "total_gb": round(memory_total, 2),
                "used_gb": round(memory_used, 2),
                "usage_percent": round(memory_percent, 2)
            }
        }

        # Convert to JSON
        metrics_json = json.dumps(metrics)
        return metrics_json

    except Exception as e:
        error_response = {
            "error": str(e),
            "timestamp": datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        }
        return json.dumps(error_response, indent=4)

def main():
    # Get and print metrics
    metrics = get_system_metrics()
    print(metrics)

if __name__ == "__main__":
    main()
