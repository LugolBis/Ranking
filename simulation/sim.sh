#!/bin/bash

DATA_DIR="Data"
CONF_FILE="conf.json"

MAX_RETRIES=20

for file in "$DATA_DIR"/*.mtx; do
    filename=$(basename "$file")

    echo "Traitement de $filename"

    TARGET_DIR=$DATA_DIR/"$filename"Dir

    rm -R $TARGET_DIR 2> /dev/null
    mkdir $TARGET_DIR
    sed -i "s|\"matrix_path\": \".*\"|\"matrix_path\": \"$DATA_DIR/$filename\"|" "$CONF_FILE"
    sed -i "s|\"output_dir\": \".*\"|\"output_dir\": \"$TARGET_DIR\"|" "$CONF_FILE"

    COUNT=0

    until ./target/release/ranking -c conf.json -s && ./target/release/simulation_plots "$TARGET_DIR/results.csv" 1e-12 42; do
        COUNT=$((COUNT + 1))

        if [ "$COUNT" -ge "$MAX_RETRIES" ]; then
            echo "Failed for $filename after try $MAX_RETRIES."
            break
        fi

        echo "Retry ($COUNT/$MAX_RETRIES)..."
        sleep 2
    done
done
