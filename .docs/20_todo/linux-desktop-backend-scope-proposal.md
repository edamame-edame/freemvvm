# 0.0.1 Linux Wayland backend 後続候補の対象化提案

- 提案版: 0.1
- 状態: 利用者レビュー待ち。要求・設計本文は未変更、実装未着手
- 対象version: `0.0.1`
- 関連計画: [Linux DRM/KMS runtime resize 実装計画案](linux-drm-runtime-resize.md)、[Linux profile検証計画](release-0.0.1-linux-lts-profile-verification.md)
- 対象文書: [RQ-11 Platform](../11_requirements/RQ-11_platform.md)、[Platform BD](../12_design/platform/abstraction.md)、[Platform DD](../13_detail_design/platform/internal-layers.md)、[release test matrix](../30_release_test/requirement-test-matrix.md)、[release scenarios](../30_release_test/system-scenarios.md)

## 背景と問題

現行Linux backendはDRM/KMS + GBM/EGL/GLESで、選択したconnector/CRTCへ直接modesetする。GNOME/Wayland desktopの通常windowとして表示するadapterではない。利用者のDebian 13 VirtualBox確認では、desktop session上で背景色が変わらず、active TTYへ切り替えた後にだけvisible frameを確認できた。

このTTY確認はdirect-display/console経路のfeasibilityを示すが、GNOME等のdesktop sessionから通常windowとして起動できるLinux UI library profileの証拠にはならない。KMS mode resizeを加えても、compositorとのsurface共有・window管理は解決しない。sudoやdisplay manager停止をLinux desktop利用の前提にしない。

現行RQ-11 0.7はUbuntu 24.04と条件を満たすDebian 12 VMを0.0.1 runtime profile候補に含める一方、Wayland (`RQ-11-015`) とX11 (`RQ-11-016`) を版未指定の後続候補としている。利用者はWaylandが0.0.1内の後続候補に含まれるなら許容した。よって本案はWaylandを直ちに0.0.1正式runtime supportへ昇格するのではなく、0.0.1計画内の後続候補として明記する。

## 推奨対象範囲

1. 現行DRM/KMS backendは0.0.1のdirect-display/console profileとして限定記録する。active TTY上のfullscreen outputは実証済みだが、GNOME等のdesktop window sessionでは表示を保証しない。rootやdisplay manager停止を利用条件にしない。
2. Wayland desktop backendを0.0.1の後続候補へ明示追加する。候補である間は実装・desktop runtime supportを主張しない。採用する場合はWayland sessionで通常Windowを作成し、TTY切替なしで表示する独立profileを目標にする。
3. Waylandを実装する段階で、compositor configure/resize通知をCoreの`PlatformWindowClientAreaSink`へ接続し、Core Layout/Sceneを再生成して新surface extentへ再描画する案を設計レビューする。client-areaとDeviceScaleは承認済みDD 1.5/1.7契約を根拠にし、Wayland固有のDPI/scale意味は別途明示する。
4. X11 (`RQ-11-016`) は0.0.1候補に加えず、後続候補に残す。Wayland候補だけで他のdesktop sessionをサポート済みと表示しない。
5. KMS runtime resize (`linux-drm-runtime-resize.md`) はdirect-display profile用の別実装課題として維持する。Wayland window resize passとKMS mode resize passを相互に読み替えない。

## 文書改訂案

- RQ-11-003 / RQ-11-014: 現行Linux runtime profileはdirect-display/console条件付きであること、Waylandは0.0.1の後続候補であることを分けて明記する。
- RQ-11-015: Wayland候補の対象版を0.0.1と明記し、候補登録のみではformal runtime supportにならないことを記す。X11は後続候補のまま。
- RQ-11-022 / RQ-11-025: KMS console profileと将来Wayland desktop profileのbackend/default/failure境界が混ざらないか照合する。要求意味の追加が必要な場合だけ改訂案へ含める。
- Platform BD: Linux direct-displayとcompositor-managed desktopのsurface責務・profile境界を分ける。Wayland flowの確定はWayland DD案の承認後とする。
- Platform DD: Waylandを実装候補へ進める場合、`xdg_toplevel` lifecycle、configure/close/input ingress、EGL/Wayland surface、frame callback/presentation、DIP client-area/scale、failure/cleanup、worker/UI dispatcher flowを別のDD proposalとして提示する。公開C ABIは既存surface/backend選択契約内に保つ案を初期値とする。
- Release tests: 既存Linux VM TCのdirect-KMS TTY evidenceにはdesktop support statusを混在させない。Wayland candidateは別profile scenario案として用意し、実装後のvisible Window/resize/close/reopenを配布artifact・同一candidate SHAに結び付ける。

## 調査・レビュー工程

1. Ubuntu 24.04とDebian VMでWayland sessionの有無、Wayland/EGL runtime/dev package、available compositor protocolを読み取り確認する。候補対象OS/VM版はRQ-11承認時に確定する。
2. Wayland client protocol、`xdg-shell`、EGL/GBM surface候補、native/runtime dependency、Linux CI/container bootstrap条件を調査し、candidate feasibility memoへ記録する。
3. RQ-11-015を「0.0.1後続候補」とする要求文書改訂案を先に提示し、利用者承認を受ける。要求本文承認後にBD/DD/release-test文書案を順に提示し、各文書単位の承認を受ける。
4. 文書承認後にWayland implementation planを別途作成・提示する。implementation plan承認までは依存追加・source/test変更をしない。
5. Wayland実装が後続工程として選択・完了した場合、Wayland session上のvisible present/resize/close/reopenを候補OS別に検証する。候補のまま0.0.1を終える場合は、desktop support未対応として明示する。

## 受け入れ境界

- TTY/KMS smokeはconsole direct-display profileの証拠に限定し、GNOME等のdesktop window supportを満たしたと扱わない。
- Waylandは0.0.1内の後続候補として計画可能だが、candidate登録だけではruntime profile/formal supportにならない。実装・package・session上のvisible testに合格したprofileだけをsupport表へ載せる。
- X11、multi-output、HiDPI/DPI migration、異なるGPU間migration、全IME/Accessibility保証は本提案だけでは追加しない。必要なら別のRQ/設計範囲を提示する。
- 提案・文書承認・CI buildのみでWayland profileを正式support済みと表示しない。配布artifactを使った実sessionのvisible testを必須とする。

## 承認境界

- 本文は「KMS direct-displayをconsole限定として正確に扱い、Wayland desktop backendを0.0.1の後続候補へ明記する」要求・計画範囲のレビュー提案である。Waylandを0.0.1の正式runtime supportにする確定ではない。
- 利用者の対象範囲承認前にRQ/BD/DD正本を書き換えない。
- 文書全体承認前にWayland実装・dependency追加・test登録・commit/pushを開始しない。
- Linux DRM/KMS resize案は別profileの実装計画として維持し、Wayland作業で代替完了としない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-13 | 提案0.1 | 利用者レビュー待ち | 利用者の「後続候補として0.0.1に含まれてるのならいいですよ」を反映。Waylandを0.0.1内の後続候補として明記し、KMS TTY evidenceとdesktop supportを分離する案。RQ/BD/DD/試験文書は未変更 |
