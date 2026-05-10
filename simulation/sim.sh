#!/bin/bash

DATA_DIR="/home/lugolbis/Bureau/UVSQ/M1/S2/ranking/td_tests/Data"
CONF_FILE="conf.json"

MAX_RETRIES=20

for file in "$DATA_DIR"/*.mtx; do
    filename=$(basename "$file")

    echo "Traitement de $filename"

    TARGET_DIR=$DATA_DIR/$filename

    rm -R $TARGET_DIR 2> /dev/null
    mkdir $TARGET_DIR
    sed -i "s|\"matrix_path\": \".*\"|\"matrix_path\": \"$TARGET_DIR\"|" "$CONF_FILE"

    COUNT=0

    until ./target/release/ranking -c conf.json -s && ./target/release/simulation_plots "$TARGET_DIR/results.csv" 1e-12 42; do
        COUNT=$((COUNT + 1))

        if [ "$COUNT" -ge "$MAX_RETRIES" ]; then
            echo "Échec pour $filename après $MAX_RETRIES tentatives."
            break
        fi

        echo "Retry ($COUNT/$MAX_RETRIES)..."
        sleep 2
    done
done