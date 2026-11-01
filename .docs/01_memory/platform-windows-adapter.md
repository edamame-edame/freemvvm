- カテゴリ: freemvvm Windows Platform Adapter / Win32 Window peer

- Windows backendはfreemvvm-platform-windows crateに置き、C ABI target compositionからWindows targetだけにfactory注入する。CoreにWin32型を持ち込まず、共通Platform SPIを使う。
- Win32 HWNDとmessage pumpは専用worker threadが所有する。Coreの同期open/close操作はworker command/replyを使い、WM_SIZEはgeneration付きDIP client-areaとして既存sinkへ渡す。sinkはCore UI dispatcherへenqueueする。
- Runtime/Factory作成でWindowを生成しない。明示openの失敗後はretry可能にし、closeはCore Window/Treeを破棄せずreopenできるようにする。Rendererが未接続の段階ではframe submitを成功扱いしない。
- この実装単位ではOS WM_CLOSEをCore close-requestへ届けない。Core open状態とHWNDが分離しないようネイティブ破棄を抑止し、close-request sinkの別設計・実装までtitlebar close/input対応済みと扱わない。
- Windows SDK呼び出しにはworkspace lockに存在するwindows-sys 0.61.2をtarget-specific private dependencyとして使用する。Linux hostのworkspace buildではWindows adapter stubをcompileし、Windows CIでfake peer lifecycle unitとWindows target checkを行う。
