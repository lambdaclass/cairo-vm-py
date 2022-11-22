#!/usr/bin/env sh

tests_path="../cairo_programs"
exit_code=0
trace=false
memory=false
passed_tests=0
failed_tests=0

for i in $@; do
    case $i in
        "trace") trace=true
        ;;
        "memory") memory=true
        ;;
        *)
        ;;
    esac
done

for file in $(ls $tests_path | grep .cairo$ | sed -E 's/\.cairo$//'); do
    path_file="$tests_path/$file"

    echo "$file"
    if ! ([ "$file" = "blake2s_felt" ] || [ "$file" = "blake2s_finalize" ] || [ "$file" = "blake2s_integration_tests" ] || [ "$file" = "blake2s_hello_world_hash" ] || [ "$file" = "dict_squash" ] || [ "$file" = "squash_dict" ] || [ "$file" = "dict_write" ] || [ "$file" = "dict_write" ] || [ "$file" = "dict_update" ] || [ "$file" = "dict_read" ]); then
        if $trace; then
            if ! diff -q $path_file.trace $path_file.rs.trace; then
                echo "Traces for $file differ"
                exit_code=1
                failed_tests=$((failed_tests + 1))
            else
                passed_tests=$((passed_tests + 1))
            fi
        fi

        if $memory; then
            if ! ./memory_comparator.py $path_file.memory $path_file.rs.memory; then
                echo "Memory differs for $file"
                exit_code=1
                failed_tests=$((failed_tests + 1))
            else
                passed_tests=$((passed_tests + 1))
            fi
        fi
    fi
done

if test $failed_tests = 0; then
    echo "All $passed_tests tests passed; no discrepancies found"
else
    echo "Comparisons: $failed_tests failed, $passed_tests passed, $((failed_tests + passed_tests)) total"
fi

exit "${exit_code}"
