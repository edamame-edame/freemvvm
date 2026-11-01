- カテゴリ: freemvvm Windows Platform Adapter / Win32 peer and Direct2D renderer

- Windows backendはfreemvvm-platform-windows crateに置き、C ABI target compositionからWindows targetだけにfactory注入する。CoreにWin32型を持ち込まず、共通Platform SPIを使う。
- Win32 HWNDとmessage pumpは専用worker threadが所有する。Coreの同期open/close操作はworker command/replyを使い、WM_SIZEはgeneration付きDIP client-areaとして既存sinkへ渡す。sinkはCore UI dispatcherへenqueueする。
- Runtime/Factory作成でWindowを生成しない。明示openの失敗後はretry可能にし、closeはCore Window/Treeを破棄せずreopenできるようにする。
- Direct2D rendererはWin32 window workerがHWNDとともに所有し、Core SceneSnapshotをpixel座標へscale変換してbackground clear、Clip、FillRectの順で処理する。GlyphRunを含むsceneはDirectWrite unitまで失敗扱いにし、部分描画をPresentedとして公開しない。
- HWND render targetは96 DPIで作成し、Core DeviceScaleをrect edgeへ適用してpixel座標に変換する。WM_SIZEでtarget pixel sizeを変更し、WM_PAINTでは直近成功sceneを再描画する。FrameTicket completionはDirect2D EndDraw成功後にだけCore completion sinkへ渡す。
- この実装単位ではOS WM_CLOSEをCore close-requestへ届けない。Core open状態とHWNDが分離しないようネイティブ破棄を抑止し、close-request sinkの別設計・実装までtitlebar close/input対応済みと扱わない。
- Windows SDK呼び出しにはwindows-sys 0.61.2と、Direct2D COM interface用の`windows` 0.62.2をtarget-specific private dependencyとして使用する。Linux hostのworkspace buildではWindows adapter stubをcompileし、Windows CIでfake renderer/peer unitとWindows target checkを行う。
- 非Windows cfg stubにもWindows factory compositionから呼ぶ`Backend::new()`を定義する。Windows host testとWindows target checkだけではLinux host stubのcompileが見えず、Linux CIで初めて不足が判明しうるため、Linux target cross-checkかLinux runnerも対象にする。

- カテゴリ: freemvvm text rendering / GlyphRun color and Windows font resolution

- Core `GlyphRunSnapshot`の初期Text colorは不透明な黒 `[0,0,0,1]`。UNIT-CORE-080でscene値に保持するが、この範囲では任意色のsetter、C ABI、language wrapperを追加しない。
- Windowsの未指定fontはDirectWrite system font collectionとOSの言語/script fallbackでshaping時に解決する。`MS UI Gothic`名や`%WINDIR%\\Fonts`の固定走査を既定仕様にしない。OSが選んだ具体font faceのdigestをGlyphRunに結び、DirectWrite rendererは同じfaceを使う。
- 明示`.ttf`はDirectWrite font-file reference/custom font collectionから読み込み、OS font登録を行わない。file欠損・不正はshape failure、rendererがsnapshotのface digestを解決できない場合はframe failure。fallbackはshape時だけで、rendererは暗黙置換しない。
- Core GlyphRun color unitはWindows DirectWrite描画unitより先。同じRust/Core feature branch上の個別unitとして検証し、その後でWindows adapterを更新する。
