# DD-21 Python 3 wrapper 作業計画

- 状態: 完了（DD-PY-01～21承認済み、2026-10-12）
- 着手日: 2026-10-12
- 対象: Python 3 `ctypes` wrapperのhandle ownership、callback callable保持、explicit close/context manager、finalizer/cleanup、thread/GIL、例外/error変換
- 上位出典: Accepted ADR-0001、Accepted要件、承認済みL1～L4、A1～A32、BD-01/14/15/17/27/28/36
- 先行詳細設計: 承認済みDD-01～20
- 対応する詳細設計: `.docs/13_detail_design/21-wrapper-python.md`

## 目的

BD-14で承認された標準`ctypes`によるPython 3 wrapperを対象に、PythonのGC・callback・例外・thread環境から共通C ABIを安全に利用する境界をレビューする。raw ctypes function/pointerを通常の公開APIへ露出せず、L1～L4とDD-01～20のhandle・Runtime・dispatcher・callback・error契約を維持する。BD-17のpure-Python wrapper/native runtime別distribution境界も維持する。

## 作業手順

1. ADR、要件、L1～L4、A1～A32、BD-14/15/17/27/28/36、DD-01～20の本文状態・承認IDを照合する。Python固有の未決事項を承認済み共通契約から分ける。
2. `ctypes`宣言のfixed-width型、`argtypes`/`restype`、calling convention、opaque handle、library/function pointerの隠蔽境界を提案する。C ABIの各symbolはheader/ABI文書をsource of truthとし、prototype parity検査を検証観点へ入れる。
3. owned tokenを持つPython object、借用引数、明示`close`、context manager、retainによる独立ownerを整理する。ownerが回収されるときの`__del__`または同等finalizer cleanup handoffも含める。
4. token発行前のcleanup ticket予約、明示close失敗時のtoken保持、owner finalizerからの非block handoff、release失敗移管、UI safe point retryとClosed shell drainを検討する。finalizer threadからUI API/native UI cleanupを直接呼ばず、Runtime全体のshutdownと個別owner解放を区別する。
5. `CFUNCTYPE` trampolineとuserdata registry/rootingを提案する。登録失敗ではPython側所有、登録成功でCoreへ移転、destroyは一度だけ。callable、CFUNCTYPE instance、stateの強参照をGCから守り、任意のPython finalizerがlock中やC destroy stack上で動かない解放順を検討する。
6. `ctypes.CDLL`/callbackとGILの関係、foreign thread callback、UI thread identityを照合する。worker変更を許すwrapper stateは専用lockで保護し、worker変更を許さないUI-affine stateはread-only/snapshot/value copyで渡す。GILはlockの代用にせず、UI thread制約はwrapper確認とCore statusの両方で守る。free-threaded build対応の約束とは分けて記述する。
7. Python callbackの`Exception`/`BaseException`をC ABI trampolineの外へ伝播させない変換を提案する。`KeyboardInterrupt`/`SystemExit`等の扱い、利用者設定exception handler、handler自身の失敗、callback種別固有failureは共通契約を変更せず未決を分離する。
8. Special method overrideをoperator overloadingと同一視せず、method別にbase chain-up必須/任意/不要を表すmetadataと、wrapper-controlled runtime guard・BD-15 static analyzerの使い分けを提案する。`super()` text searchではなくrequired base implementationへの実到達を検査する。`__del__`によるcleanup handoffは許容し、base-call enforcementとは分ける。
9. `ui_status`・immutable Error snapshot・unknown status/codeをPythonの明示失敗表現へ写す。message parseで分岐しない。UI-only call、dispatcher更新、非同期/同期shutdownの戻り区分を示す。
10. BD-17に従いwrapperはpure-Python distribution、native runtimeは別target distribution/artifactとし、PyPI/name/wheel tags/native loader等をDD-22へ送る。BD-15のtype stub/解析器連携を考慮するがtool選択・MCP schemaは確定しない。
11. Python公式資料とQtのhost-loop終了通知例を参照し、owner GCとRuntime shutdownを別々に検討する。host UI loop終了直前通知、Python main/host thread終了、main thread未捕捉例外、wrapper所有worker未捕捉例外、callback例外、`SystemExit`、通常interpreter shutdownを区別し、`os._exit()`等の保証限界、callback gate/root/in-flight drain順序、wait deadlock条件を明示する。
12. `.docs/13_detail_design/21-wrapper-python.md`へ`DD-PY-*`判断ID付きレビュー案を作成し、利用者確認を受ける。2026-10-12にDD-PY-01～21の承認を受けた後、同文書・本計画・フェーズ一覧・状況メモへ結果を反映し、DD-21を完了にする。

## 対象外と停止条件

- BD-14の`ctypes`選択やL1～L4、DD-01～20の承認済み意味論の変更
- Python package名、PyPI/private index、最低Python version、CPython/PyPy/free-threaded support、wheel tag/RID matrix、native library loader
- C ABI symbol/struct layoutの変更、raw ctypes APIの利用者公開、compiled Python extension/cffi方式への変更
- `__del__`/`weakref.finalize`/managerの具体的最終採用、ticket registry/reaper同期、exit callback/root drainの実装、公開型・API名・`.pyi`生成方式
- callback exception handlerの全言語共通API、診断payload、callback種別固有の失敗意味論
- 動的Pythonコードの完全な静的解析保証、BD-15で未決のchecker製品/MCP schema/cache詳細
- UI loop停止時の同期worker shutdown解放、Runtime完了観測APIの追加
- 組み込みLinuxのboard/libc compatibilityと標準wheel tag適合（BD-16/17/DD-22）

## レビューで確認する点

1. 標準`ctypes`でversioned C ABIを呼び、raw ctypes objectをapplication APIへ露出しない。
2. `argtypes`/`restype`と固定幅型を明示し、platform依存の`long`/pointer widthをABI scalarの代用にしない。
3. owned tokenのclose/context scopeとowner finalization時のcleanup handoffを追跡でき、finalizer失敗・遅延時にもtokenを捨てない。明示closeとfinalizerの責任を分ける。
4. callback callable・trampoline・userdataはCore destroyまでrootされ、登録失敗/成功の所有者が一意である。
5. GILとUI threadを同一視せず、callback再入・foreign-thread invocation・thread-localの限界を扱う。
6. callback例外をC ABIへ漏らさず、機能別failure契約とuser handler仕様を勝手に共通化しない。
7. GILを共有状態の暗黙lockとして扱わず、変更可能stateは明示lock、変更禁止stateはread-only/value snapshot/UI thread dispatchとする。free-threaded build対応を設計契約と同一視せず、support対象に選ぶ場合にそのbuildで検証する。
8. Python special methodのbase chain-up要件はmethod別に識別し、必要なoverrideに限ってmissing base-callを検出する。`__del__`をcleanup handoffに使う可能性は保ち、chain-up enforcementと混同しない。
9. host UI loopの終了直前通知、Python main threadの未捕捉例外、`SystemExit`、wrapper所有workerの未捕捉例外、callback例外、通常interpreter shutdownが別々に扱われる。main未捕捉例外時も既存`sys.excepthook`のtraceback報告を維持し、BD-36/DD-03のthread/context dispatchでdeadlockを避ける。`os._exit()`等の異常終了は保証範囲から区別する。
10. BD-17のpure-Python wrapper/native runtime別distribution、BD-15のtype-check前提、DD-22の配布判断を混ぜない。

## 承認記録

2026-10-12: 利用者はDD-PY-01～21を承認した。通常終了・未捕捉例外の入口にQt `aboutToQuit()`相当のhost loop hook、`sys.excepthook`、wrapper所有worker境界、`atexit`補助通知を追加し、`SystemExit`と捕捉済み例外の扱い、`os._exit()`等の保証外を記録。承認状態を詳細設計・本計画・フェーズ一覧へ反映し、未決のhook/adapter実装、callback drain、API名、package/loader等は後続設計に残してDD-21を完了する。
