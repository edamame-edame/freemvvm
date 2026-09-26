# 言語Binding設計 — ViewModel接続の第1段階

- 状態: L1～L4承認済み（2026-09-26）。具体的なAPIとABI表現は後続設計
- 作成日: 2026-09-26
- 対象: C++、C#、Rust、Python 3のwrapper
- 上位文書: `mvvm-ui-library-requirements.md`、`adr-0001-rust-core.md`、`runtime-contract.md`、`abi-spec.md`、`mvvm-semantics.md`

## 1. 範囲

Core管理のViewModel Propertyを4言語から扱う共通モデルと、言語側に既存のViewModelクラスがある場合の接続境界を提案する。Command、Collection、具体的なC ABI関数シグネチャ、コード生成、言語別パッケージ形式は後続設計へ送る。

`mvvm-semantics.md`で承認されたProperty、Binding、Validation、非同期Commandの意味論を各言語で一致させることを目的とする。言語の型やGCをC ABIへ露出させない。

## 2. 値の所在とwrapper

- 初期実装ではCore管理Propertyの値が正本となる。UI BindingはそのCore Property handleを参照する。各言語wrapperは同じhandleに対する型付きの読み書き、購読、Binding作成、解除を提供する。
- wrapperの型は各言語の使い方に合わせるが、setter後の値の確定、遅延通知、スレッド規則、検証失敗、TwoWayの一時的不一致は`mvvm-semantics.md`に従う。wrapperが独自の通知順や値の正本を持たない。
- 初期対象のViewModelは、言語側のオブジェクトがCore Propertyをメンバーとして保持する形で定義できる。ViewModelの名前付きメンバーとUI Propertyの接続はwrapperがBinding handleを所有して管理する。名前からの反射による自動発見は必須としない。

| 言語 | 初期の公開形（概念） | UI thread外からの更新 |
|---|---|---|
| C++ | RAIIの`Property<T>`と`Subscription` | 値をコピーしてdispatcherへ投稿 |
| C# | `Property<T>`と`IDisposable`購読 | UI dispatcherへ投稿し、delegateを保持 |
| Rust | 型付き`Property<T>`、解除可能な購読 | UI handleはUI threadに閉じ、送信可能なdispatcherへ値を渡す |
| Python 3 | `Property`と明示的な購読解除 | dispatcherへ投稿し、callableの寿命を保持 |

表の型名はwrapper設計上の例であり、正式なAPI名ではない。`Property<T>`の`T`はC ABIへそのまま渡さず、値型ごとの明示的な変換をwrapperが担う。float、null、object参照、コレクションの値表現はABI詳細設計で確定する。

## 3. 既存の言語側ViewModelとの接続

- 既存のC++ class、C# object、Rust struct、Python objectをCore handleへ直接変換しない。既存オブジェクトを接続する場合は、アプリが明示的なadapterを作成し、対象メンバーとCore Propertyの対応を指定する。
- 初期adapterはCore Propertyへ書き込む明示的な経路を持つ。言語側オブジェクトが別の場所で直接書き換わった場合、Coreがそれを自動的に検知する保証はない。通知を持つ既存オブジェクトの双方向同期は、通知解除、検証失敗時の差分、循環抑制まで含めて別段階で定義する。
- 既存オブジェクトを接続してもUIスレッド規則は変えない。worker上のオブジェクト更新からUIへ反映するには、値を取得・コピーして`runtime_post`へ送り、UI thread上でCore Propertyを更新する。workerからCore Propertyのgetter/setterを直接呼ばない。
- この接続方式では、既存クラスの任意のフィールド変更を「自動Binding可能」とは約束しない。4言語で同等の振る舞いを得るには、Core Propertyへ向かう更新経路を明示する。

## 4. Callback、所有権、エラー

- wrapperが購読や検証callbackをC ABIに登録した場合、wrapperは対象の言語オブジェクトをCoreのclosureが生きる間保持する。解除またはshutdownで`destroy`が呼ばれたときに一度だけ解放する。参照の解放に伴う最終化処理からUI APIを呼ばない。
- SubscriptionとBindingは明示的に解除できるようにする。wrapperのfinalizerだけに解除タイミングを依存させない。RuntimeのshutdownはアプリがUI threadで明示的に完了する。
- `ui_status`とerror handleは、各言語で成功値または明示的な失敗へ変換する。検証失敗、スレッド違反、停止済みRuntime、無効handleを区別し、callbackからの例外をABI越しに投げない。言語別の例外/Result型の正式な名前は後続設計で決める。
- `runtime_post`へ渡す値とclosureの保持・破棄はSlice 1の契約に従う。post失敗時はwrapper側が所有権を保持して解放する。post成功後はUIで実行されるか、shutdown時に未実行のまま破棄される。

## 5. 最小の利用例（意味論）

各言語のアプリは、Core Property `Name`と`Message`を持つViewModelを作る。`TextBox.Value`と`Name`をTwoWay、`Text.Content`と`Message`をOneWayで接続し、Binding handleを画面の寿命に合わせて解除する。workerが`Message`を変更する必要があれば、更新値をdispatcherへ投稿し、UI threadでsetterを呼ぶ。検証に失敗した入力は`mvvm-semantics.md`のV1～V8に従ってViewに残し、Bindingエラーを観測できる。

## 6. 検証する振る舞い

1. C++、C#、Rust、Python 3で同じ初期値、更新順、通知、Validation結果を観測できる。
2. 言語側ViewModelの明示adapterからCore Propertyへ値を送り、UI Bindingを更新できる。既存クラスの通知に依存しない経路を確認する。
3. workerからCore Propertyを直接読む・書くと`WRONG_THREAD`になり、dispatcher経由の値の反映はUI thread上で行われる。
4. 自己解除、画面終了、shutdown、post失敗のそれぞれで、言語側callback参照を一度だけ解放できる。

## 7. 承認された設計判断（2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| L1 | 初期実装のViewModel値はCore Propertyを正本とし、4言語wrapperが型付き操作を提供する | 4言語で共通のMVVM意味論 |
| L2 | 既存の言語側クラスは明示adapterで接続し、任意のフィールド変更の自動検知は初期契約に含めない | 既存アプリへの組込み方法 |
| L3 | workerからの更新は値をコピーしてdispatcherへ渡し、UI threadでCore Propertyへ書く | データ競合と非同期UI更新 |
| L4 | 購読・Bindingの解除とRuntime shutdownを明示操作とし、wrapperがcallback参照を保持・破棄する | GC、RAII、参照寿命 |

## 8. 次に設計する項目

- C ABIのProperty値型、入力・出力buffer、Validation結果とBinding errorの表現。
- 既存の言語側Observableクラスを双方向同期するadapterの通知・エラー・lifetime規則。
- 4言語の具体的なAPI、Commandの非同期タスク橋渡し、パッケージと各targetの検証。
