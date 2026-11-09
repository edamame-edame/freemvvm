# カテゴリ: Platform / Debian 12 VirtualBox visible-present確認

## Scope

Debian 12 x64上で既存Linux DRM/KMS Platform Adapterの描画を確認した環境と、再現時に区別すべき証拠範囲を記録する。framebuffer限定backendそのものの設計・実装記録ではない。

## 環境と結果

- Host VirtualBox 7.1.10。別途作成したDebian GNU/Linux 12.15 x64 VMは4 vCPU/8 GiB、VMSVGA 128 MiB、3D acceleration on。
- Guest kernelは`6.1.0-54-amd64`。virtual GPUは`VMware SVGA II [15ad:0405]`、kernel driverは`vmwgfx`、DRM deviceは`/dev/dri/card0`と`/dev/dri/renderD128`。
- Source SHAは`eeb2dc322ef9667170028f010c812f13361fbf3b`。Linux Platform testはCore 203件、Linux Platform 21件、ignored headless GLES pixel tests 2件、Linux check/Clippy、Rust 1.85.0 checkがpass。
- Debian guestでbuildした`libfreemvvm_abi_1.so`のSHA-256は`abb2d5fecfc737e4f2f5f11edf08a5373879815cc61e5b2ec0febecbb52c953f`。
- Python wrapperから上記C ABI libraryを使い、Runtime UI loopで`runtime_dispatch_turn`をpumpしながらWindowをopenした。赤系背景を画面で確認し、closeした後に青系背景で再openして画面表示を確認し、最後にcloseした。
- smokeはGDMを一時停止しrootで実行した。確認後にGDMを起動し、VMをlogin screenへ戻した。既存Ubuntu VMは変更していない。
- guest kernel logに`[drm:vmw_msg_ioctl [vmwgfx]] *ERROR* Failed to open channel.`が複数回出た。色付きframeの可視表示とopen/closeのAPI結果は成功したが、このdiagnosticの発生源・影響は別途調査対象として残す。

## 再利用時に守る証拠境界

- `Window.open()`が成功しただけでは可視presentを示さない。Host UI loopが`runtime_dispatch_turn`を処理し、画面上のframeを観測する。
- 現行Linux backendはDRM/KMS+GBM/EGL/GLESを使う。このVM smokeはfbdev (`/dev/fbN`) またはDRM/KMS dumb-bufferによる新規framebuffer sinkの試験ではない。
- headless surfaceless EGLのpixel test、VM guest OS起動、DRM nodeの存在はそれぞれvisible presentと別の証拠である。
- 今回はroot実行・Python source wrapper・debug C ABI artifactであり、配布package consumer、非root device access、input/IME/accessibility、window resize、別hypervisor/profileの合格を意味しない。
- Debian 12 VirtualBox profileをrelease対象とする最終判断には、要求された配布consumerと残りのprofile-specific criteriaを候補SHAに結び付ける。

## Platform別defaultに関する利用者方針

- 2026-11-09: 完了済みのPlatform Adapterは置き換えず、各profileに適したbackendを未指定時のdefaultにする。別の登録済み候補も明示選択できる構成を望んでいる。
- この方針を既存の承認済み設計へ反映するため、defaultのprofile matrix、設定位置、surface/scene compatibility、診断をDD改訂案として提示する。実装開始前の文書単位レビュー・承認を経て確定する。

## References

- [0.0.1 platform support scope revision](../20_todo/release-0.0.1-platform-scope-revision-proposal.md)
- [Linux framebuffer限定backend 実装計画案](../20_todo/linux-framebuffer-limited-backend.md)
