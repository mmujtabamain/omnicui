# The web version: what changes from the reference app

The reference is a desktop app. The web version keeps its look and behaviour, screen for screen,
except where this file (or `spec/decisions.md`) says otherwise.

## Gone on the web

- The capture bar and the Capture settings page (no global hotkey, no mini window, no screenshots).
- Window chrome: no traffic lights or window buttons, no drag region (decision 2). The title bar is
  the app's top bar.
- Anything that needs the desktop: the data folder and its path buttons, launch at login, the tray,
  global shortcuts (decision 15), checking for updates, opening files in the OS, Finder/Explorer.
- Screenshot attachments: the slot becomes Image, which takes a pasted or picked image (decision 5).

## Different on the web

- Accounts: signing in is new. Signed out, the app shows the sign-in page (email, password; a link
  to create an account with name, email, password). The title bar ends with an account button
  (the initial of the name) whose menu shows the email, Settings and Sign out.
- Settings opens as a modal dialog over the main window, 640×520 (decision 19's size), with the same
  sidebar and pages (General, Shortcuts, About). Escape or the close button closes it. Settings are
  stored per account (decision 17).
- Sheets (new list, new project, rename, delete confirmations) are modal dialogs in the window, with
  the backdrop the reference draws.
- The lightbox is a full-window overlay inside the page.
- Trash opens in the aside, where the reference shows it (decision 1: a title-bar button and the
  palette open it).
- Shortcuts: web chords from decision 18; Mod is Ctrl on Windows and Linux, Cmd on macOS. Reserved
  browser chords are refused when rebinding.
- Data comes from the API (`apps/web/backend/API.md`); a failed request shows the error inline or
  in the toast, never silently.

## Demo data (what a screen test starts from)

Every screen test signs in as the demo account and loads one data set (`dataset` prop):

- `standard` (default): projects Home (lists Errands, Reading, Garden) and Spring launch (Website,
  Press kit). Errands holds "Buy groceries for the week" (subtasks Oat milk (done), Sourdough loaf,
  Apples), "Pick up prescription" (notes "Counter closes at 6"), "Post the parcel to Mum" (done),
  "Book bike service", "Cut a spare key". Website holds "Rewrite the hero section" (subtasks "Draft
  headline options", "Pick the hero image" (done); two notes; a link "Brand brief" and an image
  "hero-sketch.png"), "Pricing page: compare plans table", "Collect FAQ questions from support".
  Reading: "Finish the novel on the nightstand", "Read the long article on soil health" (done),
  "Return library books". Garden: "Order tomato seeds", "Trim the hedge". Press kit: "Export logos
  in SVG and PNG", "Team bios, 50 words each" (done). Trash: "Print the old flyer" (Errands, 2 days
  ago), "Call the printer about samples" (Website, 30 minutes ago). The app opens on Home › Errands.
- `empty`: project Home with one empty list, Inbox.
- `nolists`: project Home with no lists.
- `long`: standard plus 40 more tasks in Errands ("Errand number 1…40", every fifth done).
