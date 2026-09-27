# MVVM UIライブラリ Python wrapper方式に関する上位判断案

- 状態: BD-14承認済み（2026-09-26）
- 対応: C++／C#／Rust／Python 3のうち、Python wrapperがversioned C ABIを呼び出す方式
- 根拠: Accepted ADR-0001、要求仕様第4・9・12節、承認済みA1～A32、L1～L4。DD-00は未承認

## BD-14 承認された判断

**初期Python 3 wrapperは標準ライブラリの`ctypes`で実装し、Rust Coreのversioned C ABI native libraryを動的ロードする。Python専用のcompiled extensionは初期版では作らない。C ABIで扱う値・callbackは`ctypes`から安全に宣言できる固定幅型とopaque handleを基本にする。wrapperは承認済みL1～L4の共通意味論を保ち、raw `ctypes` objectをアプリの公開APIにしない。**

## 選択理由と比較

| 観点 | ctypes（採用） | cffi |
|---|---|---|
| 配布依存 | Python標準ライブラリで追加runtime dependency不要 | cffi packageが必要。API modeではcompiled extensionとPython ABI別wheel等の運用が加わる |
| C ABI接続 | versioned C symbolsを直接ロードして呼ぶ。初期APIがopaque handle・固定幅値・明示callbackなら適合しやすい | C宣言を扱いやすく、宣言量やstruct layoutが大きいAPIでは記述と型確認が楽になる |
| ビルド境界 | wrapper自体をPython extensionとしてtarget別compileしなくてよい。native Rust libraryは従来どおりtarget別 | ABI modeはdynamic load可能だがC宣言を保守する。API modeはC compilerとtarget別Python extensionを別途管理する |
| 主な注意 | prototypeの宣言漏れ・型誤りを実行時に検出しにくい。callback objectを必要期間保持し、例外をC境界の外へ漏らさない必要がある | dependency、API/ABI mode選択、宣言と実体の同期、Python ABI別artifactの管理が必要になる |

標準ライブラリのみでC ABIへ届き、Rust native libraryとPython wrapperを分離するADR-0001の配布方針に合うため、初期方式としてctypesを選ぶ。C ABIを複雑なstruct-by-valueやC++ objectに依存させない制約は、承認済みC ABI境界と併せて保つ。

## 初期wrapperで守る契約

- C ABIのsymbol version、error/status、thread制約、ownershipは既存契約をそのまま使う。Pythonから直接Rust objectやnative pointerを管理させない。
- Pythonの公開APIはL1～L4に沿ったPython object／methodとして包み、`ctypes`の関数・pointer・callbackを通常のアプリコードへ露出しない。
- callbackはwrapperがCore側の解除・完了まで保持し、Python exceptionをC ABI越しに投げない。callback failureは承認済み診断・error経路へ変換する。
- Python wrapperはnative libraryのtarget別配置を前提に読み込み、C++／C#／Rustと同じtargetのC ABI artifactを共有する。x64 libraryをARM向けに流用しない。
- wrapper parity testは同一のMVVMシナリオを4言語で実行する要求を維持する。

## この判断では決めないこと

- Python package名、PyPI publishing方式、wheelのOS/architecture/Python-version matrix、native libraryの同梱・system installの選択。
- CPython以外のPython interpreter、Pythonの最低version、ABI stability期間。
- `ctypes` function prototype生成・header parity testの具体手法、library discovery path、callback trampoline実装。
- C ABIの各関数・型・struct layout、Python固有の非同期API。
- C++／C#／Rust wrapperのclass namingやpackage manager。L1～L4の共通意味論を変更しない範囲で後続設計する。

## 承認記録

2026-09-26に利用者承認。初期Python 3 wrapperは標準ライブラリ`ctypes`を使ってversioned C ABIを動的ロードし、Python専用compiled extensionを必須としない。Python公開APIはL1～L4の共通意味論を包み、raw ctypes objectをアプリへ露出させない。callbackの保持と例外変換も既承認規則に従う。型チェック強化とPython向けlint/MCP連携は別の上位判断BD-15で提案し、BD-03のPython例外を黙って変更しない。package、wheel、interpreter matrix等は後続判断とする。DD-00の承認やDD-01以降へ進む判断ではない。
