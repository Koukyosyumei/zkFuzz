#!/bin/bash

# Intermediate Files
meta_file="meta.csv"
trace_file="trace.csv"
side_file="side.csv"

# Separate each section
awk '/template_name/{flag=1; print > "'$meta_file'"; next} /Total_Constraints/{flag++; if(flag==2) print > "'$trace_file'"; else print > "'$side_file'"; next} flag==1 {print > "'$meta_file'"} flag==2 {print > "'$trace_file'"} flag==3 {print > "'$side_file'"}'

# Remove the last line
head -n $((lines - 1)) "$side_file" > "$side_file.tmp"
mv "$side_file.tmp" "$side_file"

calculate_average() {
  file=$1
  header=$(head -n 1 "$file")
  echo "$header"              
  awk -F, '
  NR==1 { next }
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

calculate_median() {
  file=$1
  header=$(head -n 1 "$file")
  echo "$header"              
  awk -F, '
  NR==1 { next }
  {
    for (i=1; i<=NF; i++) {
      data[i][count[i]++] = $i;
      sum[i] += $i;
    }
  }
  END {
    for (i=1; i<=NF; i++) {
      if (count[i] > 0) {
        # average
        avg = sum[i] / count[i];

        # median
        asort(data[i]);
        if (count[i] % 2 == 1) {
          median = data[i][int(count[i] / 2) + 1];
        } else {
          median = (data[i][int(count[i] / 2)] + data[i][int(count[i] / 2) + 1]) / 2;
        }

        printf "%s,", median;
      }
    }
    print "";
  }' "$file"
}

# Aggregation
echo "Meta Information Average:"
calculate_average "$meta_file"
echo "Meta Information Median:"
calculate_median "$meta_file"
echo "Trace Constraints Average:"
calculate_average "$trace_file"
echo "Trace Constraints Median:"
calculate_median "$trace_file"
echo "Side Constraints Average:"
calculate_average "$side_file"
echo "Side Constraints Median:"
calculate_median "$side_file"

# Cleanup
rm -f $meta_file $trace_file $side_file