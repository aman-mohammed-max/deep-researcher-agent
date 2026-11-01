import json
import os
import sys
import time
import urllib.request


def run_rust_trial(prompt_path: str, output_path: str):
    with open(prompt_path, encoding="utf-8") as f:
        query = f.read().strip()

    print("\n==========================================")
    print(f"Running Rust Trial: {os.path.basename(prompt_path)}")
    print("==========================================")

    t0 = time.time()
    req = urllib.request.Request(
        "http://localhost:8000/v1/jobs/async",
        data=json.dumps({"query": query}).encode("utf-8"),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req) as resp:
        submit_data = json.loads(resp.read().decode("utf-8"))

    job_id = submit_data["job_id"]
    print(f"Job submitted: ID={job_id}, Status={submit_data['status']}")

    peak_rss = 26
    status = "RUNNING"
    final_job = {}

    # Poll job status until COMPLETED or FAILED
    while True:
        time.sleep(4)
        try:
            with urllib.request.urlopen("http://localhost:8000/health") as h_resp:
                h_data = json.loads(h_resp.read().decode("utf-8"))
                if h_data.get("process_rss_mb", 0) > peak_rss:
                    peak_rss = h_data["process_rss_mb"]
        except Exception:
            pass

        try:
            with urllib.request.urlopen(f"http://localhost:8000/v1/jobs/async/{job_id}") as j_resp:
                final_job = json.loads(j_resp.read().decode("utf-8"))
                status = final_job.get("status", "RUNNING")
                if status in ("COMPLETED", "FAILED"):
                    break
        except Exception as e:
            print(f"Polling warning: {e}")

        elapsed = time.time() - t0
        print(f"  [{elapsed:.1f}s] Status: {status} | Peak RSS: {peak_rss}MB", end="\r")

    wall_time = time.time() - t0
    print(f"\nFinished in {wall_time:.2f}s with status: {status}")

    report = final_job.get("report", "")
    metrics = {
        "engine": "Rust (Upgraded 1:1 Parity)",
        "prompt_file": prompt_path,
        "job_id": job_id,
        "status": status,
        "wall_time_sec": round(wall_time, 2),
        "peak_rss_mb": peak_rss,
        "prompt_tokens": final_job.get("prompt_tokens", 0),
        "completion_tokens": final_job.get("completion_tokens", 0),
        "total_tokens": final_job.get("total_tokens", 0),
        "word_count": len(report.split()),
        "char_count": len(report),
        "created_at": final_job.get("created_at"),
        "updated_at": final_job.get("updated_at"),
        "report": report,
    }

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(metrics, f, indent=2)

    print(f"Saved run metrics to {output_path}")
    print(f"Words: {metrics['word_count']} | Tokens: {metrics['total_tokens']} | RSS: {metrics['peak_rss_mb']}MB")
    return metrics


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python run_rust_trial.py <prompt_file> <output_file>")
        sys.exit(1)
    run_rust_trial(sys.argv[1], sys.argv[2])
