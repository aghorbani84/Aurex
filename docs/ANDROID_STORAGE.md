# Android storage design — not implemented

No Android build, storage provider, permission request or installation is available
in this build. `Location::DocumentUri` is a distinct identity but returns Unsupported.
The Linux descriptor backend is not an Android SAF substitute.

Before Android support can be claimed:

1. Verify Dioxus 0.7.10 mobile launch/tooling against an installed SDK/NDK and emulator.
2. Bridge ACTION_OPEN_DOCUMENT_TREE and document-provider operations through JNI.
3. Represent opaque document IDs/URIs natively; query provider capability flags for
   create/write/rename/delete. Never assume rename/move/copy are atomic or supported.
4. Persist only explicitly user-granted URI permissions and handle revoked/unmounted
   grants without treating them as empty directories. Store connection metadata,
   never plaintext credentials. Test process restart and provider death.
5. Use MediaStore and authorized opening/sharing/install intents as appropriate;
   installation always hands off to the authorized package installer with consent.
6. Implement adaptive single-pane/touch/back behavior and test real file transfers
   through multiple providers, removable media and large documents.

No MANAGE_EXTERNAL_STORAGE or root permission is requested. Access to other apps'
private data and protected Android folders is not assumed. A privileged/root backend,
if ever introduced, must be separately enabled with explicit audited boundaries.
