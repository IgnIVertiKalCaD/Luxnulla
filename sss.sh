#!/usr/bin/env bash
set -euo pipefail

# 1) Создаём папку для восстановления
RECOVERY_DIR="recovery_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RECOVERY_DIR"
echo "▶ Папка для восстановления: $RECOVERY_DIR"

# 2) Прогоняем fsck, чтобы собрать блобы
echo "▶ Запуск git fsck --lost-found"
git fsck --lost-found >/dev/null 2>&1

BLOB_DIR=".git/lost-found/other"
if [[ ! -d "$BLOB_DIR" ]]; then
  echo "❗ Каталога $BLOB_DIR не существует. fsck не нашёл блобы."
  exit 1
fi

shopt -s nullglob
BLOBS=("$BLOB_DIR"/*)
if [[ ${#BLOBS[@]} -eq 0 ]]; then
  echo "❗ В $BLOB_DIR нет ни одного файла — блобы не найдены."
  exit 1
fi

# 3) Обрабатываем каждый блоб
i=0
for BLOB_PATH in "${BLOBS[@]}"; do
  ((i++))
  BASENAME=$(basename "$BLOB_PATH")
  # Определяем MIME-тип
  MIME=$(file --brief --mime-type "$BLOB_PATH")
  case "$MIME" in
    text/*)           EXT="txt" ;;
    application/json) EXT="json" ;;
    application/xml|*/xml)  EXT="xml" ;;
    application/javascript|text/javascript) EXT="js" ;;
    image/png)        EXT="png" ;;
    image/jpeg)       EXT="jpg" ;;
    image/gif)        EXT="gif" ;;
    application/pdf)  EXT="pdf" ;;
    video/*)          EXT="mp4" ;;
    audio/*)          EXT="mp3" ;;
    *)                EXT="bin" ;;
  esac

  OUT_NAME="${i}_${BASENAME}.${EXT}"
  cp "$BLOB_PATH" "$RECOVERY_DIR/$OUT_NAME"
  echo "  → [$MIME] → $OUT_NAME"
done

echo "✔ Готово! Проверьте восстановленные файлы в $RECOVERY_DIR"
