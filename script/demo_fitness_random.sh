#!/bin/bash

touch ./result_of_demo_fitness_random.out

for i in {1..10}; do
    ./target/release/tcct ../dataset/audit/StringCheck@zk-circom-project.circom --search_mode="ga" --path_to_mutation_setting ./parameters/demo_1.json >> ./result_of_demo_fitness_random.out
done