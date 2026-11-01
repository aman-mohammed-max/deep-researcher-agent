import asyncio
import os
import time

from dotenv import load_dotenv

load_dotenv("deploy/.env")
os.environ["HOME"] = "/tmp"


async def auto_callback(prompt):
    print(f"Auto-responding to prompt: {getattr(prompt.content, 'text', '')[:100]}")
    from nat.data_models.human_in_the_loop import HumanResponseText

    return HumanResponseText(text="Proceed with the proposed research plan without modifications.")


async def test_session():
    from nat.runtime.loader import load_workflow

    config_path = "configs/config_cli_default.yml"
    print(f"Loading {config_path}...")
    async with load_workflow(config_path) as session_manager:
        print("Session manager ready. Creating session...")
        async with session_manager.session(user_input_callback=auto_callback) as session:
            print("Session created. Testing simple query...")
            t0 = time.time()
            async with session.run("What are the key elements of post-quantum cryptography?") as runner:
                res = await runner.result(to_type=str)
                print(f"Response in {time.time() - t0:.2f}s:\n{res[:500]}")


if __name__ == "__main__":
    asyncio.run(test_session())
