#!/usr/bin/env bash

ffmpeg -framerate 0.5 -i %d.png -plays 0 -f apng output.png
