#/bin/sh
path_to_gsd_database="/home/ahrens/projects/public/gsd-database"
podman build -t json_validator .
podman run --env-file env-file -v "$path_to_gsd_database":/home/runner/work/gsd_database/gsd_database json_validator:latest
