#!/usr/bin/bash

set -eo pipefail

for inventory in $(jq -c '.[]' inventory-files.json); do
    name=$(echo $inventory | jq '.name' )
    url=$(echo $inventory | jq '.url' )

    sphobjinv co plain -u "$url" -o "objs/$name.txt"

done
