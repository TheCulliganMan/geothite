# PC source YES/NO closing time

`home/menu.asm` runs VerticalMenu, retains its selected window for fifteen
VBlanks, then closes the window and returns the result. PC Toss, Oak's rating
question and Put Mail In Pack use this same timing. B returns NO without moving
the displayed cursor; simultaneous A+B resolves to B. Disabled horizontal,
Start and Select inputs must not change the result during the wait.

The existing PC joypad wait owns the closing phase. The historical
`mailbox_confirmation_response` field stores the chosen result; while it is
present, input cannot change the question or selection. On the fifteenth frame,
`finish_visible_pc_yes_no_answer` closes the window and invokes the existing
confirmation dispatcher. Item removal, Oak rating/fanfare and shutdown remain
behind that boundary. The next source menu retains its background-upload fence.

Drawing and input readiness are separate. The printed question and YES/NO box
must remain drawn while input is blocked. The renderer-neutral production
controller advances the same noninteractive wait before returning a new input
boundary; it never dispatches a second implementation of the service.

Use `pc:oak-closing` and `pc:toss-closing` only as read-only native capture
observers reached through ordinary input. Their receipts include the retained
answer and remaining VBlanks. A capture can freeze that presentation after it
is reached; it must not create a confirmation or change its result. The
three-color item LCD exception still requires the actual frozen fullscreen
owner and contrast checks; arbitrary images and scenery do not qualify.

This closes the PC timing gap. Other field YES/NO owners still require their
own source-timing review; do not infer whole-game timing acceptance from PC
coverage. Generic field palette and independent ROM image gates remain separate.
