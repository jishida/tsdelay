# tsdelay

### Description
Print audio delay.
```
$ tsdelay --help
tsdelay 0.1.0

USAGE:
    tsdelay [FLAGS] [OPTIONS] <SOURCE>

FLAGS:
    -d, --drop-broken-audio    Drops broken audio packets
    -h, --help                 Prints help information
    -V, --version              Prints version information

OPTIONS:
    -f, --format <FORMAT>      Selects display format [default: milli]  [possible values: milli, micro, real, real-
                               milli, real-micro, raw]
    -v, --video <VIDEO PID>    Specifies video PID
    -a, --audio <AUDIO PID>    Specifies audio PID

ARGS:
    <SOURCE>
$ tsdelay --audio 0x141 input.ts
-617
```

shell script example:
```bash
INPUT_FILE=input.ts
AUDIO_ID="$(
    ffmpeg -i "$INPUT_FILE" 2>&1 |
    grep -e '^ *Stream #0.*Audio' |
    head -n 1 |
    sed -e 's/^.*\[\(0x[0-9A-Z]\{1,\}\)\].*$/\1/'
)"
DELAY_MILLISECONDS="$(tsdelay --audio "$AUDIO_ID" "$INPUT_FILE")"
```
