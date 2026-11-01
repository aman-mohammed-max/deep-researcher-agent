import asyncio
import os

from dotenv import load_dotenv

load_dotenv("deploy/.env")
os.environ["HOME"] = "/tmp"


async def test_run():
    from nat.runtime.loader import load_workflow

    print("Testing load_workflow...")
    async with load_workflow("configs/config_cli_default.yml"):
        print("Session manager loaded successfully!")


if __name__ == "__main__":
    asyncio.run(test_run())
