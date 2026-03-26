# 辞書セットアップガイド

karukan-imの辞書を拡張するための辞書導入手順です。

## 概要

| 辞書                        | エントリ数 | 用途                                   |
| --------------------------- | ---------- | -------------------------------------- |
| システム辞書（SudachiDict） | 約266万    | 標準の漢字変換（プリインストール済み） |
| jawiki辞書                  | 約71万     | Wikipedia由来の固有名詞・専門用語      |
| 顔文字・絵文字辞書          | 約1.3万    | 顔文字・Unicode絵文字                  |

## 1. jawiki辞書のシステム辞書への統合

jawikiエントリをシステム辞書に統合することで、別辞書ロードのオーバーヘッドなく語彙を拡張できます。

### 1.1 Mozc UT辞書のダウンロード

```bash
# jawiki辞書（Mozc UT形式）をダウンロード
# https://github.com/utuhiro78/mozcdic-ut-jawiki/releases から最新版を取得
curl -L -o /tmp/mozcdic-ut-jawiki.tar.bz2 \
  "https://github.com/utuhiro78/mozcdic-ut-jawiki/releases/download/20250315/mozcdic-ut-jawiki-20250315.tar.bz2"
cd /tmp && tar xjf mozcdic-ut-jawiki.tar.bz2
```

### 1.2 jawikiエントリのフィルタリング

Mozc UT形式（5カラム）からkarukan形式（4カラム）に変換し、読みと表記が同一のエントリを除外します。

```bash
python3 << 'EOF' < /tmp/mozcdic-ut-jawiki-*/mozcdic-ut-jawiki.txt > /tmp/jawiki-filtered.tsv
import sys
seen = set()
count = 0
for line in sys.stdin:
    parts = line.strip().split('\t')
    if len(parts) < 5:
        continue
    reading, word = parts[0], parts[4]
    if reading == word or not reading or not word:
        continue
    key = (reading, word)
    if key in seen:
        continue
    seen.add(key)
    print(f"{reading}\t{word}\t名詞\t")
    count += 1
print(f"# Output: {count} entries", file=sys.stderr)
EOF
```

### 1.3 システム辞書との重複除去

```bash
# 現在のシステム辞書をダンプ
karukan-dict view ~/.local/share/karukan-im/dict.bin --all > /tmp/sysdict-dump.tsv

# 重複を除去
python3 << 'EOF' < /tmp/jawiki-filtered.tsv > /tmp/jawiki-deduped.tsv
import subprocess, sys

result = subprocess.run(
    ["karukan-dict", "view", "$HOME/.local/share/karukan-im/dict.bin", "--all"],
    capture_output=True, text=True
)

sysdict = set()
for line in result.stdout.splitlines():
    parts = line.split('\t')
    if len(parts) >= 2:
        sysdict.add((parts[0], parts[1]))

count = kept = 0
for line in sys.stdin:
    line = line.rstrip('\n')
    parts = line.split('\t')
    if len(parts) < 2:
        continue
    count += 1
    if (parts[0], parts[1]) not in sysdict:
        kept += 1
        print(line)
print(f"Input: {count}, Kept: {kept}, Removed: {count - kept}", file=sys.stderr)
EOF
```

### 1.4 統合辞書のビルド

```bash
# システム辞書ダンプ（reading\tword\tscore）とjawikiエントリ（reading\tword\tPOS\t）を
# JSON形式に統合してビルド
python3 << 'PYEOF' > /tmp/merged-dict.json
import json
from collections import OrderedDict

entries = OrderedDict()
sys_pairs = set()

# システム辞書の読み込み（スコア付き）
with open("/tmp/sysdict-dump.tsv") as f:
    for line in f:
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 3:
            continue
        reading, surface = parts[0], parts[1]
        try:
            score = float(parts[2])
        except ValueError:
            continue
        entries.setdefault(reading, []).append({"surface": surface, "score": score})
        sys_pairs.add((reading, surface))

# jawikiエントリの追加（スコア6000）
with open("/tmp/jawiki-deduped.tsv") as f:
    for line in f:
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 2 or not parts[0] or not parts[1]:
            continue
        if (parts[0], parts[1]) not in sys_pairs:
            entries.setdefault(parts[0], []).append({"surface": parts[1], "score": 6000.0})

result = [{"reading": r, "candidates": c} for r, c in entries.items()]
json.dump(result, open("/tmp/merged-dict.json", "w"), ensure_ascii=False)
PYEOF

# バックアップと再ビルド
cp ~/.local/share/karukan-im/dict.bin ~/.local/share/karukan-im/dict.bin.backup
karukan-dict build /tmp/merged-dict.json -o ~/.local/share/karukan-im/dict.bin
```

## 2. 顔文字・絵文字辞書の導入

軽量なTSV形式でユーザー辞書ディレクトリに配置します。

### 2.1 ソースのダウンロードと変換

```bash
# 各辞書のクローン
cd /tmp
git clone --depth 1 https://github.com/peaceiris/emoji-ime-dictionary.git
git clone --depth 1 https://github.com/6/kaomoji-json.git
git clone --depth 1 https://github.com/tiwanari/emoticon.git
curl -sL -o mozc-emoticon.tsv \
  https://raw.githubusercontent.com/google/mozc/master/src/data/emoticon/emoticon.tsv

# 統合スクリプト
python3 << 'EOF' > /tmp/kaomoji-emoji.tsv
import json, sys

seen = set()
count = 0

def emit(reading, word, pos="顔文字"):
    global seen, count
    reading, word = reading.strip(), word.strip()
    if not reading or not word or reading == word:
        return
    if (reading, word) in seen:
        return
    seen.add((reading, word))
    print(f"{reading}\t{word}\t{pos}\t")
    count += 1

# peaceiris/emoji-ime-dictionary
for fname in ["/tmp/emoji-ime-dictionary/tsv/emoji.tsv",
              "/tmp/emoji-ime-dictionary/tsv/emoji_additional.tsv"]:
    try:
        for line in open(fname):
            parts = line.rstrip("\n").split("\t")
            if len(parts) >= 3:
                emit(parts[0].lstrip(":"), parts[1], "記号")
    except FileNotFoundError:
        pass

# 6/kaomoji-json
try:
    for entry in json.load(open("/tmp/kaomoji-json/kao-utf8.json")):
        emit(entry.get("annotation", ""), entry.get("face", ""))
except FileNotFoundError:
    pass

# tiwanari/emoticon
try:
    for line in open("/tmp/emoticon/emoticon.txt"):
        parts = line.rstrip("\n").split("\t")
        if len(parts) >= 3:
            emit(parts[0].lstrip("@"), parts[1])
except FileNotFoundError:
    pass

# Mozc built-in emoticon
try:
    for line in open("/tmp/mozc-emoticon.tsv"):
        if line.startswith("keys"):
            continue
        parts = line.rstrip("\n").split("\t")
        if len(parts) >= 2:
            for reading in parts[1].split():
                emit(reading, parts[0])
except FileNotFoundError:
    pass

print(f"# Total: {count} entries", file=sys.stderr)
EOF
```

### 2.2 ユーザー辞書として配置

```bash
mkdir -p ~/.local/share/karukan-im/user_dicts
cp /tmp/kaomoji-emoji.tsv ~/.local/share/karukan-im/user_dicts/
```

### 2.3 確認

fcitx5を再起動して動作を確認します。

```bash
fcitx5 -r -d
```

テスト入力例:

- 「にこにこ」→ `＼(^o^)／`
- 「えがお」→ `😀`
- 「はーと」→ `❤`

## 辞書ソース

| 辞書                 | ライセンス   | URL                                               |
| -------------------- | ------------ | ------------------------------------------------- |
| mozcdic-ut-jawiki    | Apache-2.0   | https://github.com/utuhiro78/mozcdic-ut-jawiki    |
| emoji-ime-dictionary | MIT          | https://github.com/peaceiris/emoji-ime-dictionary |
| kaomoji-json         | -            | https://github.com/6/kaomoji-json                 |
| tiwanari/emoticon    | MIT          | https://github.com/tiwanari/emoticon              |
| Mozc emoticon.tsv    | BSD-3-Clause | https://github.com/google/mozc                    |
