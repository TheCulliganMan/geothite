# Temporary native asset migration bootstrap

This directory is temporary review/execution tooling for PR #5. Adding it changes
no existing game source, runtime asset, authoring helper or CI workflow. The old
game remains buildable until the complete conversion is ready.

`integration.patch` is the reviewed UTF-8 source/helper/documentation update.
`run.py` applies it only in the guarded one-time Actions checkout, reconstructs
all native files from the already-committed chunks, and compares all identities.
The workflow then runs Python, native Rust and WebAssembly checks before calling
`finish`. The complete source and asset conversion is one commit, which removes
this bootstrap directory and the temporary workflow.

Do not apply the patch to a working branch without completing the migration in
the same commit. Do not publish a workflow with additional token permissions
without the user's explicit approval. There is no main-branch merge, history
rewrite, force push, LFS pointer, new credential or regenerated geometry.
