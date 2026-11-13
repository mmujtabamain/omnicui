# How the Taskscape ports were built: the builders' transcripts

Each builder ran as an agent of its own, from its prompt in `../prompts/` and the working
instructions of the environment it ran in. Its whole transcript is kept here, gzipped JSON
Lines (one message, tool call or tool result a line), with a summary beside it.

| builder | prompt | transcript | summary |
|---|---|---|---|
| P0: the contracts, the spec, the design, the `tauri` target | [P0.md](../prompts/P0.md) | none: P0 was run by the coordinating agent in its own session, not as a separate agent | [P0.md](P0.md) |
| P1: the GPUI port and its target | [P1.md](../prompts/P1.md) | [P1-gpui.jsonl.gz](P1-gpui.jsonl.gz) | [P1-gpui.md](P1-gpui.md) |
| P2: the web port, its server and its target | [P2.md](../prompts/P2.md) | [P2-web.jsonl.gz](P2-web.jsonl.gz) | [P2-web.md](P2-web.md) |

Before they were committed, the transcripts were scanned for secrets (keys, tokens, passwords,
private keys; none were found), and these were taken out:
- the environment's own instructions, and its working instructions given with each prompt (how
  to report progress, share screenshots and use the machine's toolchains): each is replaced by
  one line saying so;
- the names of the host and its services, its links and the branch, which became neutral words;
- the user's email address and name, and the home folder's path (`~`).

Read one with `zcat P1-gpui.jsonl.gz | jq -r 'select(.type=="assistant") | .message.content[]? | select(.type=="text") | .text'`.
