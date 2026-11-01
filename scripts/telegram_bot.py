#!/usr/bin/env python3
"""
Lightweight Serverless Telegram Bot Bridge for Deep Researcher Rust Engine
Integrates directly with the Rust backend via /v1/jobs/async and SSE streaming events.
"""

import asyncio
import os
import sys

import aiohttp

TELEGRAM_BOT_TOKEN = os.getenv("TELEGRAM_BOT_TOKEN")
BACKEND_URL = os.getenv("BACKEND_URL", "http://127.0.0.1:8000")
API_KEY = os.getenv("DEEP_RESEARCHER_API_KEY", "")

if not TELEGRAM_BOT_TOKEN:
    print("Error: TELEGRAM_BOT_TOKEN environment variable is not set.", file=sys.stderr)
    print("Usage: TELEGRAM_BOT_TOKEN=xyz BACKEND_URL=http://localhost:8000 python3 scripts/telegram_bot.py")


async def send_telegram_message(session: aiohttp.ClientSession, chat_id: int, text: str):
    url = f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage"
    payload = {"chat_id": chat_id, "text": text, "parse_mode": "Markdown"}
    async with session.post(url, json=payload) as resp:
        return await resp.json()


async def submit_and_track_job(session: aiohttp.ClientSession, chat_id: int, query: str):
    headers = {"Content-Type": "application/json"}
    if API_KEY:
        headers["Authorization"] = f"Bearer {API_KEY}"

    # Submit job to Rust engine
    async with session.post(
        f"{BACKEND_URL}/v1/jobs/async",
        headers=headers,
        json={"query": query},
    ) as resp:
        if resp.status != 200:
            err = await resp.text()
            await send_telegram_message(session, chat_id, f"⚠️ Error submitting query: {err}")
            return

        data = await resp.json()
        job_id = data["job_id"]
        status = data.get("status", "SUBMITTED")

    if status == "QUEUED_ON_HOLD":
        msg = (
            "⏳ *Server under load*: Memory threshold reached. "
            "Your query is safely held in queue and will start immediately once resources free up."
        )
        await send_telegram_message(session, chat_id, msg)
    else:
        msg = f"🔍 *Deep Research Started*\nJob ID: `{job_id}`\nAnalyzing query..."
        await send_telegram_message(session, chat_id, msg)

    # Poll / wait for completion
    while True:
        await asyncio.sleep(2)
        async with session.get(f"{BACKEND_URL}/v1/jobs/async/{job_id}", headers=headers) as resp:
            if resp.status != 200:
                continue
            job_data = await resp.json()
            job_status = job_data.get("status")

            if job_status == "COMPLETED":
                result = job_data.get("result", "Research finished.")
                prompt_toks = job_data.get("prompt_tokens", 0)
                comp_toks = job_data.get("completion_tokens", 0)
                total_toks = job_data.get("total_tokens", 0)

                # Telegram max message length is 4096 chars
                summary_header = (
                    f"📊 *Research Completed*\nTokens: {total_toks} ({prompt_toks} prompt / {comp_toks} completion)\n\n"
                )
                full_text = summary_header + result
                if len(full_text) > 4000:
                    chunks = [full_text[i : i + 3900] for i in range(0, len(full_text), 3900)]
                    for chunk in chunks:
                        await send_telegram_message(session, chat_id, chunk)
                else:
                    await send_telegram_message(session, chat_id, full_text)
                break
            elif job_status == "FAILED":
                err = job_data.get("error", "Unknown error")
                await send_telegram_message(session, chat_id, f"❌ *Research Failed*: {err}")
                break


async def main():
    if not TELEGRAM_BOT_TOKEN:
        return

    offset = 0
    print("Telegram Bot Bridge running. Listening for updates...")
    async with aiohttp.ClientSession() as session:
        while True:
            try:
                url = f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/getUpdates?offset={offset}&timeout=30"
                async with session.get(url, timeout=35) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        for update in data.get("result", []):
                            offset = update["update_id"] + 1
                            msg = update.get("message", {})
                            chat_id = msg.get("chat", {}).get("id")
                            text = msg.get("text", "")

                            if chat_id and text:
                                if text.startswith("/start"):
                                    welcome = (
                                        "👋 *Welcome to Deep Researcher AI*\n"
                                        "Send me any complex question, market inquiry, or research query!"
                                    )
                                    await send_telegram_message(session, chat_id, welcome)
                                else:
                                    asyncio.create_task(submit_and_track_job(session, chat_id, text))
            except Exception as e:
                print(f"Error in poll loop: {e}")
                await asyncio.sleep(5)


if __name__ == "__main__":
    asyncio.run(main())
