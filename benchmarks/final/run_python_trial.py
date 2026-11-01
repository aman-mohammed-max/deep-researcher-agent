import asyncio
import json
import os
import sys
import time

import psutil
from dotenv import load_dotenv

load_dotenv("deploy/.env")
os.environ["HOME"] = "/tmp"


async def auto_approval_callback(prompt):
    from nat.data_models.human_in_the_loop import HumanResponseText

    return HumanResponseText(text="Proceed with the proposed deep research plan.")


async def run_python_trial(prompt_path: str, output_path: str):
    with open(prompt_path, encoding="utf-8") as f:
        query = f.read().strip()

    print("\n==========================================")
    print(f"Running Python Trial: {os.path.basename(prompt_path)}")
    print("==========================================")

    process = psutil.Process()
    peak_rss_box = [process.memory_info().rss / (1024 * 1024)]

    t0 = time.time()
    from nat.runtime.loader import load_workflow

    config_path = "configs/config_cli_default.yml"

    status = "RUNNING"
    report = ""
    error_msg = None

    async def monitor_mem():
        try:
            while True:
                await asyncio.sleep(1)
                curr = process.memory_info().rss / (1024 * 1024)
                peak_rss_box[0] = max(peak_rss_box[0], curr)
                elapsed = time.time() - t0
                print(f"  [{elapsed:.1f}s] Python running | Peak RSS: {peak_rss_box[0]:.1f}MB", end="\r", flush=True)
        except asyncio.CancelledError:
            pass

    try:
        async with load_workflow(config_path) as session_manager:
            curr_rss = process.memory_info().rss / (1024 * 1024)
            peak_rss_box[0] = max(peak_rss_box[0], curr_rss)

            async with session_manager.session(user_input_callback=auto_approval_callback) as session:
                curr_rss = process.memory_info().rss / (1024 * 1024)
                peak_rss_box[0] = max(peak_rss_box[0], curr_rss)

                async with session.run(query) as runner:
                    monitor_task = asyncio.create_task(monitor_mem())
                    try:
                        report = await runner.result(to_type=str)
                        status = "COMPLETED"
                    finally:
                        monitor_task.cancel()
                        try:
                            await monitor_task
                        except asyncio.CancelledError:
                            pass

    except Exception as e:
        status = "FAILED"
        error_msg = str(e)
        print(f"\nPython execution error: {e}")

    wall_time = time.time() - t0
    peak_rss = peak_rss_box[0]
    print(f"\nFinished in {wall_time:.2f}s with status: {status} | Peak RSS: {peak_rss:.1f}MB")

    metrics = {
        "engine": "Python (NAT + LangChain)",
        "prompt_file": prompt_path,
        "status": status,
        "error": error_msg,
        "wall_time_sec": round(wall_time, 2),
        "peak_rss_mb": round(peak_rss, 1),
        "word_count": len(report.split()),
        "char_count": len(report),
        "report": report,
    }

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(metrics, f, indent=2)

    print(f"Saved run metrics to {output_path}")
    print(f"Words: {metrics['word_count']} | RSS: {metrics['peak_rss_mb']}MB")
    return metrics


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python run_python_trial.py <prompt_file> <output_file>")
        sys.exit(1)
    asyncio.run(run_python_trial(sys.argv[1], sys.argv[2]))
