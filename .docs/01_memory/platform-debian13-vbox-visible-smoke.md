# カテゴリ: Platform / Debian 13 VirtualBox visible-present確認

## Scope

Debian GNU/Linux 13 x64 VirtualBox profileで、既存Linux DRM/KMS + GBM/EGL/GLES adapterのvisible presentとWindow peer再openを確認した証跡を記録する。Debian 12やUbuntu、物理GPU、別hypervisorへの結果外挿はしない。

## 環境とartifact

- Guest OS: Debian GNU/Linux 13.7 (trixie), x86_64。
- Guest kernel: `6.12.116+deb13-amd64`。
- 仮想GPU: VMware SVGA II `[15ad:0405]`、kernel driver `vmwgfx`。`/dev/dri/card0`と`/dev/dri/renderD128`を確認済み。
- VM settings (VirtualBox `showvminfo --machinereadable`): 4 vCPU、4096 MiB RAM、VMSVGA、128 MiB VRAM、3D acceleration on。
- Host VirtualBox: `7.1.10r169112`。
- Guest Additions installerは7.1.10。`vboxvideo` module buildはDebian 13 kernel API不一致で失敗したが、今回の描画は`vmwgfx`/DRM経路を使用した。
- Source HEAD: `480d8f2f80ab0b2098eaa90b42f9d9ebf2eda045`。
- Rust/Cargo: `rustc 1.90.0`、`cargo 1.90.0`。
- Artifact: debug `libfreemvvm_abi_1.so`、SHA-256 `182369c5826045a0334d16dbed9f60576828123d3a9ec019acf1ba539685dab4`。
- Python source wrapperから上記artifactを直接loadした。wheel/package consumerではない。

## 実行結果

- `cargo build --locked -p freemvvm-c-abi` がDebian guest上で成功。
- Python `window_peer_smoke.py` をTTY上のroot shellから実行。Runtime UI loopでdispatchを繰り返し、red → blueの背景更新、close/reopen後のgreen presentを試した。
- 利用者はTTYの画面で背景色の変化を確認。smokeは`Python Window visible present and reopen smoke: OK`で終了。
- Visible present、色変更によるredraw、close/reopen後の再presentはpass。resizeは未試験。現行Linux backendはopen時にconnector modeとsurface sizeを一度決定し、runtime display mode/client-area changeを処理する経路がないため、resizeを成功として扱えない。
- 実行中にkernel log `[drm:vmw_msg_ioctl [vmwgfx]] *ERROR* Failed to open channel.` が複数回出た。upstream Linux `vmwgfx_msg.c`では、このログはDRM message ioctlからhost向けRPC channelを開けない場合に出る。今回の色付きframeはそのログがあっても表示されたため、VM環境diagnosticとして保持し、visible present failureとは扱わない。
- Proof boundary: root/TTY/debug artifactでのDRM/KMS表示のみ。非root device permission、desktop compositor配下での表示、resize、配布package、input/IME/accessibility、他profileは未確認。

## 再現コマンド

```bash
cd /home/freemvvm/freemvvm-debian13-validation
export PYTHONPATH=/home/freemvvm/freemvvm-debian13-validation/wrappers/python/src
python3 wrappers/python/examples/window_peer_smoke.py /home/freemvvm/.cache/freemvvm-target/debug/libfreemvvm_abi_1.so
```

## References

- [0.0.1 Linux LTS profile verification plan](../20_todo/release-0.0.1-linux-lts-profile-verification.md)
- [Upstream Linux vmwgfx message ioctl](https://github.com/torvalds/linux/blob/master/drivers/gpu/drm/vmwgfx/vmwgfx_msg.c)
