#!/bin/bash

module_name=$1
module_types=("modules" "subnets")

clean_up_module(){
    module_type=$1
    module_name=$2

    if [ -d "$module_type/$module_name" ]; then
        echo "Cleaning up module $module_name in $module_type"
        rm -rf "$module_type/$module_name"
        rm -rf ".$module_name"
        echo "Module $module_name cleaned up"    
    fi
}

if [ "$module_name" == "all" ]; then
    for module_type in "${module_types[@]}"; do
        echo "Cleaning up all modules in ${module_type}"
        if [ -d "$module_type" ]; then
            for module in "$module_type"/*; do
                if [ -d "$module" ]; then
                    echo "Cleaning up module ${module##*/} in ${module_type}"
                    clean_up_module "${module_type}" "${module##*/}"
                fi
            done
        fi
    done    
    echo "All modules cleaned up"
    exit 0
fi

if [ -z "$module_name" ]; then
    echo "Usage: $0 <module_name>"
    exit 1
fi

for module_type in "${module_types[@]}"; do
    if [ -d "$module_type/$module_name" ]; then
        echo "Cleaning up module $module_name"
        clean_up_module "$module_type" "$module_name"
        exit 0
    fi
done
