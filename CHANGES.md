# Fork Changes (vmemjp/karukan)

This fork of [togatoga/karukan](https://github.com/togatoga/karukan) adds NixOS support and keybinding improvements.

Some features (Shift+Arrow partial conversion, F6-F8 direct conversion, half-width katakana conversion) were inspired by or adapted from [riq0h/karukan](https://github.com/riq0h/karukan).

## NixOS Support

### flake.nix

`flake.nix` を追加。以下のパッケージとdevShellを提供:

- `nix build .#karukan-cli` — karukan-server, karukan-dict, sudachi-dict, ajimee-bench
- `nix build .#karukan-fcitx5` — fcitx5 アドオン (karukan.so + libkarukan_im.so)
- `nix develop` — 開発環境 (Rust, cmake, fcitx5, libclang, rust-analyzer)

### fcitx5 アドオンの手動インストール

```bash
# ビルド
nix build .#karukan-fcitx5

# 環境変数付きで fcitx5 を起動
FCITX_ADDON_DIRS="$(readlink -f result)/lib/fcitx5:/run/current-system/sw/lib/fcitx5" \
XDG_DATA_DIRS="$(readlink -f result)/share:$XDG_DATA_DIRS" \
fcitx5 -d

# karukan に切り替え
fcitx5-remote -s karukan
```

## キーバインド追加

Composing (入力中) と Conversion (変換候補選択中) の両方で有効:

| キー | 動作 |
|---|---|
| F6 | ひらがな確定 |
| F7 / Ctrl+I | カタカナ変換確定 |
| F8 | 半角カタカナ変換確定 |
| Ctrl+H | バックスペース (Emacs-style) |
| Ctrl+U | 入力キャンセル (全消し) |
| Shift+Left/Right | 選択範囲の拡大/縮小 (部分変換用) |
| Shift+Home/End | カーソルから先頭/末尾まで選択 |
| Ctrl+Delete | 変換候補を学習データから削除 |
| 1-9 (候補表示中) | 候補を数字キーで直接選択・確定 |

## 辞書ビルド (mozcdic-ut)

mozcdic-ut の全辞書をマージした辞書をビルドできる。辞書バイナリはライセンス上リポジトリに含めない。

```bash
# 全 mozcdic-ut パッケージを取得・展開・マージ・ビルド
for pkg in mozcdic-ut-jawiki mozcdic-ut-neologd mozcdic-ut-skk-jisyo \
           mozcdic-ut-edict2 mozcdic-ut-personal-names mozcdic-ut-place-names \
           mozcdic-ut-alt-cannadic mozcdic-ut-sudachidict; do
  nix build nixpkgs#$pkg -o /tmp/$pkg
  f=$(ls /tmp/$pkg/*.tar.bz2)
  name=$(basename "$f" .tar.bz2)
  tar xjf "$f" -C /tmp/
  awk -F'\t' '{print $1"\t"$5}' "/tmp/$name"
done > /tmp/karukan-dict-all.tsv

# 辞書バイナリをビルド
nix run .#karukan-cli -- dict build /tmp/karukan-dict-all.tsv \
  --format mozc -o ~/.local/share/karukan-im/dict.bin
```

211万 reading、210MB の辞書が生成される。

### 辞書ソースとライセンス

| ソース | ライセンス |
|---|---|
| jawiki (Wikipedia) | CC BY-SA 3.0 |
| neologd | Apache 2.0 |
| skk-jisyo | GPL |
| sudachidict | Apache 2.0 |
| edict2 | CC BY-SA 4.0 |
| personal-names | - |
| place-names | - |
| alt-cannadic | - |

## karukan-engine の追加関数

- `hiragana_to_halfwidth_katakana()` — ひらがな → 半角カタカナ変換
- `katakana_to_halfwidth()` (内部関数) — 全角カタカナ → 半角カタカナ変換
