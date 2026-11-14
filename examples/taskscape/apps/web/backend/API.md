# Taskscape web API

JSON over HTTPS, session authentication (cookie) with CSRF on writes. Every URL below is under `/api/`, has no trailing slash, and needs a signed-in user unless it says otherwise. A user only ever sees their own projects and everything inside them; another user's row answers **404**, never 403. Validation errors answer **400** with DRF's usual `{field: [messages]}` body.

Times are ISO 8601 UTC. Ids are integers. Lists are returned whole (no pagination).

## Auth and settings (built)

| Method and path | Body | Answer |
|---|---|---|
| GET `auth/csrf` (no sign-in) | | `{csrfToken}` and the CSRF cookie |
| POST `auth/signup` (no sign-in) | `{email, password (8+), name?}` | 201 `{id, email, name}`, signed in; creates project "Personal" with list "Inbox" |
| POST `auth/signin` (no sign-in) | `{email, password}` | 200 `{id, email, name}` or 400 |
| POST `auth/signout` | | 204 |
| GET `auth/me` (no sign-in) | | 200 `{id, email, name}` or 401 |
| GET, PATCH `settings` | `{theme: system\|light\|dark, reduced_motion: system\|on\|off, show_completed, last_project, last_list}` | the settings |
| GET, PUT `shortcuts` | PUT `{command, accel}`; accel `""` unbinds, `null` resets | the user's bindings `[{command, accel}]`; 409 when the chord is taken |

## Projects: `projects`

A project: `{id, name, sort_order, created_at, updated_at}`.

- GET `projects`: the user's projects by `sort_order`, then `created_at`.
- POST `projects` `{name}`: creates it at the end (`sort_order` = last + 1) with one list, "Inbox". 201. `name` is trimmed; empty after trimming is 400.
- PATCH `projects/{id}` `{name?, sort_order?}`: rename or reorder. 200.
- DELETE `projects/{id}`: deletes it with its lists, tasks, notes and attachments (files removed from storage). 204. Deleting the user's last project is 400 (`{"detail": "A project is needed."}`).

## Lists: `lists`

A list: `{id, project, name, sort_order, created_at, updated_at, task_count, done_count}` (counts are of live top-level and nested tasks, not trashed ones).

- GET `lists?project={id}`: the lists of one of the user's projects, by `sort_order`. Without `project`, every list of the user.
- POST `lists` `{project, name}`: at the end of that project. 201. Empty name is 400; another user's project is 404.
- PATCH `lists/{id}` `{name?, sort_order?, project?}`: rename, reorder, or move to another of the user's projects. 200.
- DELETE `lists/{id}`: deletes it and its tasks. 204.
- POST `lists/{id}/import` `{tasks: [{title, notes?, done?, children?: [...]}]}`: appends that tree. 201 with the created top-level tasks.
- GET `lists/{id}/export`: `{name, tasks: [{title, notes, done, children: [...]}]}` (live tasks only, in order).

## Tasks: `tasks`

A task: `{id, list, parent, title, notes, done, done_at, sort_order, created_at, updated_at, attachments: [attachment], note_items: [note]}`. Trashed tasks (`deleted_at` set) never appear here.

- GET `tasks?list={id}`: every live task of that list (top-level and nested), by `sort_order` then `created_at`. `?project={id}` for a whole project; neither for all the user's tasks. `?q=text` keeps tasks whose title or notes contain the text (case-insensitive), across the same scope.
- POST `tasks` `{list, title, notes?, parent?}`: at the end of its siblings. 201. `parent` must be a live task in the same list. Empty title is 400.
- PATCH `tasks/{id}` `{title?, notes?, done?}`: setting `done` stamps or clears `done_at`. **Completing a task completes its live descendants too**; un-completing it un-completes only itself. 200.
- POST `tasks/{id}/move` `{list?, parent?, sort_order?}`: re-home it (with its subtree) under another parent (`null` = top level) and/or into another of the user's lists; a task can't be moved under itself or its own descendant (400). 200.
- POST `tasks/{id}/reorder` `{sort_order}`. 200.
- POST `tasks/bulk` `{ids: [...], action: done|undone|trash|move, list?}`: one call for the bulk bar. `trash` moves every subtree to the Trash and answers `{trashed: [ids]}` for undo; `move` needs `list`. Any id the user doesn't own makes the whole call 404 and changes nothing.
- DELETE `tasks/{id}`: moves it and its live subtree to the Trash (sets `deleted_at` on each, the same instant). 200 `{trashed: [ids]}`.
- GET `tasks/{id}/copy`: `{text}`: the task and its subtree as a Markdown checklist (`- [ ] title`, `- [x] title`, two spaces of indent per level).

## Notes: `notes`

A note: `{id, task, content, sort_order, created_at, updated_at}`. `content` is HTML, sanitized on save to the tags `p br strong b em i u s ul ol li a code pre blockquote h1 h2 h3` (an `a` keeps only an `http`, `https` or `mailto` `href`); everything else is dropped, text kept.

- GET `notes?task={id}`.
- POST `notes` `{task, content}`: at the end. 201.
- PATCH `notes/{id}` `{content?, sort_order?}`. 200.
- DELETE `notes/{id}`. 204.

## Attachments: `attachments`

An attachment: `{id, task, kind: file|link, name, url, size, mime, created_at}`. For a file, `url` is where to download it (under `/media/`).

- GET `attachments?task={id}`.
- POST `attachments` multipart `{task, file, name?}`: uploads a file (up to 25 MB; larger is 400). `name` defaults to the file's name; `mime` is taken from the upload. 201.
- POST `attachments` JSON `{task, url, name?}`: a link (`http` or `https` only). `name` defaults to the URL. 201.
- PATCH `attachments/{id}` `{name}`: rename. 200.
- DELETE `attachments/{id}`: deletes it and its stored file. 204.

## Trash: `trash`

- GET `trash`: the user's trashed tasks, **most recently trashed first**, each `{id, title, list, list_name, deleted_at}`. Only the top-most trashed task of each trashed subtree is listed.
- POST `trash/restore` `{ids}`: restores those tasks and their subtrees trashed at the same moment. If a task's parent is still in the Trash or gone, it comes back at the top level of its list. 200 `{restored: [ids]}`.
- POST `trash/purge` `{ids}`: deletes them and their subtrees for good. 204.
- POST `trash/empty`: deletes everything in the user's Trash for good. 204.
- Restoring or purging an id that is no longer in the Trash (already restored or purged elsewhere) is **409** with `{detail, ids}` naming the stale ones, and changes nothing.

## Logging

Every write logs one line at INFO (`resource action id user`). Server errors are logged at ERROR, which posts them to the alert webhook when one is configured.
