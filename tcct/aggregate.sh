#!/bin/bash

# 中間ファイルの準備
meta_file="meta.csv"
trace_file="trace.csv"
side_file="side.csv"

# 各セクションを分離
awk '/template_name/{flag=1; print > "'$meta_file'"; next} /Total_Constraints/{flag++; if(flag==2) print > "'$trace_file'"; else print > "'$side_file'"; next} flag==1 {print > "'$meta_file'"} flag==2 {print > "'$trace_file'"} flag==3 {print > "'$side_file'"}'

# 最終行を除いてファイルを一時ファイルに書き出す
head -n $((lines - 1)) "$side_file" > "$side_file.tmp"
# 一時ファイルを元のファイルに移動
mv "$side_file.tmp" "$side_file"

# 平均値計算関数
calculate_average() {
  file=$1
  header=$(head -n 1 "$file") # ヘッダー取得
  echo "$header"              # ヘッダー出力
  awk -F, '
  NR==1 { next } # ヘッダーをスキップ
  {
    for (i=1; i<=NF; i++) {
      sum[i] += $i;
      count[i]++;
    }
  }
  END {
    for (i=1; i<=NF; i++) {
      if (count[i] > 0) {
        printf "%s,", (sum[i] / count[i]);
      }
    }
    print "";
  }' "$file"
}

# 集計と結果出力
echo "Meta Information Average:"
calculate_average "$meta_file"
echo "Trace Constraints Average:"
calculate_average "$trace_file"
echo "Side Constraints Average:"
calculate_average "$side_file"

# 中間ファイルのクリーンアップ
rm -f $meta_file $trace_file $side_file