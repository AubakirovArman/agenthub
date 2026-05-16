# V4.15 Chat Session Browser

Status: done

Implemented:

- `/chats` now lists chats with auto titles and pin state.
- `/search <text>` searches chat titles and message text.
- `/rename <title>` stores an explicit title for the current chat.
- `/pin` and `/unpin` keep important chats at the top or release them.
- `/chat <selector>` can open a chat by exact id or unique title/id fragment.
- Slash completion and `/help` include the new session commands.
- README, product CLI docs, wiki source docs, and changelog document the flow in four languages.

Verification:

- `cargo test shell::`
- `scripts/check-module-size.sh 200`
