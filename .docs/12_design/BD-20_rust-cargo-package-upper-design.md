# MVVM UIライブラリ Rust Cargo packageに関する上位判断

- 状態: BD-20承認済み（2026-09-27）
- 対応: Rust利用者向けsafe API wrapper、raw C ABI declarations、target別native runtimeの配布境界
- 根拠: Accepted ADR-0001、要求仕様第4・9・12節、BD-16～BD-19、Cargo crate/build script/linkage仕様。DD-00は未承認

## BD-20 承認された判断

**Rust向けwrapperはCargo library crateとして配布し、raw `unsafe extern` C ABI宣言を担う低レベル`-sys` crateと、それを安全なRust APIで包む高レベルcrateを分ける。両crateはRust Coreを含めず、BD-17～BD-19と同一targetのC ABI native runtimeを別artifactとして利用する。別環境で生成・保管したnative artifactを利用側で取得・取り込める経路を提供し、`prebuilt-native`相当のCargo featureでその利用を有効化する。実際のbuild targetはCargoの`--target`で選択し、必要なboard/toolchain差分はnamed artifact profileで選べるようにする。featureはartifact取得モードの有効化に使い、target tripleの代替にはしない。registryはCargo互換registryを前提にできる形とするが、crates.ioかprivate registryかは決めない。**

## 配布境界

| 構成要素 | 推奨案 | 後続へ送る詳細 |
|---|---|---|
| Low-level crate | versioned C ABIの型・関数をraw FFIで公開する`-sys` crate | crate名、feature、bindgenか手書き宣言か、sys API visibility |
| Safe wrapper crate | Rust ownership／lifetimeを利用してABI handleを安全に包むlibrary crate | API naming、Send/Sync、callback、async facade |
| Native runtime | target別C ABI shared libraryを別native artifactとして提供 | install layout、link search、runtime loader path、static/dynamic選択 |
| External artifact import | 事前build済みartifactを別environmentから取得し、cache/stagingへ取り込んでRust crateから利用するfeature | artifact registry、fetch command/API、cache directory、credentials、offline policy |
| Target matrix | Windows x64／Linux x64から始め、BD-16のARM cross-buildとhardware gateを維持 | target triple、linker、libc baseline、board profile |
| Registry | Cargo package形式に合わせる。registry選択は後続 | crates.io/private registry、release credential、signing |

## 理由と影響

- Cargo library packageは`.crate` source packageとしてregistryへ配布できる。Cargo build scriptはnative libraryのlinkingに必要なsearch pathやlink instructionをbuildへ伝える手段を持つ（[Cargo: Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)、[Cargo: Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)）。
- Userはtoolchainやhardwareがある別環境でtarget native artifactをbuildし、その成果を利用側へ渡せる。Rust build scriptには`TARGET`と`HOST`が別々に渡されるため、cross-buildでhost側ではなく実際のtargetに一致するartifactを選ぶ（[Cargo: Environment Variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html)）。
- Cargoは`--target`でtarget tripleを指定し、`--features`でpackage featureを有効化する別の入口を持つ。したがって`prebuilt-native` featureをartifact取り込み機能の切替に使い、target triple自体は`--target`で指定する。target毎のfeature名を列挙する方式はcustom targetやboard profileに拡張しにくい（[Cargo: cargo build](https://doc.rust-lang.org/cargo/commands/cargo-build.html)、[Cargo: Features](https://doc.rust-lang.org/cargo/reference/features.html)）。
- Build中の自動ネットワーク取得は再現性・offline build・credential管理を複雑にするため、artifactの取得は明示的なfetch/acquire stepとし、通常のcompileは検証済みのlocal cache/stagingを消費する案を推奨する。具体toolとcache仕様は別判断とする。
- 取り込むartifactはtarget triple/profile、C ABI major、提供feature set、content digest等のmetadataを持ち、wrapper側の期待と不一致ならcompile/link前に失敗させる。署名方式とprovenance schemaは後続に残す。
- `-sys` crateとsafe wrapperを分ければ、unsafeなC ABI detailsを一箇所に閉じ込め、利用者が高レベルAPIからraw pointerを通常扱わない構造を作れる。Cargoはcrate linkageをstatic／dynamic双方で扱えるが、実際のnative runtimeはtargetとlink modeに適合させる必要がある（[Rust Reference: Linkage](https://doc.rust-lang.org/reference/linkage.html)）。
- CoreをRust crateへ重複して含めず、他言語wrapperと同じC ABI native artifactを使うことで、ADR-0001の単一Rust Coreと共通target binaryを保つ。
- native runtimeを別artifactにするため、Cargo crateを取得しただけではnative libraryの全配置が完結しない。SDK/artifact installationとlibrary discoveryの導入体験を縦断検証し、許容可能か確認する必要がある。
- 事前artifactを再利用すれば、ARM向けcross-compile toolchainを持たない利用者環境でもtarget別Rust wrapperをbuildできる。一方、target metadataとartifact provenanceの検証を怠るとhost／target取り違えやABI mismatchを招く。

## 本判断で決めないこと

- 実際のcrate名、registry、release policy、crate versionとABI majorの対応。
- native runtimeをsystem package、SDK archive、private registry artifact等のどれから取得させるか。
- 別environment artifactの具体registry、fetch command/API、cache layout、signature/provenance schema、retention policy。
- `build.rs`が要求する環境変数／metadata、pkg-config利用、runtime search path、static linking。
- Rust最低version、edition、`Send`／`Sync`の型契約、callbackの安全API。
- bindgen生成／手書き宣言、header distribution、license、support期間。

## レビューで確認する点

1. `-sys` raw FFI crateとsafe wrapper crateの二層をCargo package境界として使うこと。
2. Rust wrapper crateにRust Coreを重複して同梱せず、他言語と同一target C ABI native runtimeを使うこと。
3. native runtime artifactをCargo wrapper crateから分け、target別導入・linkingが必要であることを明示すること。
4. Windows x64／Linux x64を先行し、ARM LinuxはBD-16のbuild/runtime verification境界を維持すること。
5. prebuilt native artifactを別environmentから取得して取り込む利用機能を用意し、target/profile/ABI不一致を拒否すること。
6. artifact取得featureとCargo `--target`を区別し、個別target名をCargo feature一覧として固定しないこと。
7. crates.io/private registry、fetch command/API、library loading/link mode、crate names、Rust version matrixを本判断で確定しないこと。

## 参考資料

- [The Cargo Book: Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html) — library crateをregistry向けにpackage／publishする方法。
- [The Cargo Book: Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) — native libraryのbuild/link情報をCargoへ伝える方法。
- [The Cargo Book: Features](https://doc.rust-lang.org/cargo/reference/features.html) — package featureの役割と有効化。
- [The Cargo Book: Environment Variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html) — build scriptの`TARGET`／`HOST`等。
- [The Cargo Book: cargo build](https://doc.rust-lang.org/cargo/commands/cargo-build.html) — `--target`と`--features`のcompile option。
- [The Rust Reference: Linkage](https://doc.rust-lang.org/reference/linkage.html) — Rust crateのstatic/dynamic linkage。

## 承認記録

2026-09-27に利用者承認。Rust wrapperをraw FFIの`-sys` crateとsafe API crateに分け、Rust Coreはwrapper crateへ重複同梱せず、各言語で共有するtarget別C ABI native runtimeを別artifactとして利用する。別環境で生成したtarget artifactを取得・取り込む機能を用意し、`prebuilt-native`相当のfeatureで利用を切り替える。Cargoの`--target`でtarget tripleを選び、board/toolchain差分はnamed artifact profileで選ぶ。artifact取得は明示的なfetch/acquireとし、target/profile/ABI mismatchを検出してlink前に失敗させる。registry、fetch command、cache、signature/provenance schema、link/load方式、crate名、Rust versionは未決。DD-00の承認やDD-01以降へ進む判断ではない。
