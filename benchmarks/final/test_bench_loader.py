import asyncio
import os

from dotenv import load_dotenv

load_dotenv("deploy/.env")
os.environ["HOME"] = "/tmp"


async def test_run():
    from nat.runtime.loader import load_workflow

    config_path = "frontends/benchmarks/deepresearch_bench/configs/config_deep_research_bench.yml"
    print(f"Loading {config_path}...")
    async with load_workflow(config_path):
        print("Bench workflow session manager loaded successfully!")


if __name__ == "__main__":
    asyncio.run(test_run())
