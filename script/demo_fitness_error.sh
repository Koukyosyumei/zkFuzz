#!/bin/bash

touch ./result_of_demo_fitness_error.out

for i in {1..10}; do
    ./target/release/proofuzz ../dataset/audit/StringCheck@zk-circom-project.circom --search_mode="ga" --path_to_mutation_setting ./parameters/demo_0.json >> ./result_of_demo_fitness_error.out
done